#!/usr/bin/env bash
# Update YouTube live stream IDs for CHAOS Dashboard
# Usage: ./scripts/update-live-ids.sh
# Requires: YOUTUBE_API_KEY environment variable
#
# Queries YouTube Data API for each channel's current live stream.
# Updates static/js/video.js with new IDs.
# Run periodically (e.g., daily cron) to keep streams working.
set -euo pipefail
cd "$(dirname "$0")/.."

CYAN='\033[0;36m'; GREEN='\033[0;32m'; RED='\033[0;31m'; NC='\033[0m'
info() { echo -e "${CYAN}[yt]${NC} $1"; }
ok()   { echo -e "${GREEN}[yt]${NC} $1"; }
err()  { echo -e "${RED}[yt]${NC} $1"; }

YT_KEY="${YOUTUBE_API_KEY:-}"
if [ -z "$YT_KEY" ]; then
  err "YOUTUBE_API_KEY not set. Get one from https://console.cloud.google.com"
  exit 1
fi

VIDEO_JS="static/js/video.js"
if [ ! -f "$VIDEO_JS" ]; then
  err "$VIDEO_JS not found"
  exit 1
fi

# Channel ID → panel key mapping
declare -A CHANNELS=(
  ["UCIALMKvObZNtJ68-rmLjgdQ"]="live-bloomberg"     # Bloomberg Television
  ["UCNye-wNBqNL5ZzHSJj3l8Bg"]="live-aljazeera"     # Al Jazeera English
  ["UCQfwfsi5VrQ8yKZ-UWmAEFg"]="live-france24"      # FRANCE 24 English
  ["UCknLrEdhRCp1aegoMqRaCZg"]="live-dw"             # DW News
  ["UCW2QcKZiU8aUGg4yxCIditg"]="live-euronews"       # Euronews
  ["UCoMdktPbSTixAyNGwb-UYkQ"]="live-skynews"        # Sky News
  ["UCvJJ_dzjViJCoLf5uKUTwoA"]="live-cnbc"           # CNBC Television
  ["UCQ-afEGT-I0ick2Lsf3sAAQ"]="live-nhk"            # NHK WORLD-JAPAN
  ["UCFMnGq-xWnFSQlPOMaV6XSQ"]="live-tvbs"          # TVBS NEWS
  ["UC5l1Yto5oOIgRXlI4p4VKbw"]="live-cti"            # 中天電視
  ["UCR-H30JEwUXffQi42oLN3Cg"]="live-ebc"            # 東森新聞
  ["UCLyBnOUgR_xhy2Kk2qYvvSA"]="live-phoenix"        # 鳳凰衛視
  ["UCgrNz-aDmcr2uuto8_DL2jg"]="live-cgtn"           # CGTN
)

UPDATED=0

for CHANNEL_ID in "${!CHANNELS[@]}"; do
  PANEL_KEY="${CHANNELS[$CHANNEL_ID]}"
  info "Checking ${PANEL_KEY}..."

  # Search for active live broadcast
  RESPONSE=$(curl -sf "https://www.googleapis.com/youtube/v3/search?part=id&channelId=${CHANNEL_ID}&eventType=live&type=video&key=${YT_KEY}" 2>/dev/null || echo "")

  if [ -z "$RESPONSE" ]; then
    err "  API call failed for ${PANEL_KEY}"
    continue
  fi

  VIDEO_ID=$(echo "$RESPONSE" | python3 -c "
import sys, json
try:
    d = json.load(sys.stdin)
    items = d.get('items', [])
    if items:
        print(items[0]['id']['videoId'])
except: pass
" 2>/dev/null)

  if [ -n "$VIDEO_ID" ]; then
    # Update video.js
    OLD_LINE=$(grep "'${PANEL_KEY}':" "$VIDEO_JS" || echo "")
    if [ -n "$OLD_LINE" ]; then
      OLD_ID=$(echo "$OLD_LINE" | grep -o "ytId: '[^']*'" | cut -d"'" -f2)
      if [ "$OLD_ID" != "$VIDEO_ID" ]; then
        sed -i "s|'${PANEL_KEY}':.*{.*ytId:.*'[^']*'.*}|'${PANEL_KEY}':  { ytId: '${VIDEO_ID}' }|" "$VIDEO_JS"
        ok "  ${PANEL_KEY}: ${OLD_ID} → ${VIDEO_ID}"
        UPDATED=$((UPDATED + 1))
      else
        info "  ${PANEL_KEY}: unchanged (${VIDEO_ID})"
      fi
    fi
  else
    info "  ${PANEL_KEY}: no active live stream found"
  fi
done

echo ""
if [ "$UPDATED" -gt 0 ]; then
  ok "Updated ${UPDATED} stream IDs"
  ok "Rebuild CHAOS container to apply: podman rmi chaos_chaos --force && podman-compose -f docker-compose.dev.yml up -d --build chaos"
else
  info "No changes needed"
fi
