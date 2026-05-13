# Memcrate installer for Windows.
#
#   irm https://raw.githubusercontent.com/memcrate/memcrate/main/install.ps1 | iex
#
# Optional environment variables:
#   $env:MEMCRATE_VERSION      Specific version tag to install (default: latest).
#   $env:MEMCRATE_INSTALL_DIR  Where to put the binary
#                              (default: $env:LOCALAPPDATA\Programs\memcrate).

$ErrorActionPreference = 'Stop'

$Repo = 'memcrate/memcrate'
$BinName = 'memcrate.exe'

function Write-Info($msg) { Write-Host $msg }
function Throw-Err($msg) { throw "error: $msg" }

function Detect-Target {
    if ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture -eq 'Arm64') {
        Throw-Err "Windows ARM64 is not supported yet. Build from source: cargo install memcrate"
    }
    if (-not [Environment]::Is64BitOperatingSystem) {
        Throw-Err "32-bit Windows is not supported. Build from source: cargo install memcrate"
    }
    return 'x86_64-pc-windows-msvc'
}

function Resolve-InstallDir {
    if ($env:MEMCRATE_INSTALL_DIR) { return $env:MEMCRATE_INSTALL_DIR }
    return (Join-Path $env:LOCALAPPDATA 'Programs\memcrate')
}

function Add-ToUserPath($Dir) {
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if ($userPath -and ($userPath.Split(';') -contains $Dir)) { return $false }
    $newPath = if ($userPath) { "$userPath;$Dir" } else { $Dir }
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    return $true
}

$target = Detect-Target
$version = if ($env:MEMCRATE_VERSION) { $env:MEMCRATE_VERSION } else { 'latest' }
$installDir = Resolve-InstallDir

if ($version -eq 'latest') {
    $url = "https://github.com/$Repo/releases/latest/download/memcrate-$target.zip"
} else {
    $url = "https://github.com/$Repo/releases/download/$version/memcrate-$target.zip"
}

Write-Info "Memcrate installer"
Write-Info "  target:      $target"
Write-Info "  version:     $version"
Write-Info "  install dir: $installDir"
Write-Info ""
Write-Info "Downloading $url"

$tmp = Join-Path $env:TEMP "memcrate-install-$([Guid]::NewGuid().Guid)"
New-Item -ItemType Directory -Path $tmp -Force | Out-Null

try {
    $archivePath = Join-Path $tmp 'memcrate.zip'
    try {
        Invoke-WebRequest -Uri $url -OutFile $archivePath -UseBasicParsing
    } catch {
        Throw-Err "download failed. Check that version '$version' has a release asset for '$target' at https://github.com/$Repo/releases"
    }

    Expand-Archive -Path $archivePath -DestinationPath $tmp -Force

    $extractedBin = Join-Path $tmp $BinName
    if (-not (Test-Path $extractedBin)) {
        Throw-Err "archive did not contain expected binary '$BinName'. Contents: $((Get-ChildItem $tmp).Name -join ', ')"
    }

    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
    $dest = Join-Path $installDir $BinName
    Move-Item -Path $extractedBin -Destination $dest -Force

    Write-Info ""
    Write-Info "Installed: $dest"
    & $dest --version

    $added = Add-ToUserPath $installDir
    if ($added) {
        Write-Info ""
        Write-Info "Added $installDir to your user PATH (persists for new shells)."
    }

    # Also update the current session so `memcrate` works without opening a new terminal.
    if (-not ($env:Path.Split(';') -contains $installDir)) {
        $env:Path = "$installDir;$env:Path"
        Write-Info "Updated PATH for this session — you can run `memcrate` right now."
    }

    Write-Info ""
    Write-Info "Next:"
    Write-Info "  memcrate init `$HOME\vault    # scaffold a vault"
    Write-Info "  memcrate --help              # see available commands"
    Write-Info "  https://memcrate.dev         # docs"
} finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}
