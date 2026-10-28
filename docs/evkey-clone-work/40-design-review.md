# 40 — Design review, pass 1: three visual directions for the settings window

**Owner:** BA/Design. **Decision owner:** the human owner (charter §5: design must not decide taste alone).
**Input:** `20-spec.md` §C (behavior), charter C2/C7, contract §7 (Svelte 5, hand-written CSS tokens,
no UI kit, inline SVG icons only).
**Not yet done:** the offline HTML preview (plan 0.5) — produced after the owner shortlists a direction.

## 0. Shared constraints (all directions)

| Item | Value |
|---|---|
| Window | 980 × 680 outer, fixed default, min 860 × 600, resizable |
| Client area | native Win11 title bar (~32 px, `UNVERIFIED` exact; method: `GetSystemMetrics(SM_CYCAPTION)` + frame at 100 % scale) → working height **648 px**, width **980 px** |
| Grid | 4 px base; every size below is a multiple of 4 |
| Scaling | everything in CSS px; WebView2 handles 125/150 % DPI |
| Motion | `prefers-reduced-motion: reduce` → all durations 0 ms, no pulsing |
| Focus ring | 2 px solid accent + 2 px offset, on every interactive element |
| Status strip | present in all directions (spec C.0): on/off, kiểu gõ, bảng mã, app override, each clickable |
| Vietnamese text | body line-height ≥ 1.5, control line-height ≥ 1.43 (20/14) — stacked marks (`Ấ Ẩ Ẫ Ổ Ở Ữ`) need ~20 % extra ascent; no `overflow: hidden` on single-line text with line-height < 1.4 |
| Anti-goals | no group-box borders with captions, no grey 3D buttons, no OK/Cancel/Apply row, no modal for any setting, no more than one accent colour |

WCAG AA thresholds used: text < 18.66 px regular ≥ **4.5 : 1**; large text ≥ 3 : 1; UI component
boundaries (control borders, toggle tracks, focus ring) ≥ **3 : 1** (SC 1.4.11). All ratios below were
computed with the WCAG 2.x relative-luminance formula on the exact hex values.

---

## 1. Typeface decision (Vietnamese diacritics)

Test string for every check: `Ấ Ầ Ẩ Ẫ Ậ ấ ầ ẩ ẫ ậ Ế Ổ Ở Ữ Ỹ Ỷ người Đường thuở khuỷu GÕ TẮT` at 12, 14, 20, 28 px,
line-height 1.2 and 1.5, `lang="vi"`.

| # | Stack | Cost | Vietnamese coverage | Diacritic risk | Verdict |
|---|---|---|---|---|---|
| F1 | `"Segoe UI Variable Text", "Segoe UI Variable", "Segoe UI", system-ui, sans-serif` (display: `"Segoe UI Variable Display"`) | 0 KB (system) | Segoe UI covers full Vietnamese (long-standing Windows UI font for vi-VN). Variable version coverage of all 134 letters `UNVERIFIED` — method: render test string, check no fallback glyphs via DevTools "Rendered Fonts" | Win10 1809 has no Variable → falls back to Segoe UI (fine, slightly different metrics). Hinting at 12–14 px is the best available in WebView2 | **Primary for UI text** |
| F2 | `"Be Vietnam Pro", F1…` (bundled woff2, `vietnamese` + `latin` subset, weights 400/500/600) | ≈ 3 × 25–35 KB `UNVERIFIED` (method: `pyftsubset --unicodes=U+0000-00FF,U+0102-0103,U+0110-0111,U+0128-0129,U+0168-0169,U+01A0-01A1,U+01AF-01B0,U+0300-0323,U+1EA0-1EF9`, measure) — negligible vs P7 10 MB | Designed specifically for Vietnamese; stacked marks are drawn side-by-side Vietnamese style | Wider set (~6–8 % wider than Segoe at same size, `UNVERIFIED`, measure with test string) → long labels like `Sửa lỗi dấu gợi ý (Browsers, Excel,...)` may wrap in 320 px label column | **Display (≥ 20 px) in direction A; primary in B** |
| F3 | `"Inter", F1…` (bundled variable woff2) | ≈ 300 KB full, ≈ 90 KB subset `UNVERIFIED` | Google Fonts ships a `vietnamese` subset for Inter, so all 134 letters exist | Inter is tuned for tight UI line-heights (1.2–1.3); at 1.2 the double marks on capitals (`Ẩ Ẫ`) approach/exceed the line box → clipping in inputs. Whether Inter applies Vietnamese-style mark stacking (`locl` for `vi`) is `UNVERIFIED` — method: render `ẩ ẫ ổ` with `lang="vi"` vs `lang="en"`, diff screenshots | Rejected as primary: adds bytes for no gain over F1 on Windows, and its main strength (tight leading) is exactly what Vietnamese can't use |

