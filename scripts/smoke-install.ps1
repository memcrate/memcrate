# Windows counterpart of smoke-install.sh: installs memcrate through one channel
# in a scratch home, then runs setup and checks what it wrote.
#
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts/smoke-install.ps1 [script|npm]
#
# script (the default) runs install.ps1 the way `irm | iex` does. It edits the
# user PATH in the registry; the exact original value is restored on exit.
# npm and MEMCRATE_EXPECT_VERSION work as in smoke-install.sh.

param(
    [ValidateSet('script', 'npm')]
    [string]$Channel = 'script'
)

$ErrorActionPreference = 'Stop'

function Fail($msg) { throw "FAIL: $msg" }

# npm warns on stderr, which Windows PowerShell 5.1 can turn into a terminating
# error; the exit code is what counts.
function Invoke-Native([string]$What, [scriptblock]$Block) {
    $ErrorActionPreference = 'Continue'
    & $Block
    if ($LASTEXITCODE -ne 0) { Fail "$What exited $LASTEXITCODE" }
}

# Registries can lag the release workflow by a minute or two.
function Wait-ForNpm {
    for ($i = 1; $i -le 20; $i++) {
        $latest = try { "$(Invoke-Native 'npm view' { npm view memcrate version })".Trim() } catch { 'unreachable' }
        if ($latest -eq $expect) { return }
        Write-Host "waiting for the npm registry to list $expect (latest is $latest)"
        Start-Sleep -Seconds 15
    }
    Fail "the npm registry still does not list $expect after 5 minutes"
}

# run() turns a failed skill install into a "Skipped" line and still exits 0.
function Invoke-Setup {
    $out = & $exe --yes | Out-String
    if ($LASTEXITCODE -ne 0) { Fail "memcrate --yes exited $LASTEXITCODE" }
    Write-Host $out
    if ($out -match 'Skipped') { Fail 'setup skipped a tool' }
    return $out
}

function Assert-Files {
    $vault = Join-Path $fakeHome 'memcrate-vault'
    foreach ($f in '.memcrate', 'Core\Context\Profile.md', 'Core\Context\Projects.md', 'Core\Context\Current State.md') {
        if (-not (Test-Path (Join-Path $vault $f))) { Fail "vault is missing $f" }
    }
    foreach ($tool in '.claude', '.codex') {
        foreach ($skill in 'load', 'save', 'pin') {
            $dir = Join-Path $fakeHome "$tool\skills\$skill"
            if (-not (Test-Path (Join-Path $dir 'SKILL.md'))) { Fail "missing $tool\skills\$skill\SKILL.md" }
            if (-not (Test-Path (Join-Path $dir '.memcrate-skill'))) { Fail "missing ownership marker in $tool\skills\$skill" }
        }
    }
}

$repo = Split-Path -Parent $PSScriptRoot
$scratch = Join-Path ([IO.Path]::GetTempPath()) "memcrate-smoke-$([Guid]::NewGuid().Guid)"
$binDir = Join-Path $scratch 'bin'
$fakeHome = Join-Path $scratch 'home'
New-Item -ItemType Directory -Path $fakeHome -Force | Out-Null
$expect = "$env:MEMCRATE_EXPECT_VERSION" -replace '^v', ''

if ($Channel -eq 'script') {
    $envKey = Get-Item 'HKCU:\Environment'
    $hadPath = $envKey.GetValueNames() -contains 'Path'
    if ($hadPath) {
        $savedPathKind = $envKey.GetValueKind('Path')
        $savedPathRaw = $envKey.GetValue('Path', $null, 'DoNotExpandEnvironmentNames')
    }
}
$savedSessionPath = $env:Path
$savedHome = $env:HOME
$savedProfile = $env:USERPROFILE
$savedInstallDir = $env:MEMCRATE_INSTALL_DIR

