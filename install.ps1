# journal-cli installer and updater for Windows

$ErrorActionPreference = 'Stop'

$Repo = "DerSton/journal-cli"
$BinaryName = "journal-cli.exe"
$AliasName = "jnl.bat"
$InstallDir = "$env:LOCALAPPDATA\journal-cli"
$AssetName = "journal-cli-windows-x86_64.exe"

# Fetch latest release version from GitHub API (including pre-releases)
Write-Host "Fetching latest release version from GitHub..." -ForegroundColor Gray
$LatestTag = "latest"
try {
    # Ensure TLS 1.2 is used for GitHub API/download
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $Releases = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases" -UseBasicParsing
    if ($Releases -and $Releases.Count -gt 0) {
        $LatestTag = $Releases[0].tag_name
        Write-Host "Latest release found: $LatestTag" -ForegroundColor Gray
    }
} catch {
    Write-Host "Warning: Could not fetch latest release version from API. Falling back to latest redirect." -ForegroundColor Yellow
}

if ($LatestTag -eq "latest") {
    $DownloadUrl = "https://github.com/$Repo/releases/latest/download/$AssetName"
} else {
    $DownloadUrl = "https://github.com/$Repo/releases/download/$LatestTag/$AssetName"
}

Write-Host "=== journal-cli Installer ===" -ForegroundColor Cyan

# Check if 64-bit OS
if ([Environment]::Is64BitOperatingSystem -eq $false) {
    Write-Error "Error: Currently, prebuilt binaries are only provided for 64-bit Windows."
    exit 1
}

# Create installation directory if it doesn't exist
if (-not (Test-Path -Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$TargetPath = Join-Path $InstallDir $BinaryName
$AliasPath = Join-Path $InstallDir $AliasName
$TempPath = Join-Path $env:TEMP "journal-cli-temp.exe"

Write-Host "Downloading latest version of journal-cli..." -ForegroundColor Gray
try {
    # Ensure TLS 1.2 is used for GitHub download
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempPath -UseBasicParsing
} catch {
    Write-Host "Error: Failed to download journal-cli from $DownloadUrl." -ForegroundColor Red
    Write-Host "Please ensure that a release has been published at https://github.com/$Repo/releases" -ForegroundColor Yellow
    Write-Host "Details: $_" -ForegroundColor DarkGray
    if (Test-Path $TempPath) { Remove-Item $TempPath }
    exit 1
}

# Replace the old binary with the new one
try {
    if (Test-Path $TargetPath) {
        Remove-Item $TargetPath -Force
    }
    Move-Item -Path $TempPath -Destination $TargetPath -Force
} catch {
    Write-Host "Error: Could not overwrite the existing journal-cli.exe." -ForegroundColor Red
    Write-Host "Please make sure journal-cli is not running and try again." -ForegroundColor Yellow
    if (Test-Path $TempPath) { Remove-Item $TempPath }
    exit 1
}

# Create the jnl alias script (jnl.bat)
try {
    $BatchContent = "@echo off`r`njournal-cli.exe %*`r`n"
    [System.IO.File]::WriteAllText($AliasPath, $BatchContent)
    Write-Host "Created alias wrapper: jnl -> journal-cli.exe" -ForegroundColor Gray
} catch {
    Write-Host "Warning: Could not create jnl wrapper. You can still run journal-cli directly." -ForegroundColor Yellow
}

Write-Host "Successfully installed/updated journal-cli to $TargetPath" -ForegroundColor Green

# Add to PATH if not already present
$UserPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
$PathExists = $false
foreach ($Path in $UserPath -split ';') {
    if ($Path.Trim().TrimEnd('\') -eq $InstallDir.Trim().TrimEnd('\')) {
        $PathExists = $true
        break
    }
}

if (-not $PathExists) {
    Write-Host "Adding $InstallDir to User PATH environment variable..." -ForegroundColor Gray
    [Environment]::SetEnvironmentVariable("Path", $UserPath + ";" + $InstallDir, [EnvironmentVariableTarget]::User)
    $env:Path += ";$InstallDir"
    Write-Host "You may need to restart your terminal/PowerShell session for PATH changes to take effect." -ForegroundColor Yellow
}

# Finished