Monospace (macro triggers, hotkey keycaps optional, config path): `"Cascadia Mono", Consolas, monospace`
(system on Win11; Consolas covers Vietnamese).

---

## 2. Direction A — "Lặng" (calm sidebar)

**Idea:** Win11-native calm. Left navigation, one centred settings column, soft neutral surfaces, a single
deep-jade accent. Feels like a modern Windows Settings page that loads instantly.

### 2.1 Layout and density arithmetic

```
┌──────────── 980 ────────────────────────────────────────────────────────┐
│ sidebar 220 │ status strip 48 (full content width 760)                  │
│  logo 56    ├───────────────────────────────────────────────────────────┤
│  nav 8×36   │ page header 64 (title 20/28 + subtitle 12/18)             │
│  gap 4      │  ┌─ column 664 (760 − 2×48) ───────────────────────────┐  │
│             │  │ group label 32                                       │  │
│  footer 48  │  │ row 56 ×9                                            │  │
│ (on/off pill│  │                                                      │  │
│  + version) │  └──────────────────────────────────────────────────────┘  │
└─────────────┴───────────────────────────────────────────────────────────┘
```
| Measure | Value |
|---|---|
| Sidebar | 220 wide; padding 12; nav item 196 × 36, icon 16, gap 12, radius 6; active = accent-soft bg + 3 × 16 accent bar at left |
| Content | 760 wide; side padding 48 → column **664** |
| Row | **56** high: label 14/20 + description 12/16, vertical padding 10; control right-aligned |
| Label column | 664 − control slot 240 − gap 24 = **400** |
| Controls | height 32 (inputs, selects, buttons, segmented); toggle 40 × 24 (thumb 16); checkbox 16 |
| Visible rows | 648 − 48 strip − 64 header − 32 group = 504 → **9 rows** per screen without scroll |
| Spacing scale | 4, 8, 12, 16, 24, 32, 48 |

### 2.2 Type scale (F1 body, F2 display)

| Token | px / line-height | Weight | Use |
|---|---|---|---|
| `display` | 28 / 36 (F2) | 600 | dashboard state `Tiếng Việt · Telex` |
| `title` | 20 / 28 (F2) | 600 | page title |
| `heading` | 16 / 24 | 600 | group headers in dashboard |
| `body` | 14 / 20 → 22 for multi-line | 400 | labels, inputs |
| `body-strong` | 14 / 20 | 600 | row labels |
| `caption` | 12 / 18 | 400 | descriptions, help |
| `overline` | 12 / 16, +0.02em | 600 | group labels (sentence case, not ALL CAPS — caps stack marks badly: `GÕ TẮT`) |
| `mono` | 13 / 20 | 400 | macro triggers, paths |

### 2.3 Colour tokens and contrast

| Token | Light | Dark |
|---|---|---|
| `bg` (window, sidebar) | `#F7F7F5` | `#161716` |
| `surface` (content) | `#FFFFFF` | `#1E1F1E` |
| `surface-2` (hover, keycaps, table header) | `#F0F0EE` | `#262826` |
| `divider` (decorative, not a boundary) | `#E3E3E0` | `#2E302E` |
| `control-border` | `#8C8C87` | `#6B6D69` |
| `text` | `#1C1C1A` | `#EDEDEA` |
| `text-2` | `#5C5C58` | `#A9AAA5` |
| `text-3` (placeholder, disabled hint) | `#6E6E6A` | `#8A8B86` |
| `accent` | `#0F6E5A` | `#4FC3A1` |
| `accent-fg` | `#FFFFFF` | `#08241C` |
| `accent-soft` | `#E3F1ED` | `#1B3A32` |
| `danger` / `danger-soft` | `#B42318` / `#FDECEA` | `#FF8A80` / `#3A1F1D` |
| `warning` | `#8A5A00` | `#F2C14E` |
| `success` | `#1E7A3C` | `#6FD08C` |

