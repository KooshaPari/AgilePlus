# AgilePlus Dashboard Launcher
# Idempotent daemon launcher that:
# 1. Checks if AGILEPLUS_DASHBOARD_PORT is free
# 2. If free → starts agileplus-dashboard as hidden daemon
# 3. If occupied → health-checks http://localhost:$PORT/health
#    - Healthy → reuse existing instance
#    - Unhealthy/foreign → kill + start fresh
# 4. Opens native Electrobun window or fallback to browser

$ErrorActionPreference = "SilentlyContinue"

# Configuration
$DashboardBinary = "E:\agileplus-target\release\agileplus-dashboard.exe"
$DashboardPort = [int]$env:AGILEPLUS_DASHBOARD_PORT
if (-not $DashboardPort -or $DashboardPort -le 0) { $DashboardPort = 8770 }
$HealthCheckUrl = "http://127.0.0.1:$DashboardPort/health"
$DashboardUrl = "http://127.0.0.1:$DashboardPort"
$RepoPath = "E:\Dev\AgilePlus"

# Validate binary exists
if (-not (Test-Path $DashboardBinary)) {
    Write-Host "[ERROR] Dashboard binary not found: $DashboardBinary" -ForegroundColor Red
    Write-Host "[INFO] Build with: CARGO_TARGET_DIR=E:/agileplus-target cargo build --release -p agileplus-dashboard" -ForegroundColor Yellow
    timeout /t 3 > $null
    exit 1
}

# Helper: Check if port is in use
function Test-PortInUse {
    param([int]$Port)
    try {
        $tcpConnection = Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue
        return $null -ne $tcpConnection
    } catch {
        return $false
    }
}

# Helper: Health-check the dashboard
function Test-DashboardHealth {
    param([string]$Url)
    try {
        $response = Invoke-WebRequest -Uri $Url -TimeoutSec 2 -UseBasicParsing -ErrorAction Stop
        return $response.StatusCode -eq 200
    } catch {
        return $false
    }
}

# Helper: Kill any process using the port
function Kill-ProcessOnPort {
    param([int]$Port)
    try {
        $tcpConnection = Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue
        if ($tcpConnection) {
            $process = Get-Process -Id $tcpConnection.OwningProcess -ErrorAction SilentlyContinue
            if ($process) {
                Write-Host "[WARN] Killing process on port $Port : $($process.ProcessName)" -ForegroundColor Yellow
                Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
                Start-Sleep -Milliseconds 500
            }
        }
    } catch {
        # Silent catch
    }
}

# Main logic
Write-Host "[AgilePlus Dashboard Launcher]" -ForegroundColor Cyan

# Check if port is in use
$portInUse = Test-PortInUse $DashboardPort

if ($portInUse) {
    Write-Host "[INFO] Port $DashboardPort is occupied. Health-checking..." -ForegroundColor Blue
    if (Test-DashboardHealth $HealthCheckUrl) {
        Write-Host "[OK] Healthy dashboard found on port $DashboardPort. Reusing." -ForegroundColor Green
        # Proceed to open the URL
    } else {
        Write-Host "[WARN] Port $DashboardPort occupied by unhealthy service. Freeing & restarting..." -ForegroundColor Yellow
        Kill-ProcessOnPort $DashboardPort
        # Start fresh below
        $portInUse = $false
    }
}

if (-not $portInUse) {
    Write-Host "[INFO] Starting agileplus-dashboard on port $DashboardPort..." -ForegroundColor Blue
    Push-Location $RepoPath
    try {
        $env:AGILEPLUS_DASHBOARD_PORT = $DashboardPort
        # Start as hidden, detached daemon (no console window)
        $proc = Start-Process -FilePath $DashboardBinary `
            -NoNewWindow `
            -WindowStyle Hidden `
            -PassThru `
            -EnvironmentVariables @{ "AGILEPLUS_DASHBOARD_PORT" = $DashboardPort }

        Write-Host "[OK] Dashboard PID $($proc.Id) started (detached)" -ForegroundColor Green

        # Give it a moment to bind
        Start-Sleep -Seconds 2
    } catch {
        Write-Host "[ERROR] Failed to start dashboard: $_" -ForegroundColor Red
        exit 1
    } finally {
        Pop-Location
    }
}

# Now open the dashboard URL
Write-Host "[INFO] Opening dashboard at $DashboardUrl" -ForegroundColor Blue

# Try Electrobun first (native app container)
# NOTE: Electrobun wiring for AgilePlus is the end-state target; for now, fallback to browser
$electronBunPath = "C:\Users\koosh\AppData\Local\Apps\Electrobun\Electrobun.exe"
$electrobunAgilePlusApp = "E:\Dev\AgilePlus\packaging\electrobun-app.exe"  # Hypothetical final artifact

if (Test-Path $electrobunAgilePlusApp) {
    Write-Host "[INFO] Opening via Electrobun (AgilePlus native app)" -ForegroundColor Cyan
    Start-Process -FilePath $electrobunAgilePlusApp
} elseif (Test-Path $electronBunPath) {
    Write-Host "[INFO] Electrobun found but AgilePlus app not yet wired. (Target: native Electrobun app)" -ForegroundColor DarkYellow
    Write-Host "[INTERIM] Opening in default browser..." -ForegroundColor DarkYellow
    Start-Process $DashboardUrl
} else {
    Write-Host "[INTERIM] Opening in default browser (Electrobun not set up yet)" -ForegroundColor DarkYellow
    Start-Process $DashboardUrl
}

Write-Host "[OK] Done. Dashboard dashboard is live at $DashboardUrl" -ForegroundColor Green
exit 0
