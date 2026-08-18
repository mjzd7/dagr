# ====================================================================
# ⚡ DAGR Universal PowerShell Installer for Windows
# Usage: irm https://raw.githubusercontent.com/mjzd7/dagr/main/scripts/install.ps1 | iex
# ====================================================================
$ErrorActionPreference = "Stop"

$InstallDir = "$env:USERPROFILE\.dagr\bin"
if (!(Test-Path -Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$BinaryPath = "$InstallDir\dagr.exe"
$DownloadUrl = "https://github.com/mjzd7/dagr/releases/latest/download/dagr-windows-x86_64.exe"

Write-Host "⚡ [DAGR] Downloading pre-compiled Windows x64 binary..." -ForegroundColor Cyan

try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $BinaryPath
} catch {
    Write-Host "⚠️  Could not download pre-compiled release. Trying cargo install..." -ForegroundColor Yellow
    cargo install --git https://github.com/mjzd7/dagr.git dagr --force
    $BinaryPath = "$env:USERPROFILE\.cargo\bin\dagr.exe"
}

# Add to user PATH if not present
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    $env:Path += ";$InstallDir"
}

Write-Host "🔌 [DAGR] Auto-configuring MCP and Agent Skills..." -ForegroundColor Cyan
& $BinaryPath mcp install --client all
& $BinaryPath skills install --target all

Write-Host "`n✅ [DAGR] Installation successful! Restart your editor to connect." -ForegroundColor Green