| Pair | Light | Dark | AA |
|---|---|---|---|
| text / bg | 15.91 | 15.32 | ✓ |
| text / surface | 17.07 | 14.10 | ✓ |
| text / surface-2 (keycaps) | 14.96 | 12.66 | ✓ |
| text-2 / surface | 6.72 | 7.07 | ✓ |
| text-2 / bg | 6.26 | 7.69 | ✓ |
| text-2 / surface-2 | 5.89 | 6.35 | ✓ |
| text-3 / surface | 5.12 | 4.82 | ✓ |
| text-3 / bg | 4.77 | 5.24 | ✓ |
| text-3 / surface-2 | 4.49 | 4.33 | ✗ → **rule: text-3 never on surface-2** (use text-2) |
| accent (links, active label) / surface | 6.18 | 7.60 | ✓ |
| accent / bg | 5.76 | 8.26 | ✓ |
| accent-fg / accent (primary button) | 6.18 | 7.54 | ✓ |
| accent / accent-soft (active nav label) | 5.32 | 5.69 | ✓ |
| text / accent-soft | 14.69 | 10.54 | ✓ |
| danger / surface | 6.57 | 7.24 | ✓ |
| danger / danger-soft | 5.75 | 6.60 | ✓ |
| warning / surface | 5.93 | 9.85 | ✓ |
| success / surface | 5.38 | 8.72 | ✓ |
| control-border / surface (non-text, 3 : 1) | 3.38 | 3.16 | ✓ |
| divider / surface | 1.29 | — | decorative only; never the sole boundary of a control |

### 2.4 Shape, elevation, motion
Radii: controls 6, cards/popovers 10, keycaps 4, toggles full. Surfaces flat (no shadow); elevation
only for popovers/menus/toasts: `0 1px 2px rgb(0 0 0 / .08), 0 8px 24px rgb(0 0 0 / .12)` (dark: `/ .40`,
`/ .50`). Motion: state change 120 ms `cubic-bezier(.2,0,0,1)`; popover in 160 ms (opacity + 4 px
translate), out 100 ms; page switch 0 ms content swap + 120 ms fade of the column; toggle thumb 140 ms.

### 2.5 Hotkey row (Phím tắt)
Row 56: label/description left (400). Right slot 240: capture field 168 × 32, radius 6,
1 px control-border, shows keycaps (`Ctrl` `Shift`: each 22 high, padding 0 8, radius 4, surface-2,
12/16 600, gap 4, separator `+` in text-2). Then preset ▾ (32 × 32 ghost) and reset ↺ (32 × 32 ghost;
visible only when value ≠ default or row hovered/focused). Capturing: 2 px accent border, accent-soft
fill, text `Nhấn tổ hợp phím…` in accent, 6 px dot pulsing 1 s. Pending: keycaps + ✓/✕ 24 × 24 inside the
field. Conflict: row grows to 76, caption line in danger `Trùng với "Đổi kiểu gõ"` + text button `Hoán đổi`.
Warning (app shortcut): amber 16 px icon with tooltip, no row growth.

### 2.6 Macro table (Gõ tắt)
Toolbar 48: search 280 × 32 (icon left, `Ctrl+F` hint right), spacer, `Thêm` (primary), `Nhập…`, `Xuất…`
(ghost). Sticky header 32 on surface-2 (caption 12/16 600, text-2). Rows **40**: checkbox col 40,
trigger col 168 (mono 13/20), expansion flex (body, single-line ellipsis, full text in tooltip), actions
col 72 (edit/delete icons, visible on hover/focus/selection). Inline add row pinned under header,
accent-soft fill. Selection: accent-soft row; bulk bar replaces toolbar (`3 mục đã chọn · Xoá · Bỏ chọn`).
Visible: 648 − 48 strip − 64 header − 48 toolbar − 32 head = 456 → **11 rows**; virtualized beyond.

---

## 3. Direction B — "Giấy" (paper, top tabs, editorial)

**Idea:** warm, quiet, document-like. Top tab bar, narrow reading column, hairline dividers instead of
cards, generous rows. The most "calm"; the least dense.

