# NewsPredict Visual Polish — Design Spec

**Date:** 2026-03-26
**Status:** Approved
**Scope:** Market card images, category icons, data visualization enhancements

---

## 1. Requirements Summary

NewsPredict currently uses CSS gradients as placeholders for all visual elements. No actual images, no category-specific icons, and minimal data visualization beyond a basic SVG price chart. This spec covers three areas of visual improvement.

### Success Criteria

- Every market card displays a category-relevant image (fallback when DB `imageUrl` is null)
- Each category has a unique colored icon in pills, cards, and detail pages
- Price changes animate visually (flash green/red)
- Portfolio stat display enhanced with larger numbers and trend arrows
- Zero external runtime dependencies (all images pre-downloaded to `public/`)
- All new components follow existing i18n patterns
- Respects `prefers-reduced-motion` for animations

---

## 2. Category Taxonomy — Single Source of Truth

The DB constraint (`schema.ts:45`) defines the authoritative categories:

```
geopolitics, economics, science, technology, health, environment, sports, entertainment, politics, other
```

A mapping layer normalizes between DB values and UI display:

| DB Value | UI Display (en) | UI Display (zh) | Image Folder | Icon |
|----------|----------------|-----------------|-------------|------|
| `economics` | Markets | 市场 | `economics/` | `TrendingUp` |
| `politics` | Politics | 政治 | `politics/` | `Globe` |
| `technology` | Tech | 科技 | `technology/` | `Cpu` |
| `geopolitics` | Conflict | 冲突 | `geopolitics/` | `AlertTriangle` |
| `environment` | Climate | 气候 | `environment/` | `CloudRain` |
| `health` | Health | 健康 | `health/` | `Heart` |
| `science` | Science | 科学 | `science/` | `Atom` |
| `entertainment` | Entertainment | 娱乐 | `entertainment/` | `Film` |
| `sports` | Sports | 体育 | `sports/` | `Trophy` |
| `other` | Other | 其他 | `other/` | `HelpCircle` |

`CategoryPills` currently uses display names for filtering. The `CategoryIcon` component accepts **DB values** and handles display internally.

---

## 3. Market Card Category Images

### Approach

Pre-download 3-5 images per category from Unsplash (free commercial license). Store in `public/images/categories/{db_category}/`. Select image via deterministic hash of market ID.

**Only used as fallback** — if a market already has `imageUrl` in the database, that takes priority (existing card logic).

### File Structure

```
public/images/categories/
├── economics/     (4 images: finance, trading, currency, charts)
├── politics/      (3 images: government, diplomacy, parliament)
├── technology/    (4 images: circuits, code, devices, AI)
├── geopolitics/   (3 images: maps, military, borders)
├── environment/   (3 images: climate, storms, nature)
├── health/        (3 images: medical, lab, hospital)
├── science/       (3 images: space, research, microscope)
├── entertainment/ (3 images: media, stage, culture)
├── sports/        (3 images: stadium, competition, athletics)
└── other/         (3 images: generic abstract)
```

### Image Specifications

- Format: WebP only (all modern browsers support it)
- Dimensions: 800x400px (2:1 aspect ratio)
- File size target: <80KB per image
- Total: ~32 images x 80KB = ~2.5MB
- Pre-download and compress before commit (manual step using `cwebp` or squoosh.app)

### Implementation

**New utility:** `src/lib/category-image.ts`

```typescript
const CATEGORY_IMAGE_COUNT: Record<string, number> = {
  economics: 4, politics: 3, technology: 4, geopolitics: 3,
  environment: 3, health: 3, science: 3, entertainment: 3,
  sports: 3, other: 3,
};

function simpleHash(str: string): number {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    hash = ((hash << 5) - hash + str.charCodeAt(i)) | 0;
  }
  return Math.abs(hash);
}

export function getCategoryImage(category: string, marketId: string): string {
  const cat = CATEGORY_IMAGE_COUNT[category] ? category : 'other';
  const count = CATEGORY_IMAGE_COUNT[cat];
  const index = (simpleHash(marketId) % count) + 1;
  return `/images/categories/${cat}/${index}.webp`;
}
```

**Modified components:**
- `src/components/cards/hero-card.tsx` — when `imageUrl` prop is undefined, use `getCategoryImage(category, marketId)` (needs `marketId` prop added)
- `src/components/cards/news-prediction-card.tsx` — same fallback logic
- `src/app/page.tsx` — import `getCategoryImage`, pass computed `imageUrl` in data mapping (lines 60-75)
- `src/app/markets/page.tsx` — pass computed `imageUrl` to list items

**Accessibility:** Cards use CSS `background-image` so no `alt` text needed (decorative). Screen readers get the market question text from the `<h3>`.

---

## 4. Colored Category Icons

### Component

**New file:** `src/components/ui/category-icon.tsx` (new `ui/` directory)

```typescript
interface CategoryIconProps {
  category: string;      // DB category value
  size?: 'sm' | 'md' | 'lg';  // 24px, 32px, 40px
}
```

### Color Map

