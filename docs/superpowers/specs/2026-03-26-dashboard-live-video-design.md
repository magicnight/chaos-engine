# CHAOS Dashboard Live Video Panels — Design Spec

**Date:** 2026-03-26
**Status:** Approved
**Scope:** YouTube Live iframe video panels for the CHAOS Engine dashboard

---

## 1. Requirements Summary

Add live news video streaming to the CHAOS Engine dashboard via YouTube Live iframe embeds. Each channel is an independent GridStack panel, consistent with the existing panel system.

### Success Criteria

- 15 news channels available as dashboard panels
- Video panels use the same drag/resize/hide/show mechanism as existing panels
- Default: all video panels hidden (no autoplay on load)
- Panels appear in panel settings under a "Live" category
- iframe destroyed when panel is hidden (no background resource usage)
- iframe uses loading="lazy" and allow="autoplay; encrypted-media"
- Channel names have i18n support (Chinese/English)
- YouTube video IDs are configurable per channel

---

## 2. Channel List (15 channels)

### International (8)

| Channel | Language |
|---------|----------|
| Bloomberg TV | English |
| Al Jazeera English | English |
| France 24 | English |
| DW News | English |
| Euronews | English |
| Sky News | English |
| CNBC | English |
| NHK World | English |

### Chinese (7)

| Channel | Language |
|---------|----------|
| CCTV-4 中文国际 | 中文 |
| TVBS 新闻台 | 中文 |
| 中天新闻 | 中文 |
| 东森新闻 | 中文 |
| 凤凰卫视 | 中文 |
| CGTN | English |
| 央视新闻 | 中文 |

Note: YouTube live stream video IDs change periodically. Stored as configurable values in LIVE_CHANNELS registry.

---

## 3. Implementation

### 3.1 Panel Definitions

Add 15 entries to the PANELS array with category "Live", default size 4x4, icon "📺".

### 3.2 Video Channel Registry

LIVE_CHANNELS object maps panel ID to { name, nameZh, ytId }. YouTube video IDs are default values that can be updated.

### 3.3 Update Function

One generic updateVideoPanel(panelId) function:
- Looks up channel in LIVE_CHANNELS
- Creates YouTube embed iframe (autoplay=1, mute=1, controls=1)
- Adds LIVE badge overlay
- Skips if iframe already exists (no duplicate creation)

### 3.4 Panel Lifecycle

- Show: user enables in settings -> iframe created -> video starts muted
- Hide: panel removed -> iframe destroyed -> resources freed
- Maximize: iframe stretches to fill via CSS absolute positioning

### 3.5 Integration

Panels with id starting with "live-" route to updateVideoPanel() in updatePanelById().

### 3.6 i18n

Add to I18N object: Live: { en: 'Live TV', zh: '直播频道' }

---

## 4. Risks and Mitigations

| Risk | Mitigation |
|------|-----------|
| YouTube live stream ID changes | Configurable in LIVE_CHANNELS, easy to update |
| Multiple iframes hurt performance | Default hidden, destroyed on hide, loading="lazy" |
| Autoplay blocked by browser | mute=1 — muted autoplay allowed by all browsers |
| YouTube blocked in some regions | Show "Stream unavailable" fallback message |
| Dashboard too crowded | Panels default hidden, user chooses which to show |

---

## 5. Out of Scope

- YouTube Data API auto-resolution of live IDs
- Non-YouTube sources (HLS/m3u8)
- Picture-in-picture across panels
- Recording/DVR
- Custom channel URL input

---

## 6. File Change Summary

| File | Action | Description |
|------|--------|-------------|
| static/dashboard.html | Modify | Add LIVE_CHANNELS, 15 PANELS entries, updateVideoPanel(), i18n keys |

Single file change.
