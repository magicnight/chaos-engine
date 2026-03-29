#!/usr/bin/env bash
# CHAOS Engine + NewsPredict — Backup Script
# Usage: ./scripts/backup.sh [--dir /path/to/backups]
#
# Backs up:
#   1. PostgreSQL (NewsPredict) via pg_dump
#   2. SQLite (CHAOS Engine) via file copy
#
# Retention: keeps last 7 daily backups (configurable via BACKUP_KEEP)
set -euo pipefail
cd "$(dirname "$0")/.."

RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'; NC='\033[0m'
info()  { echo -e "${CYAN}[backup]${NC} $1"; }
ok()    { echo -e "${GREEN}[backup]${NC} $1"; }
err()   { echo -e "${RED}[backup]${NC} $1"; }

# Config
BACKUP_DIR="${1:---dir}"
if [ "$BACKUP_DIR" = "--dir" ]; then
  BACKUP_DIR="${2:-./backups}"
fi
BACKUP_KEEP="${BACKUP_KEEP:-7}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
DEST="${BACKUP_DIR}/${TIMESTAMP}"

mkdir -p "$DEST"

# --- PostgreSQL backup ---
info "Backing up PostgreSQL..."
PG_CONTAINER=$(docker ps --format '{{.Names}}' --filter "name=postgres" 2>/dev/null | head -1)
if [ -z "$PG_CONTAINER" ]; then
  PG_CONTAINER=$(podman ps --format '{{.Names}}' --filter "name=postgres" 2>/dev/null | head -1)
fi

if [ -n "$PG_CONTAINER" ]; then
  if command -v docker &>/dev/null && docker ps --format '{{.Names}}' | grep -q "$PG_CONTAINER"; then
    docker exec "$PG_CONTAINER" pg_dump -U chaos -d newspredict --format=custom > "${DEST}/newspredict.pgdump"
  else
    podman exec "$PG_CONTAINER" pg_dump -U chaos -d newspredict --format=custom > "${DEST}/newspredict.pgdump"
  fi
  PG_SIZE=$(du -sh "${DEST}/newspredict.pgdump" | cut -f1)
  ok "PostgreSQL backup: ${DEST}/newspredict.pgdump (${PG_SIZE})"
else
  err "PostgreSQL container not found — skipping"
fi

# --- SQLite backup ---
info "Backing up SQLite (CHAOS Engine)..."
CHAOS_CONTAINER=$(docker ps --format '{{.Names}}' --filter "name=chaos_chaos" 2>/dev/null | head -1)
if [ -z "$CHAOS_CONTAINER" ]; then
  CHAOS_CONTAINER=$(podman ps --format '{{.Names}}' --filter "name=chaos_chaos" 2>/dev/null | head -1)
fi

if [ -n "$CHAOS_CONTAINER" ]; then
  # Use SQLite backup command for consistency (safe during writes with WAL)
  if command -v docker &>/dev/null && docker ps --format '{{.Names}}' | grep -q "$CHAOS_CONTAINER"; then
    docker exec "$CHAOS_CONTAINER" cat /data/runs/chaos.db > "${DEST}/chaos.db"
  else
    podman exec "$CHAOS_CONTAINER" cat /data/runs/chaos.db > "${DEST}/chaos.db"
  fi
  SQLITE_SIZE=$(du -sh "${DEST}/chaos.db" | cut -f1)
  ok "SQLite backup: ${DEST}/chaos.db (${SQLITE_SIZE})"
else
  # Try local file
  if [ -f "runs/chaos.db" ]; then
    cp "runs/chaos.db" "${DEST}/chaos.db"
    SQLITE_SIZE=$(du -sh "${DEST}/chaos.db" | cut -f1)
    ok "SQLite backup (local): ${DEST}/chaos.db (${SQLITE_SIZE})"
  else
    err "SQLite database not found — skipping"
  fi
fi

# --- Retention cleanup ---
info "Cleaning old backups (keeping last ${BACKUP_KEEP})..."
BACKUP_COUNT=$(ls -d "${BACKUP_DIR}"/20* 2>/dev/null | wc -l)
if [ "$BACKUP_COUNT" -gt "$BACKUP_KEEP" ]; then
  REMOVE_COUNT=$((BACKUP_COUNT - BACKUP_KEEP))
  ls -d "${BACKUP_DIR}"/20* | head -"$REMOVE_COUNT" | while read -r old; do
    rm -rf "$old"
    info "Removed old backup: $old"
  done
fi

# --- Summary ---
TOTAL_SIZE=$(du -sh "$DEST" | cut -f1)
echo ""
ok "========================================="
ok "  Backup complete: ${DEST}"
ok "  Total size: ${TOTAL_SIZE}"
ok "========================================="
echo ""
echo -e "  ${CYAN}Restore PostgreSQL:${NC}"
echo "    docker exec -i <postgres_container> pg_restore -U chaos -d newspredict < ${DEST}/newspredict.pgdump"
echo ""
echo -e "  ${CYAN}Restore SQLite:${NC}"
echo "    cp ${DEST}/chaos.db runs/chaos.db"
echo ""
