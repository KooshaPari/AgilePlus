#!/usr/bin/env pwsh
# AgilePlus One-line Uninstaller (Windows)

$ErrorActionPreference = 'Stop'

$InstallDir = if ($env:AGILEPLUS_HOME) { $env:AGILEPLUS_HOME } else { "$env:LOCALAPPDATA\AgilePlus" }

Write-Host '==> Uninstalling AgilePlus...' -ForegroundColor Cyan

if (Test-Path $InstallDir) {
    Write-Host "    Removing $InstallDir..." -ForegroundColor Gray
    Remove-Item $InstallDir -Recurse -Force
}

$binPath = "$InstallDir\bin"
$currentPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if ($currentPath -like "*$binPath*") {
    $newPath = ($currentPath -split ';' | Where-Object { $_ -ne $binPath }) -join ';'
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
}

$paths = @(
    "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\PhenotypeApps\AgilePlus CLI.lnk",
    "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\PhenotypeApps\AgilePlus Dashboard.lnk",
    "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\PhenotypeApps\AgilePlus.lnk"
)
foreach ($p in $paths) {
    if (Test-Path $p) { Remove-Item $p -Force }
}

Write-Host 'AgilePlus uninstalled.' -ForegroundColor Green