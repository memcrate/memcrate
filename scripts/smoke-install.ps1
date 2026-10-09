# Windows counterpart of smoke-install.sh: runs install.ps1 the way `irm | iex`
# does, in a scratch home, then setup, and checks what it wrote.
#
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts/smoke-install.ps1
#
# install.ps1 edits the user PATH in the registry; the exact original value is restored on exit.

$ErrorActionPreference = 'Stop'

function Fail($msg) { throw "FAIL: $msg" }

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

$envKey = Get-Item 'HKCU:\Environment'
$hadPath = $envKey.GetValueNames() -contains 'Path'
if ($hadPath) {
    $savedPathKind = $envKey.GetValueKind('Path')
    $savedPathRaw = $envKey.GetValue('Path', $null, 'DoNotExpandEnvironmentNames')
}
$savedSessionPath = $env:Path
$savedHome = $env:HOME
$savedProfile = $env:USERPROFILE
$savedInstallDir = $env:MEMCRATE_INSTALL_DIR

try {
    Write-Host "==> install.ps1 (PowerShell $($PSVersionTable.PSVersion))"
    $env:MEMCRATE_INSTALL_DIR = $binDir
    & { Get-Content -Raw (Join-Path $repo 'install.ps1') | Invoke-Expression }

    $exe = Join-Path $binDir 'memcrate.exe'
    if (-not (Test-Path $exe)) { Fail "no binary at $exe" }

    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if (-not (($userPath -split ';') -contains $binDir)) { Fail 'install dir was not added to the user PATH' }
    $resolved = (Get-Command memcrate).Source
    if ($resolved -ne $exe) { Fail "memcrate on PATH resolves to $resolved, not $exe" }

    $env:HOME = $fakeHome
    $env:USERPROFILE = $fakeHome

    Write-Host '==> memcrate --yes'
    Invoke-Setup | Out-Null
    Assert-Files

    Write-Host '==> memcrate --yes again (must reuse the vault and refresh the skills)'
    $out = Invoke-Setup
    if ($out -notmatch 'Using the existing vault') { Fail 'second run did not reuse the vault' }
    Assert-Files

    Write-Host 'smoke-install: OK'
} finally {
    if ($hadPath) {
        New-ItemProperty -Path 'HKCU:\Environment' -Name 'Path' -Value $savedPathRaw -PropertyType $savedPathKind -Force | Out-Null
    } else {
        Remove-ItemProperty -Path 'HKCU:\Environment' -Name 'Path' -ErrorAction SilentlyContinue
    }
    # The installer told Explorer about its PATH change; this makes it re-read the restored value.
    [Environment]::SetEnvironmentVariable('MEMCRATE_SMOKE_UNUSED', $null, 'User')
    $env:Path = $savedSessionPath
    $env:HOME = $savedHome
    $env:USERPROFILE = $savedProfile
    $env:MEMCRATE_INSTALL_DIR = $savedInstallDir
    Remove-Item -Recurse -Force $scratch -ErrorAction SilentlyContinue
}