### 3.1 Layout and density arithmetic
```
┌──────────────────────────── 980 ───────────────────────────────────────┐
│ header 56: wordmark 120 │ tabs (8, underline style) │ on/off pill 112  │
├────────────────────────────────────────────────────────────────────────┤
│ status band 40 (centred, text-2: "Tiếng Việt · Telex · Unicode")       │
├────────────────────────────────────────────────────────────────────────┤
│             column 600, centred (190 margins)                          │
│             page header 56 · rows 64 · hairline dividers               │
└────────────────────────────────────────────────────────────────────────┘
```
| Measure | Value |
|---|---|
| Header | 56; tabs short labels (`Tổng quan, Gõ, Bảng mã, Gõ tắt, Loại trừ, Phím tắt, Hệ thống, Giới thiệu`), 14/20, padding 0 12, active = 2 px accent underline + text weight 600. Tab strip ≈ 8 × 84 = 672 `UNVERIFIED` (depends on F2 width; measure) — fits 980 − 120 − 112 − 2×24 = 700 |
| Column | **600** (≈ 70 characters at 14 px — reading measure) |
| Row | **64**: label 15/22 + description 13/20, padding 11 |
| Label column | 600 − 200 − 24 = **376** |
| Controls | height 32; toggle 44 × 24 |
| Visible rows | 648 − 56 − 40 − 56 = 496 → **7 rows** |
| Spacing | 4, 8, 12, 16, 24, 40, 64 |

### 3.2 Type scale (F2 Be Vietnam Pro throughout)
`display 32/40 600`, `title 22/30 600`, `heading 17/26 600`, `body 15/22 400`, `caption 13/20 400`,
`overline 12/16 600`, `mono 13/20`. Larger base (15) suits Be Vietnam Pro's wider proportions and the
editorial feel.

### 3.3 Colour tokens and contrast

| Token | Light | Dark |
|---|---|---|
| `bg` | `#FBF8F3` | `#1A1714` |
| `surface` | `#FFFFFF` | `#23201C` |
| `surface-2` | `#F3EEE6` | `#2C2823` |
| `divider` | `#E8E1D6` | `#3A352E` |
| `control-border` | `#948B7E` | `#756C60` |
| `text` | `#22201C` | `#F1ECE4` |
| `text-2` | `#6B645A` | `#B3AA9C` |
| `text-3` | `#716A5E` | `#958C7F` |
| `accent` (lacquer red) | `#A93D17` | `#F08A5D` |
| `accent-fg` | `#FFFFFF` | `#2A1206` |
| `accent-soft` | `#F7E6DD` | `#3D2619` |
| `danger` | `#B42318` | `#FF8F80` |
| `warning` | `#855400` | `#E9B949` |
| `success` | `#2F6B2F` | `#8BCB7A` |

| Pair | Light | Dark | AA |
|---|---|---|---|
| text / bg | 15.35 | 15.18 | ✓ |
| text / surface | 16.26 | 13.79 | ✓ |
| text / surface-2 | 14.08 | 12.45 | ✓ |
| text-2 / surface | 5.84 | 7.06 | ✓ |
| text-2 / bg | 5.51 | 7.77 | ✓ |
| text-2 / surface-2 | 5.06 | 6.37 | ✓ |
| text-3 / surface | 5.35 | 4.89 | ✓ |
| text-3 / bg | 5.05 | 5.38 | ✓ |
| text-3 / surface-2 | 4.63 | 4.42 | light ✓ / dark ✗ → same rule as A |
| accent / surface | 6.24 | 6.56 | ✓ |
| accent / bg | 5.89 | 7.22 | ✓ |
| accent-fg / accent | 6.24 | 7.16 | ✓ |
| accent / accent-soft | 5.14 | 5.70 | ✓ |
| danger / surface | 6.57 | 7.33 | ✓ |
| warning / surface | 6.43 | 8.88 | ✓ |
| success / surface | 6.43 | 8.43 | ✓ |
| control-border / surface | 3.36 | 3.14 | ✓ |

Risk: red accent next to `danger` red — danger must always pair with an icon + text, never colour alone;
`danger` and `accent` differ in hue by ~10° only. This is B's main weakness.

### 3.4 Shape, elevation, motion
Radii 4 (controls), 6 (popovers), keycaps 3. No shadows at all; popovers use 1 px divider border +
`0 12px 32px rgb(40 30 20 / .10)`. Motion 160 ms ease-out; tab underline slides 200 ms.

### 3.5 Hotkey row
Row 64. Capture field is borderless text-like until hover (underline 1 px control-border), keycaps
rendered as small caps text with thin borders (radius 3). Capturing: underline becomes 2 px accent, text
`Nhấn tổ hợp phím…` italic-free (Vietnamese italics are harder to read at 13 px). Presets as a text link
`Gợi ý ▾` under the field; reset as text link `Mặc định`. Conflict line in caption under the row.

