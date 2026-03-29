#!/usr/bin/env bash
# Download category images from Unsplash
set -euo pipefail
cd "$(dirname "$0")/.."

OUT="public/images/categories"

# Full Unsplash image URLs (verified, free commercial license)
declare -A URLS

# Economics / Finance
URLS["economics/1"]="https://images.unsplash.com/photo-1611974789855-9c2a0a7236a3?w=800&h=400&fit=crop&q=80"
URLS["economics/2"]="https://images.unsplash.com/photo-1590283603385-17ffb3a7f29f?w=800&h=400&fit=crop&q=80"
URLS["economics/3"]="https://images.unsplash.com/photo-1642790106117-e829e14a795f?w=800&h=400&fit=crop&q=80"

# Politics
URLS["politics/1"]="https://images.unsplash.com/photo-1529107386315-e1a2ed48a620?w=800&h=400&fit=crop&q=80"
URLS["politics/2"]="https://images.unsplash.com/photo-1541872703-74c5e44368f9?w=800&h=400&fit=crop&q=80"
URLS["politics/3"]="https://images.unsplash.com/photo-1575320181282-9afab399332c?w=800&h=400&fit=crop&q=80"

# Technology
URLS["technology/1"]="https://images.unsplash.com/photo-1518770660439-4636190af475?w=800&h=400&fit=crop&q=80"
URLS["technology/2"]="https://images.unsplash.com/photo-1550751827-4bd374c3f58b?w=800&h=400&fit=crop&q=80"
URLS["technology/3"]="https://images.unsplash.com/photo-1488590528505-98d2b5aba04b?w=800&h=400&fit=crop&q=80"

# Geopolitics / Conflict
URLS["geopolitics/1"]="https://images.unsplash.com/photo-1524661135-423995f22d0b?w=800&h=400&fit=crop&q=80"
URLS["geopolitics/2"]="https://images.unsplash.com/photo-1580752300992-559f8e44475b?w=800&h=400&fit=crop&q=80"
URLS["geopolitics/3"]="https://images.unsplash.com/photo-1542281286-9e0a16bb7366?w=800&h=400&fit=crop&q=80"

# Environment / Climate
URLS["environment/1"]="https://images.unsplash.com/photo-1470071459604-3b5ec3a7fe05?w=800&h=400&fit=crop&q=80"
URLS["environment/2"]="https://images.unsplash.com/photo-1534088568595-a066f410bcda?w=800&h=400&fit=crop&q=80"
URLS["environment/3"]="https://images.unsplash.com/photo-1561470508-fd4df1ed90b2?w=800&h=400&fit=crop&q=80"

# Health
URLS["health/1"]="https://images.unsplash.com/photo-1576091160399-112ba8d25d1d?w=800&h=400&fit=crop&q=80"
URLS["health/2"]="https://images.unsplash.com/photo-1579684385127-1ef15d508118?w=800&h=400&fit=crop&q=80"
URLS["health/3"]="https://images.unsplash.com/photo-1532938911079-1b06ac7ceec7?w=800&h=400&fit=crop&q=80"

# Science
URLS["science/1"]="https://images.unsplash.com/photo-1446776811953-b23d57bd21aa?w=800&h=400&fit=crop&q=80"
URLS["science/2"]="https://images.unsplash.com/photo-1507413245164-6160d8298b31?w=800&h=400&fit=crop&q=80"
URLS["science/3"]="https://images.unsplash.com/photo-1451187580459-43490279c0fa?w=800&h=400&fit=crop&q=80"

# Entertainment
URLS["entertainment/1"]="https://images.unsplash.com/photo-1514525253161-7a46d19cd819?w=800&h=400&fit=crop&q=80"
URLS["entertainment/2"]="https://images.unsplash.com/photo-1470229722913-7c0e2dbbafd3?w=800&h=400&fit=crop&q=80"
URLS["entertainment/3"]="https://images.unsplash.com/photo-1603190287605-e6ade32fa852?w=800&h=400&fit=crop&q=80"

# Sports
URLS["sports/1"]="https://images.unsplash.com/photo-1461896836934-bd45ba8c8e36?w=800&h=400&fit=crop&q=80"
URLS["sports/2"]="https://images.unsplash.com/photo-1517649763962-0c623066013b?w=800&h=400&fit=crop&q=80"
URLS["sports/3"]="https://images.unsplash.com/photo-1579952363873-27f3bade9f55?w=800&h=400&fit=crop&q=80"

# Other
URLS["other/1"]="https://images.unsplash.com/photo-1557683316-973673baf926?w=800&h=400&fit=crop&q=80"
URLS["other/2"]="https://images.unsplash.com/photo-1558591710-4b4a1ae0f04d?w=800&h=400&fit=crop&q=80"
URLS["other/3"]="https://images.unsplash.com/photo-1553356084-58ef4a67b2a7?w=800&h=400&fit=crop&q=80"

DOWNLOADED=0
FAILED=0

for key in "${!URLS[@]}"; do
  cat_name="${key%/*}"
  idx="${key#*/}"
  dir="${OUT}/${cat_name}"
  mkdir -p "$dir"
  file="${dir}/${idx}.jpg"

  if [ -f "$file" ] && [ -s "$file" ]; then
    echo "[skip] ${key}.jpg exists"
    continue
  fi

  echo -n "[dl] ${key}... "
  if curl -sfL -o "$file" "${URLS[$key]}" 2>/dev/null; then
    SIZE=$(du -sh "$file" | cut -f1)
    echo "OK (${SIZE})"
    DOWNLOADED=$((DOWNLOADED + 1))
  else
    echo "FAILED"
    rm -f "$file"
    FAILED=$((FAILED + 1))
  fi
done

echo ""
echo "Done: ${DOWNLOADED} downloaded, ${FAILED} failed"