| DB Category | Gradient | Lucide Icon |
|-------------|----------|-------------|
| `economics` | `#00d4ff → #0066ff` | `TrendingUp` |
| `politics` | `#f59e0b → #ef4444` | `Globe` |
| `technology` | `#8b5cf6 → #6366f1` | `Cpu` |
| `geopolitics` | `#ef4444 → #dc2626` | `AlertTriangle` |
| `environment` | `#22c55e → #16a34a` | `CloudRain` |
| `health` | `#ec4899 → #db2777` | `Heart` |
| `science` | `#06b6d4 → #0891b2` | `Atom` |
| `entertainment` | `#f97316 → #ea580c` | `Film` |
| `sports` | `#eab308 → #ca8a04` | `Trophy` |
| `other` | `#6b7280 → #4b5563` | `HelpCircle` |

### Usage Locations

- `CategoryPills` — add small icon before text label (needs DB value → display name mapping)
- `NewsPredictionCard` — replace category letter placeholder with `CategoryIcon size="sm"`
- `HeroCard` — category badge gets icon
- Market detail page (`markets/[id]/page.tsx`) — category header

### Accessibility

`CategoryIcon` renders `aria-hidden="true"` since the category name is always shown as text alongside the icon.

---

## 5. Data Visualization Enhancements

### 5a. Price Flash Animation

**Mechanism:** In `MarketDetailClient` (`client.tsx`), use `useRef` to store previous price. When `live.yesPrice` differs from ref, apply a CSS class for 600ms.

```typescript
// In client.tsx
const prevYes = useRef(yesPrice);
const [flashClass, setFlashClass] = useState('');

useEffect(() => {
  if (live.yesPrice !== prevYes.current) {
    setFlashClass(live.yesPrice > prevYes.current ? 'animate-flash-green' : 'animate-flash-red');
    prevYes.current = live.yesPrice;
    const timer = setTimeout(() => setFlashClass(''), 600);
    return () => clearTimeout(timer);
  }
}, [live.yesPrice]);
```

**CSS keyframes** added to `globals.css`:
```css
@keyframes flash-green { 0% { background-color: rgba(34,197,94,0.2) } 100% { background-color: transparent } }
@keyframes flash-red { 0% { background-color: rgba(239,68,68,0.2) } 100% { background-color: transparent } }

@media (prefers-reduced-motion: reduce) {
  .animate-flash-green, .animate-flash-red { animation: none; }
}
```

**Files modified:** `src/app/globals.css`, `src/app/markets/[id]/client.tsx`

### 5b. DonutGauge Enhancement

**Current state:** `src/components/market/donut-gauge.tsx` — already has percentage text in center, default size 80px.

**Actual enhancements (not duplicating existing features):**
- Add entrance animation: `stroke-dasharray` transition on mount (CSS transition, not JS)
- Add subtle glow effect matching the gauge color

**File modified:** `src/components/market/donut-gauge.tsx`

### 5c. Portfolio Stat Cards

**Current state:** `SummaryBar` already has monospace font, trend arrows, and stat layout with glass effect.

**Enhancements:**
- Make numbers larger (text-sm → text-lg for the portfolio balance/PnL section)
- Add subtle pulsing glow on positive PnL
- Balance/PnL section in `portfolio/page.tsx` (lines 83-100): increase number size, add trend indicator arrow

**Files modified:** `src/app/portfolio/page.tsx` (balance/PnL card sizing)

---

## 6. Risks and Mitigations

| Risk | Mitigation |
|------|-----------|
| Images increase container size by ~2.5MB | Acceptable for a PWA; WebP compression keeps it small |
| Category from CHAOS doesn't match DB constraint | `getCategoryImage()` falls back to `other` for unknown categories |
| CSS background images not lazy-loaded | Images are small (<80KB) and only 1-2 visible above the fold; acceptable |
| Price flash too distracting | 600ms subtle background-color fade; respects `prefers-reduced-motion` |
| Hash function uneven distribution | `simpleHash` with modulo is sufficient for 3-4 images per bucket |

---

## 7. Out of Scope

- User avatar images (keep letter initials)
- AI-generated market images
- Sparkline mini-charts in list cards (future)
- Trading volume bar charts (future)
- Dark/light theme toggle (dark only)
- Switching from CSS background to `next/image` (not worth the card component refactor)

---

## 8. File Change Summary

| File | Action | Description |
|------|--------|-------------|
| `public/images/categories/**/*.webp` | New | ~32 category images |
| `src/lib/category-image.ts` | New | Deterministic image selection utility |
| `src/components/ui/category-icon.tsx` | New | Colored category icon component |
| `src/components/cards/hero-card.tsx` | Modify | Category image fallback, add `marketId` prop |
| `src/components/cards/news-prediction-card.tsx` | Modify | Category image + icon |
| `src/components/layout/category-pills.tsx` | Modify | Add category icons to pills |
| `src/components/market/donut-gauge.tsx` | Modify | Entrance animation + glow |
| `src/app/markets/[id]/client.tsx` | Modify | Price flash animation with `useRef` |
| `src/app/globals.css` | Modify | Flash keyframes + reduced-motion |
| `src/app/portfolio/page.tsx` | Modify | Larger numbers in balance/PnL card |
| `src/app/page.tsx` | Modify | Import `getCategoryImage`, pass to cards |
| `src/app/markets/page.tsx` | Modify | Pass category images to list items |