### 3.6 Macro table
No visible table chrome: rows 44 separated by hairlines; trigger in mono, `→` glyph, expansion in body.
Search is a large 40 px borderless field at the top of the column (`Tìm gõ tắt…`). Add = an always-present
blank last row. Column 600 limits expansion width → long expansions wrap to 2 lines (row 64). Visible ≈ 9 rows.

---

## 4. Direction C — "Bảng điều khiển" (hybrid: rail + content + live inspector)

**Idea:** power-user dashboard. Narrow icon rail, dense content column, and a persistent right
inspector that always shows live state and a "Thử gõ" test field — state is never more than a glance away
on any screen. `Ctrl+K` command palette to jump to any setting by name.

### 4.1 Layout and density arithmetic
```
┌──────────────────────────────── 980 ─────────────────────────────────┐
│rail│ header 56 (title + Ctrl+K search 240)      │ inspector 280      │
│ 64 ├────────────────────────────────────────────┤  status card 136   │
│ 8× │ column 588 (636 − 2×24)                    │  thử gõ 160        │
│ 40 │ group 32 · rows 48 ×11                     │  app hiện tại 96   │
│    │                                            │  phím tắt 120      │
└────┴────────────────────────────────────────────┴────────────────────┘
```
| Measure | Value |
|---|---|
| Rail | 64; icon buttons 40 × 40, radius 10, tooltip labels (labels hidden → **discoverability cost**) |
| Content | 980 − 64 − 280 = 636; padding 24 → column **588** |
| Row | **48**: label 13/20 + inline description in tooltip/caption only when needed |
| Label column | 588 − 200 − 16 = **372** |
| Controls | height 28 (compact), toggle 36 × 20 |
| Visible rows | 648 − 56 − 32 = 560 → **11 rows** |
| Inspector | 280; cards 248 wide, padding 16, gap 12 |

### 4.2 Type scale (F1)
`display 24/32 600`, `title 18/26 600`, `body 13/20 400`, `body-strong 13/20 600`, `caption 12/16`,
`mono 12/18`. Smallest base of the three: 13 px with Vietnamese stacked marks is the readability floor —
must be checked with the test string at 100 % scale.

### 4.3 Colour tokens and contrast

| Token | Light | Dark |
|---|---|---|
| `bg` | `#F4F6FA` | `#0F1218` |
| `surface` | `#FFFFFF` | `#171B23` |
| `surface-2` | `#EEF1F6` | `#1F2430` |
| `divider` | `#E2E6EE` | `#2A3040` |
| `control-border` | `#8A93A3` | `#5E6879` |
| `text` | `#111827` | `#E6E9EF` |
| `text-2` | `#4B5563` | `#9AA3B2` |
| `text-3` | `#646B78` | `#808A9A` |
| `accent` (indigo) | `#3451C7` | `#8DA2FB` |
| `accent-fg` | `#FFFFFF` | `#0B1020` |
| `accent-soft` | `#E7ECFD` | `#222B4A` |
| `danger` | `#C0262D` | `#FF8A8A` |
| `warning` | `#8A5300` | `#F0BE4C` |
| `success` | `#1D7A46` | `#6BD69A` |

| Pair | Light | Dark | AA |
|---|---|---|---|
| text / bg | 16.40 | 15.42 | ✓ |
| text / surface | 17.74 | 14.18 | ✓ |
| text / surface-2 | 15.67 | 12.76 | ✓ |
| text-2 / surface | 7.56 | 6.78 | ✓ |
| text-2 / bg | 6.98 | 7.37 | ✓ |
| text-2 / surface-2 | 6.68 | 6.10 | ✓ |
| text-3 / surface | 5.36 | 4.94 | ✓ |
| text-3 / bg | 4.96 | 5.37 | ✓ |
| text-3 / surface-2 | 4.74 | 4.45 | light ✓ / dark ✗ → same rule |
| accent / surface | 6.67 | 7.14 | ✓ |
| accent / bg | 6.17 | 7.76 | ✓ |
| accent-fg / accent | 6.67 | 7.84 | ✓ |
| accent / accent-soft | 5.66 | 5.75 | ✓ |
| danger / surface | 5.90 | 7.60 | ✓ |
| warning / surface | 6.33 | 10.00 | ✓ |
| success / surface | 5.35 | 9.62 | ✓ |
| control-border / surface | 3.10 | 3.07 | ✓ (thin margin — no lighter variant allowed) |

