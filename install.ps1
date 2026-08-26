#!/usr/bin/env pwsh
# AgilePlus One-line Installer (Windows)
# Usage: irm https://raw.githubusercontent.com/KooshaPari/AgilePlus/main/install.ps1 | iex

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

# Configuration
$Repo = 'KooshaPari/AgilePlus'
$InstallDir = if ($env:AGILEPLUS_HOME) { $env:AGILEPLUS_HOME } else { "$env:LOCALAPPDATA\AgilePlus" }
$Version = if ($env:AGILEPLUS_VERSION) { $env:AGILEPLUS_VERSION } else { 'latest' }
$RepoRoot = "$env:TEMP\agileplus-install-$(Get-Random)"

Write-Host '==> AgilePlus Installer' -ForegroundColor Cyan
Write-Host "    Install dir: $InstallDir" -ForegroundColor Gray
Write-Host "    Version:     $Version" -ForegroundColor Gray

# 1. Check prerequisites
Write-Host '--> Checking prerequisites...' -ForegroundColor Cyan
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    throw 'Git is required. Install from https://git-scm.com/downloads'
}
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host '    Installing Rust via rustup...' -ForegroundColor Yellow
    Invoke-WebRequest 'https://win.rustup.rs/x86_64' -OutFile "$env:TEMP\rustup-init.exe"
    & "$env:TEMP\rustup-init.exe" -y --default-toolchain stable --profile minimal
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
}

# 2. Download AgilePlus
Write-Host '--> Downloading AgilePlus...' -ForegroundColor Cyan
New-Item -ItemType Directory -Path $RepoRoot -Force | Out-Null
Push-Location $RepoRoot
git clone --depth 1 --branch main "https://github.com/$Repo.git" . 2>&1 | Out-Null
Pop-Location

# 3. Build binaries
Write-Host '--> Building agileplus CLI + dashboard server (release mode)...' -ForegroundColor Cyan
Push-Location $RepoRoot
cargo build --release -p agileplus-cli -p agileplus-dashboard 2>&1 | Out-Null
Pop-Location

# 4. Install to LOCALAPPDATA
Write-Host '--> Installing to $InstallDir\bin...' -ForegroundColor Cyan
New-Item -ItemType Directory -Path "$InstallDir\bin" -Force | Out-Null
Copy-Item "$RepoRoot\target\release\agileplus.exe" "$InstallDir\bin\agileplus.exe" -Force
Copy-Item "$RepoRoot\target\release\agileplus-dashboard.exe" "$InstallDir\bin\agileplus-dashboard.exe" -Force

# 5. Add to PATH
$binPath = "$InstallDir\bin"
$currentPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if ($currentPath -notlike "*$binPath*") {
    [Environment]::SetEnvironmentVariable('Path', "$currentPath;$binPath", 'User')
    $env:Path = "$env:Path;$binPath"
}

# 6. Create Start Menu shortcuts
$shell = New-Object -ComObject WScript.Shell
New-Item -ItemType Directory -Path "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\PhenotypeApps" -Force | Out-Null

# CLI
$shortcut = $shell.CreateShortcut("$env:APPDATA\Microsoft\Windows\Start Menu\Programs\PhenotypeApps\AgilePlus CLI.lnk")
$shortcut.TargetPath = 'cmd.exe'
$shortcut.Arguments = "/k `"set PATH=$binPath;%PATH% && agileplus --help`""
$shortcut.WorkingDirectory = $InstallDir
$shortcut.Description = 'AgilePlus CLI - Governance & Spec-Driven Dev'
$shortcut.Save()

# Dashboard
$shortcut2 = $shell.CreateShortcut("$env:APPDATA\Microsoft\Windows\Start Menu\Programs\PhenotypeApps\AgilePlus Dashboard.lnk")
$shortcut2.TargetPath = 'cmd.exe'
$shortcut2.Arguments = "/c `"set PATH=$binPath;%PATH% && start /B agileplus-dashboard --port 3000 && timeout /t 2 >nul && start http://127.0.0.1:3000 && pause`""
$shortcut2.WorkingDirectory = $InstallDir
$shortcut2.Description = 'AgilePlus Dashboard'
$shortcut2.Save()

# 7. Cleanup
Remove-Item $RepoRoot -Recurse -Force

# 8. Verify
Write-Host '--> Verifying installation...' -ForegroundColor Cyan
Write-Host "    agileplus:           $InstallDir\bin\agileplus.exe" -ForegroundColor Green
Write-Host "    agileplus-dashboard: $InstallDir\bin\agileplus-dashboard.exe" -ForegroundColor Green
Write-Host "    Start Menu:          PhenotypeApps\AgilePlus CLI" -ForegroundColor Green
Write-Host "                        PhenotypeApps\AgilePlus Dashboard" -ForegroundColor Green

Write-Host ''
Write-Host 'AgilePlus installed successfully!' -ForegroundColor Green
Write-Host ''
Write-Host 'To use:' -ForegroundColor Cyan
Write-Host '  agileplus rubric score --repo C:\path\to\project' -ForegroundColor White
Write-Host '  agileplus dag pick' -ForegroundColor White
Write-Host '  agileplus cockpit publish' -ForegroundColor White