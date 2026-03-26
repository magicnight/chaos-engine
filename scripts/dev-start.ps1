# CHAOS Engine + NewsPredict — Development Startup (Windows/Podman)
# Usage: .\scripts\dev-start.ps1 [-Rebuild]
param([switch]$Rebuild)

$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot)

function Info($msg)  { Write-Host "[dev] $msg" -ForegroundColor Cyan }
function Ok($msg)    { Write-Host "[dev] $msg" -ForegroundColor Green }
function Err($msg)   { Write-Host "[dev] $msg" -ForegroundColor Red }

# --- Pre-flight: find compose command ---
$COMPOSE = $null
if (Get-Command "podman-compose" -ErrorAction SilentlyContinue) {
    $COMPOSE = "podman-compose"
} elseif (Get-Command "python" -ErrorAction SilentlyContinue) {
    # Check if podman_compose module exists
    $check = python -c "import podman_compose" 2>&1
    if ($LASTEXITCODE -eq 0) {
        $COMPOSE = "python -m podman_compose"
    }
}
if (-not $COMPOSE) {
    if (Get-Command "docker" -ErrorAction SilentlyContinue) {
        $COMPOSE = "docker compose"
    }
}
if (-not $COMPOSE) {
    Err "No container runtime found. Install Podman or Docker."
    Err "  Podman: scoop install podman && pip install podman-compose"
    Err "  Docker: https://docs.docker.com/get-docker/"
    exit 1
}
Ok "Using: $COMPOSE"

# --- .env setup ---
if (-not (Test-Path ".env")) {
    Info "Creating .env from .env.example..."
    Copy-Item ".env.example" ".env"

    $secret = -join ((1..32) | ForEach-Object { [char](Get-Random -Min 97 -Max 123) })
    $cronSecret = -join ((1..16) | ForEach-Object { [char](Get-Random -Min 97 -Max 123) })

    (Get-Content ".env") -replace '^NEXTAUTH_SECRET=.*', "NEXTAUTH_SECRET=$secret" |
        Set-Content ".env"
    (Get-Content ".env") -replace '^CRON_SECRET=.*', "CRON_SECRET=$cronSecret" |
        Set-Content ".env"
    Ok ".env created with random secrets"
} else {
    Ok ".env already exists"
}

# --- Stop existing containers ---
Info "Stopping existing containers (if any)..."
Invoke-Expression "$COMPOSE -f docker-compose.dev.yml down 2>&1" | Out-Null

# --- Build & Start ---
$buildFlag = ""
if ($Rebuild) {
    $buildFlag = "--build"
    Info "Force rebuild requested"
}

Info "Starting all services..."
Invoke-Expression "$COMPOSE -f docker-compose.dev.yml up -d $buildFlag"

# --- Wait for PostgreSQL ---
Info "Waiting for PostgreSQL..."
$ready = $false
for ($i = 1; $i -le 30; $i++) {
    $result = podman exec chaos_postgres_1 pg_isready -U chaos -d newspredict 2>&1
    if ($LASTEXITCODE -eq 0) {
        Ok "PostgreSQL ready"
        $ready = $true
        break
    }
    Start-Sleep -Seconds 1
}
if (-not $ready) {
    Err "PostgreSQL failed to start within 30s"
    Invoke-Expression "$COMPOSE -f docker-compose.dev.yml logs postgres"
    exit 1
}

# --- Run DB migration ---
Info "Running database migration..."
$migrationFile = "newspredict/drizzle/migrations/0000_tan_blue_blade.sql"
if (Test-Path $migrationFile) {
    Get-Content $migrationFile | podman exec -i chaos_postgres_1 psql -U chaos -d newspredict 2>&1 | Out-Null
    Ok "Database migration complete"
} else {
    Err "Migration file not found: $migrationFile"
}

# --- Wait for CHAOS Engine ---
Info "Waiting for CHAOS Engine health check..."
$healthy = $false
for ($i = 1; $i -le 60; $i++) {
    try {
        $resp = podman exec chaos_chaos_1 wget -q --spider http://localhost:3117/api/v1/health 2>&1
        if ($LASTEXITCODE -eq 0) {
            Ok "CHAOS Engine healthy"
            $healthy = $true
            break
        }
    } catch {}
    Start-Sleep -Seconds 2
}
if (-not $healthy) {
    Info "CHAOS Engine not healthy yet (may still be running initial sweep)"
}

# --- Seed markets ---
Info "Seeding initial markets from CHAOS..."
$cronSecret = (Select-String -Path ".env" -Pattern "^CRON_SECRET=(.*)" | ForEach-Object { $_.Matches.Groups[1].Value })
if (-not $cronSecret) { $cronSecret = "chaos-cron-dev-secret" }
try {
    Invoke-RestMethod -Uri "http://localhost:8080/api/market-seeds" -Headers @{"x-cron-secret"=$cronSecret} -ErrorAction SilentlyContinue | Out-Null
} catch {}

# --- Status ---
Write-Host ""
Ok "========================================="
Ok "  CHAOS Engine + NewsPredict is running!"
Ok "========================================="
Write-Host ""
Write-Host "  App:       " -NoNewline -ForegroundColor Cyan; Write-Host "http://localhost:8080"
Write-Host "  Dashboard: " -NoNewline -ForegroundColor Cyan; Write-Host "http://localhost:8080/api/v1/health"
Write-Host "  API:       " -NoNewline -ForegroundColor Cyan; Write-Host "http://localhost:8080/api/v1/data"
Write-Host ""
Write-Host "  Commands:" -ForegroundColor Cyan
Write-Host "    $COMPOSE -f docker-compose.dev.yml logs -f    # View logs"
Write-Host "    $COMPOSE -f docker-compose.dev.yml down        # Stop all"
Write-Host "    .\scripts\dev-start.ps1 -Rebuild               # Rebuild & start"
Write-Host ""