### 4.4 Shape, elevation, motion
Radii 8 (controls), 12 (cards), keycaps 6. Cards on bg use `0 1px 2px rgb(16 24 40 / .06)`; popovers
`0 12px 32px rgb(16 24 40 / .16)`. Motion 140 ms; toggle uses a 220 ms spring-like
`cubic-bezier(.34,1.56,.64,1)` (disabled under reduced motion). Command palette opens in 120 ms.

### 4.5 Hotkey row
Row 48. Capture field 160 × 28 with keycaps 20 high; preset ▾ fused to the field as a split button;
reset appears inline. The inspector's "Phím tắt" card mirrors the three most important hotkeys live, so
a change is visible in two places. Conflict: tooltip-anchored error + inline caption (row → 68).

### 4.6 Macro table
Dense data grid: rows 32, header 28, columns checkbox 32 / trigger 140 mono / expansion flex / usage hint.
Inspector switches to an editor for the selected macro (trigger + multi-line expansion + preview) —
no inline editing in the grid. Visible ≈ 15 rows.

---

## 5. Comparison

| Criterion | A "Lặng" | B "Giấy" | C "Bảng điều khiển" |
|---|---|---|---|
| Layout | sidebar 220 + column 664 | top tabs + column 600 | rail 64 + column 588 + inspector 280 |
| Rows visible (settings) | 9 @ 56 | 7 @ 64 | 11 @ 48 |
| Macro rows visible | 11 @ 40 | ~9 @ 44 | ~15 @ 32 |
| Base text | 14 px | 15 px | 13 px |
| Vietnamese diacritic safety | high (14/20, F1+F2) | highest (15/22, F2) | lowest (13/20) — needs test |
| State always visible (owner req. 2) | status strip + sidebar footer pill | status band | strip + inspector (strongest) |
| Hotkey ease (owner req. 1) | clear field + preset + reset buttons | link-style, subtler, slower to discover | split button, compact |
| Discoverability of screens | labelled nav, always visible | labelled tabs | icons only (tooltips) ✗ |
| Calm / low friction | high | highest | medium (busier) |
| Familiarity for Windows users | Win11 Settings-like | web/editorial | pro-tool |
| Contrast (all text pairs AA) | ✓ with text-3 rule | ✓ with text-3 rule; accent≈danger hue ✗ | ✓ with text-3 rule; borders at 3.07–3.10 |
| Font payload | ~60–100 KB (F2 display only) | ~100 KB | 0 KB |
| P4 (≤ 90 MB window) risk | low | low | medium (inspector + palette + live preview always mounted) |
| Build effort (Svelte, hand CSS) | medium | low | high |

## 6. Recommendation

**Direction A "Lặng"**, with two borrowings:
1. From C: the **"Thử gõ" test field and current-app line live in the status strip popover** (one click from
   any screen), satisfying owner requirement 2 without a permanent 280 px inspector.
2. From B: **15/22 body for descriptions on the dashboard** only, if the diacritic test shows 12 px captions
   are cramped.

Rationale: best balance of calm and density (9 rows), labelled navigation (no hidden icons), the clearest
hotkey capture affordance (visible field + preset + reset — owner requirement 1), Win11-native feel with
zero font cost for body text, and every text pair passes AA. B is calmer but slower for a settings tool
with 8 screens and has an accent/danger collision; C is the most powerful but the busiest, has the weakest
diacritic margin at 13 px, and costs the most memory and effort.

**Open questions for the owner**
- Accent hue: jade `#0F6E5A` (A) vs indigo vs lacquer red — the whole palette swaps by token.
- Be Vietnam Pro for titles (adds ~60–100 KB) or pure Segoe UI Variable?
- Is a light/dark/system theme toggle enough, or should the window follow Win11 Mica (translucent)? Mica
  in WebView2 needs a transparent window and costs GPU memory — `UNVERIFIED` impact on P4.

**Next step (plan 0.5):** offline HTML preview of the chosen direction: Tổng quan, Phím tắt (with live
capture-field prototype), Gõ tắt (1 000 sample rows), in light and dark, plus the diacritic test page for
F1/F2/F3 screenshots.
