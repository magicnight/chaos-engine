#!/usr/bin/env bash
# CHAOS Engine + NewsPredict — Development Startup (Linux/Mac)
# Usage: ./scripts/dev-start.sh [--rebuild]
set -euo pipefail
cd "$(dirname "$0")/.."

RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'; NC='\033[0m'
info()  { echo -e "${CYAN}[dev]${NC} $1"; }
ok()    { echo -e "${GREEN}[dev]${NC} $1"; }
err()   { echo -e "${RED}[dev]${NC} $1"; }

# --- Pre-flight checks ---
if ! command -v docker &>/dev/null; then
  err "docker not found. Install Docker: https://docs.docker.com/get-docker/"
  exit 1
fi
if ! docker compose version &>/dev/null; then
  err "docker compose not found. Install Docker Compose V2."
  exit 1
fi

# --- .env setup ---
if [ ! -f .env ]; then
  info "Creating .env from .env.example..."
  cp .env.example .env
  # Generate random secrets for dev
  RAND_SECRET=$(openssl rand -hex 16 2>/dev/null || head -c 32 /dev/urandom | base64 | tr -d '/+=' | head -c 32)
  sed -i.bak "s/^NEXTAUTH_SECRET=.*/NEXTAUTH_SECRET=${RAND_SECRET}/" .env
  RAND_CRON=$(openssl rand -hex 8 2>/dev/null || head -c 16 /dev/urandom | base64 | tr -d '/+=' | head -c 16)
  sed -i.bak "s/^CRON_SECRET=.*/CRON_SECRET=${RAND_CRON}/" .env
  rm -f .env.bak
  ok ".env created with random secrets"
else
  ok ".env already exists"
fi

# --- Build & Start ---
BUILD_FLAG=""
if [ "${1:-}" = "--rebuild" ]; then
  BUILD_FLAG="--build"
  info "Force rebuild requested"
fi

info "Starting all services..."
docker compose -f docker-compose.dev.yml up -d $BUILD_FLAG

# --- Wait for PostgreSQL ---
info "Waiting for PostgreSQL..."
for i in $(seq 1 30); do
  if docker compose -f docker-compose.dev.yml exec -T postgres pg_isready -U chaos -d newspredict &>/dev/null; then
    ok "PostgreSQL ready"
    break
  fi
  if [ "$i" -eq 30 ]; then
    err "PostgreSQL failed to start within 30s"
    docker compose -f docker-compose.dev.yml logs postgres
    exit 1
  fi
  sleep 1
done

# --- Run DB migration ---
info "Running database migration..."
docker compose -f docker-compose.dev.yml exec -T postgres \
  psql -U chaos -d newspredict -f /dev/stdin < newspredict/drizzle/migrations/0000_tan_blue_blade.sql 2>/dev/null || true
ok "Database migration complete"

# --- Wait for CHAOS Engine ---
info "Waiting for CHAOS Engine health check..."
for i in $(seq 1 60); do
  if docker compose -f docker-compose.dev.yml exec -T chaos wget -q --spider http://localhost:3117/api/v1/health 2>/dev/null; then
    ok "CHAOS Engine healthy"
    break
  fi
  if [ "$i" -eq 60 ]; then
    err "CHAOS Engine not healthy after 60s (may still be starting)"
  fi
  sleep 2
done

# --- Seed markets ---
info "Seeding initial markets from CHAOS..."
CRON_SECRET=$(grep '^CRON_SECRET=' .env 2>/dev/null | cut -d= -f2 || echo "chaos-cron-dev-secret")
curl -s -H "x-cron-secret: ${CRON_SECRET}" http://localhost:8080/api/market-seeds >/dev/null 2>&1 || true

# --- Status ---
echo ""
ok "========================================="
ok "  CHAOS Engine + NewsPredict is running!"
ok "========================================="
echo ""
echo -e "  ${CYAN}App:${NC}       http://localhost:8080"
echo -e "  ${CYAN}Dashboard:${NC} http://localhost:8080/api/v1/health"
echo -e "  ${CYAN}API:${NC}       http://localhost:8080/api/v1/data"
echo ""
echo -e "  ${CYAN}Commands:${NC}"
echo "    docker compose -f docker-compose.dev.yml logs -f    # View logs"
echo "    docker compose -f docker-compose.dev.yml down        # Stop all"
echo "    ./scripts/dev-start.sh --rebuild                     # Rebuild & start"
echo ""
