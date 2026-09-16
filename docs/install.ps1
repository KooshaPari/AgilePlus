#Requires -Version 5.1
# AgilePlus installer — irm | iex
# Usage: irm https://raw.githubusercontent.com/<REDACTED>/AgilePlus/main/docs/install.ps1 | iex

$ErrorActionPreference = "Stop"
$Repo = "<REDACTED>/AgilePlus"
$Binary = "agileplus"
$InstallDir = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { "$env:LOCALAPPDATA\AgilePlus\bin" }

# Get latest release
$release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
$tag = $release.tag_name
Write-Host "Installing AgilePlus $tag for Windows..."

$url = "https://github.com/$Repo/releases/download/$tag/agileplus-windows-x86_64.zip"
$tmp = Join-Path $env:TEMP "agileplus-install"
New-Item -ItemType Directory -Force -Path $tmp | Out-Null

Invoke-WebRequest -Uri $url -OutFile "$tmp\agileplus.zip"
Expand-Archive -Path "$tmp\agileplus.zip" -DestinationPath $tmp -Force

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item "$tmp\agileplus.exe" "$InstallDir\$Binary.exe" -Force

Remove-Item -Recurse -Force $tmp

# Add to PATH if not already
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$currentPath;$InstallDir", "User")
    Write-Host "Added $InstallDir to PATH (restart terminal to take effect)"
}

Write-Host "✓ Installed to $InstallDir\$Binary.exe"