try {
    if ($Channel -eq 'script') {
        # A %VAR% entry the installer must leave unexpanded.
        $seed = '%SystemRoot%\memcrate-smoke-seed'
        $seeded = if ($savedPathRaw) { "$($savedPathRaw.TrimEnd(';'));$seed" } else { $seed }
        New-ItemProperty -Path 'HKCU:\Environment' -Name 'Path' -Value $seeded -PropertyType ExpandString -Force | Out-Null

        Write-Host "==> install.ps1 (PowerShell $($PSVersionTable.PSVersion))"
        $env:MEMCRATE_INSTALL_DIR = $binDir
        & { Get-Content -Raw (Join-Path $repo 'install.ps1') | Invoke-Expression }

        $exe = Join-Path $binDir 'memcrate.exe'
        if (-not (Test-Path $exe)) { Fail "no binary at $exe" }

        $envKey = Get-Item 'HKCU:\Environment'
        if ($envKey.GetValueKind('Path') -ne 'ExpandString') { Fail 'installer changed the user PATH registry type' }
        $rawPath = $envKey.GetValue('Path', $null, 'DoNotExpandEnvironmentNames')
        if ($rawPath -notlike "*$seed*") { Fail 'installer expanded %VAR% entries in the user PATH' }

        Write-Host '==> install.ps1 again (must leave the user PATH alone)'
        & { Get-Content -Raw (Join-Path $repo 'install.ps1') | Invoke-Expression }
        $rawAgain = (Get-Item 'HKCU:\Environment').GetValue('Path', $null, 'DoNotExpandEnvironmentNames')
        if ($rawAgain -cne $rawPath) { Fail 'second install changed the user PATH' }

        $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
        if (-not (($userPath -split ';') -contains $binDir)) { Fail 'install dir was not added to the user PATH' }
        $resolved = (Get-Command memcrate).Source
        if ($resolved -ne $exe) { Fail "memcrate on PATH resolves to $resolved, not $exe" }
    } else {
        Write-Host "==> npm install (PowerShell $($PSVersionTable.PSVersion))"
        if ($expect) {
            Wait-ForNpm
            $pkg = 'memcrate'
        } else {
            # postinstall downloads the release matching the package version, which
            # this checkout's version may not have yet.
            $expect = "$(Invoke-Native 'npm view' { npm view memcrate version })".Trim()
            $pkgDir = Join-Path $scratch 'pkg'
            Copy-Item -Recurse (Join-Path $repo 'npm') $pkgDir
            Push-Location $pkgDir
            try {
                Invoke-Native 'npm pkg set' { npm pkg set "version=$expect" }
                Invoke-Native 'npm pack' { npm pack --pack-destination $scratch } | Out-Null
            } finally {
                Pop-Location
            }
            $pkg = Join-Path $scratch "memcrate-$expect.tgz"
        }
        $prefix = Join-Path $scratch 'npm'
        Invoke-Native 'npm install' { npm install --global --prefix $prefix $pkg }

        $env:Path = "$prefix;$env:Path"
        $exe = (Get-Command memcrate).Source
        if ((Split-Path -Parent $exe) -ne $prefix) { Fail "memcrate on PATH resolves to $exe, not the npm shim in $prefix" }
    }

    if ($expect) {
        $got = (& $exe --version | Out-String).Trim()
        if ($LASTEXITCODE -ne 0) { Fail "memcrate --version exited $LASTEXITCODE" }
        if ($got -ne "memcrate $expect") { Fail "$Channel installed '$got', expected 'memcrate $expect'" }
        Write-Host "==> $Channel installed $got"
    }

    if ($Channel -eq 'npm') {
        Write-Host '==> npm shim passes the exit code through'
        $ErrorActionPreference = 'Continue'
        & $exe --no-such-flag *> $null
        $code = $LASTEXITCODE
        $ErrorActionPreference = 'Stop'
        if ($code -ne 2) { Fail "a bad flag exited $code through the npm shim; the binary exits 2" }
    }

    $env:HOME = $fakeHome
    $env:USERPROFILE = $fakeHome

    Write-Host '==> memcrate --yes'
    Invoke-Setup | Out-Null
    Assert-Files

    Write-Host '==> memcrate --yes again (must reuse the vault and refresh the skills)'
    $out = Invoke-Setup
    if ($out -notmatch 'Using the existing vault') { Fail 'second run did not reuse the vault' }
    Assert-Files

    Write-Host "smoke-install ($Channel): OK"
} finally {
    if ($Channel -eq 'script') {
        if ($hadPath) {
            New-ItemProperty -Path 'HKCU:\Environment' -Name 'Path' -Value $savedPathRaw -PropertyType $savedPathKind -Force | Out-Null
        } else {
            Remove-ItemProperty -Path 'HKCU:\Environment' -Name 'Path' -ErrorAction SilentlyContinue
        }
        # The installer told Explorer about its PATH change; this makes it re-read the restored value.
        [Environment]::SetEnvironmentVariable('MEMCRATE_SMOKE_UNUSED', $null, 'User')
    }
    $env:Path = $savedSessionPath
    $env:HOME = $savedHome
    $env:USERPROFILE = $savedProfile
    $env:MEMCRATE_INSTALL_DIR = $savedInstallDir
    Remove-Item -Recurse -Force $scratch -ErrorAction SilentlyContinue
}
