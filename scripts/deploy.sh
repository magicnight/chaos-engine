#!/usr/bin/env bash
# CHAOS Engine + NewsPredict — Production Deployment (Linux Server)
# Usage: ./scripts/deploy.sh [--rebuild]
#
# Prerequisites:
#   - Docker & Docker Compose V2 installed
#   - .env configured with production values (secrets, API keys)
#   - Caddy/Nginx configured separately as reverse proxy
set -euo pipefail
cd "$(dirname "$0")/.."

RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'; YELLOW='\033[0;33m'; NC='\033[0m'
info()  { echo -e "${CYAN}[deploy]${NC} $1"; }
ok()    { echo -e "${GREEN}[deploy]${NC} $1"; }
warn()  { echo -e "${YELLOW}[deploy]${NC} $1"; }
err()   { echo -e "${RED}[deploy]${NC} $1"; }

# --- Pre-flight checks ---
if ! command -v docker &>/dev/null; then
  err "docker not found. Install Docker: https://docs.docker.com/get-docker/"
  exit 1
fi
if ! docker compose version &>/dev/null; then
  err "docker compose V2 not found."
  exit 1
fi

# --- .env validation ---
if [ ! -f .env ]; then
  err ".env not found! Copy .env.example to .env and configure production values."
  err "Required: NEXTAUTH_SECRET, CRON_SECRET, POSTGRES_PASSWORD"
  exit 1
fi

# Check required production variables
MISSING=""
for VAR in NEXTAUTH_SECRET CRON_SECRET POSTGRES_PASSWORD; do
  VAL=$(grep "^${VAR}=" .env 2>/dev/null | cut -d= -f2 || echo "")
  if [ -z "$VAL" ] || [ "$VAL" = "change-this-to-a-random-secret-in-production" ] || [ "$VAL" = "change-this-too" ] || [ "$VAL" = "chaos_secret" ]; then
    MISSING="${MISSING} ${VAR}"
  fi
done
if [ -n "$MISSING" ]; then
  err "Production secrets not configured in .env:${MISSING}"
  err "Generate secure values: openssl rand -hex 32"
  exit 1
fi

# --- Build & Deploy ---
BUILD_FLAG=""
if [ "${1:-}" = "--rebuild" ]; then
  BUILD_FLAG="--build"
  info "Force rebuild requested"
fi

info "Pulling base images..."
docker compose -f docker-compose.prod.yml pull postgres 2>/dev/null || true

info "Building and starting services..."
docker compose -f docker-compose.prod.yml up -d $BUILD_FLAG

# --- Wait for PostgreSQL ---
info "Waiting for PostgreSQL..."
for i in $(seq 1 30); do
  if docker compose -f docker-compose.prod.yml exec -T postgres pg_isready -U chaos -d newspredict &>/dev/null; then
    ok "PostgreSQL ready"
    break
  fi
  if [ "$i" -eq 30 ]; then
    err "PostgreSQL failed to start"
    exit 1
  fi
  sleep 1
done

# --- Run DB migration (idempotent) ---
info "Running database migration..."
docker compose -f docker-compose.prod.yml exec -T postgres \
  psql -U chaos -d newspredict -f /dev/stdin < newspredict/drizzle/migrations/0000_tan_blue_blade.sql 2>/dev/null || true
ok "Database migration complete"

# --- Wait for CHAOS Engine ---
info "Waiting for CHAOS Engine..."
for i in $(seq 1 90); do
  if docker compose -f docker-compose.prod.yml exec -T chaos wget -q --spider http://localhost:3117/api/v1/health 2>/dev/null; then
    ok "CHAOS Engine healthy"
    break
  fi
  if [ "$i" -eq 90 ]; then
    warn "CHAOS Engine not healthy after 90s (initial sweep may be in progress)"
  fi
  sleep 2
done

# --- Seed markets ---
info "Seeding markets from CHAOS data..."
CRON_SECRET=$(grep '^CRON_SECRET=' .env | cut -d= -f2)
curl -sf -H "x-cron-secret: ${CRON_SECRET}" http://localhost:3000/api/market-seeds >/dev/null 2>&1 || true

# --- Health check ---
info "Running final health check..."
sleep 3
SERVICES_UP=$(docker compose -f docker-compose.prod.yml ps --format "{{.Name}}" --filter "status=running" | wc -l)
if [ "$SERVICES_UP" -ge 3 ]; then
  ok "All 3 services running"
else
  warn "Only ${SERVICES_UP}/3 services running"
  docker compose -f docker-compose.prod.yml ps
fi

echo ""
ok "============================================"
ok "  Production deployment complete!"
ok "============================================"
echo ""
echo -e "  ${CYAN}NewsPredict:${NC}  http://localhost:3000"
echo -e "  ${CYAN}CHAOS API:${NC}    http://localhost:3117"
echo ""
echo -e "  ${CYAN}Reverse proxy:${NC}"
echo "    Configure Caddy/Nginx externally to proxy your domain to these ports"
echo "    Example Caddyfile:"
echo "      yourdomain.com {"
echo "        handle /api/v1/* { reverse_proxy localhost:3117 }"
echo "        handle * { reverse_proxy localhost:3000 }"
echo "      }"
echo ""
echo -e "  ${CYAN}Commands:${NC}"
echo "    docker compose -f docker-compose.prod.yml logs -f     # View logs"
echo "    docker compose -f docker-compose.prod.yml down         # Stop all"
echo "    docker compose -f docker-compose.prod.yml restart      # Restart"
echo "    ./scripts/deploy.sh --rebuild                          # Rebuild & deploy"
echo ""
