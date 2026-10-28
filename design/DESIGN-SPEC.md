# VietKB — Design spec, v3 (quick-settings window)

Source of truth for pixels. Screens: `design/light/{go,macro,hotkey,system}.html` and the same in
`design/dark/` — **720 × 560**, inline CSS, no JS, no assets. Gallery: `design/index.html` (type specimen +
8 frames at true size). Generator: `design/_src/build.py` (layout/markup) + `design/_src/kit.py` (tokens,
icons, keycap/checkbox helpers). Contrast: `python design/_src/contrast.py` → `_src/contrast-report.md`.
Behaviour: `docs/evkey-clone-work/20-spec.md` §C, as narrowed by charter C9/C10 (720 × 560, ≤ 4 tabs).

---

## 0. The product in one paragraph, then the idea

VietKB's window is a **quick-settings panel**, not an application. Someone opens it to flip one typing
option and close it again. So the landing tab is the typing options, every tab fits without scrolling,
and anything long (the full macro table, the charset converter, the app-rule editor) opens as its own
window later instead of growing this one.

1. **Vở và mực tím.** Light = notebook paper (warm neutrals, hue ≈ 40°) with one accent, the purple school
   ink; dark = the ink (cool violet-grey, hue ≈ 255°) with the same pigment glowing.
2. **Three voices.** What you press = keycap (Segoe UI Small / Cascadia Mono). What you get = **Sitka**.
   Everything else = Segoe UI Variable.
3. **Phím trong giếng** — a raised keycap in a recessed well — is the signature and the only gradient.

Decisions:
- Frameless (`decorations: false`, `resizable: false`, `maximizable: false`). The 40 px strip is the
  caption: mark, tabs, state, and **two** caption buttons (minimise, close — no maximise in a fixed window),
  46 × 40, 1 px 10 × 10 glyphs matching the OS.
- **State is always visible** (20-spec C.0): on tab Gõ in the status row (where it is editable); on the
  other three tabs as a compact chip in the strip, `● Tiếng Việt · Telex · Unicode`, which toggles on click.
- Tone style default **kiểu mới** (`hoà, thuý`). Kiểm tra chính tả is off in the mock, which is why
  `Tự động khôi phục` shows its disabled state.
- Mock e-mail is `ha.nth@dongduong.com.vn` (no diacritics in a local part). No uppercase anywhere (§7).

---

## 1. Layout — 720 × 560, 4 px grid, no scrolling

```
x: 0  24                                                     696 720
y: 0  ┌ strip 40: mark · tabs (Gõ · Gõ tắt · Phím tắt · Hệ thống) · state chip · – × ┐
  40  ├ 1 px hairline (line-2) — sheet starts, surface colour ─────────────────────────┤
  41  │ tab content, 24 px side padding → 672 px column                               │
 544  │ last content edge, 16 px bottom padding                                      │
 560  └──────────────────────────────────────────────────────────────────────────────┘
```

| Element | Value | Why |
|---|---|---|
| Window | 720 × 560, fixed | charter C9 |
| Strip | 40 | tabs 28 high centred (6 + 28 + 6); plane colour, is the drag region |
| Sheet | 520 incl. 1 px top hairline | surface colour; 24 side / 16 bottom padding → **672 × 503** usable |
| Status row (Gõ) | 40 incl. 1 px bottom line | toggle 36 × 20, selects 30 |
| Setting row | **46** = 4 + 20 (label 14/20) + 18 (note 12/18) + 4 | the one rhythm of the product |
| Short row | 40 | single-line rows (exclusion list) |
| Disclosure row | 40 | |
| Thử gõ strip | 36 | one line of Sitka 16/24 |
| Toolbar (Gõ tắt) | 36 holding 32 controls | |
| Table header / bulk bar | 36 | |
| Controls | 32 buttons, inputs, segmented, capture well · 30 status selects · 28 small buttons, tabs | |
| Keycap | 22 in wells / 20 inline | |

### 1.1 Vertical budgets (each tab adds up to ≤ 560; nothing scrolls)

**Gõ (`go.html`)** — the landing tab

| y from | + | Item |
|---|---|---|
| 0 | 40 | strip |
| 40 | 1 | sheet hairline |
| 41 | 40 | status row (on/off · `Ctrl` `Shift` · Kiểu gõ · Bảng mã) |
| 81 | 8 | gap |
| 89 | 368 | 8 setting rows × 46 |
| 457 | 40 | `Nâng cao · 8 tuỳ chọn` disclosure (collapsed) |
| 497 | 11 | flexible gap (`margin-top: auto` on the strip) |
| 508 | 36 | Thử gõ strip |
| 544 | 16 | bottom padding |
| **560** | | = 40 + 1 + 40 + 8 + 368 + 40 + 11 + 36 + 16 |

**Gõ tắt (`macro.html`)**: 40 strip + 1 + 16 + 36 toolbar + 12 + table (1 + 36 bulk/header + 8 × 46 + 1 =
406) + 10 + 18 footnote = **540**, 20 px spare at the bottom. 8 rows, never a partial row.

**Phím tắt (`hotkey.html`)**: 40 + 1 + 14 + 20 intro + 39 group head + 3 × 46 rows + 34 presets + 39 group
head + 66 (row with conflict line) + 2 × 46 = **483**; footer (41) is pinned to the bottom: 503–544, + 16
padding = **560**. 20 px of air above the footer.

**Hệ thống (`system.html`)**: 40 + 1 + 12 + Khởi động 2 × 46 + 12 + Giao diện 3 × 46 + 12 + Cập nhật 2 × 46
+ 12 + Loại trừ 3 × 40 = **531**, ≈ 29 px spare.

Measured on the 720 × 560 renders (last non-background pixel): Gõ 546 (the Thử gõ field is 508–543, its
3 px focus halo reaches 546), Gõ tắt 537, Phím tắt 539, Hệ thống 526. No tab scrolls; no row is cut.

### 1.2 Horizontal

Tab Gõ rows: label column flexible, control column right-aligned at x = 696. Phím tắt: label · 236 px capture
well · 28 px slot. Gõ tắt table: 40 checkbox · 112 trigger · flexible expansion · 68 actions. Hệ thống:
**hanging section labels** 120 + 16 gutter + 536 rows — headings cost no height.

## 2. Type (720 × 560 ramp — fewer sizes than v2)

| Role | Size/LH | Weight | Family | Why |
|---|---|---|---|---|
| Setting label | 14/20 | 500 | Segoe UI Variable Text | the thing scanned; weight, not colour, makes it primary |
| Explanation | 12/18 | 400 | Segoe UI Variable Small | one line each; Small optical size keeps marks sturdy at 12 |
| Tabs, buttons, selects, status label | 13/20 | 500 (status label 600) | Text | |
| Group head (Phím tắt, Hệ thống) | 13/20 | 600 | Text | sentence case, no caps |
| Produced text (Thử gõ) | 16/24 | 400 | **Sitka Text** | output voice |
| Tone examples in pills | 15/20 | 400 | Sitka Text | output voice inside a control |
| Trigger keycaps, inline key examples | 12.5/16, 12/18 | 500 / 400 | Cascadia Mono | what you press |
| Keycap legend | 12/16 (11.5 small) | 600 | Segoe UI Variable Small | |

No display sizes remain: a quick-settings panel has no page titles — the active tab *is* the title.
Tabular numerals on 148, 8, 6, 14 MB, 0.1.0.

## 3. Colour

### 3.1 Neutrals

Light — paper, hue ≈ 38–42°, saturation 8–14 %. Pure grey on a warm screen reads dirty; a warm cast reads
as paper and makes the violet accent look like ink on it.

| Step | Hex | Token / use |
|---|---|---|
| N0 | `#FFFFFF` | `raised`, `float` — controls, table, popover |
| N1 | `#FAF9F6` | `surface` — content plane |
| N2 | `#F2F0EB` | `plane` — caption strip / window ground (`hover` `#F2EFE9`, `hover-plane` `#E9E6DF`) |
| N3 | `#EDEAE4` | `sunken` — wells, segmented track |
| N4 | `#ECE8E1` / `#E0DBD2` | `line-1` row dividers / `line-2` control & group borders |
| N5 | `#CBC5B9` | `line-3` dashed empty field, scrollbar thumb |
| N6 | `#8E887E` | `ctl-line` — checkbox/radio border (3.52:1) |
| N7 | `#6B655B` | `t3` — descriptions |
| N8 | `#57524A` | `t2` — secondary |
| N9 | `#1C1A16` | `t1` — primary |

Dark — ink, hue ≈ 250–258°, saturation 8–12 %. Elevation is expressed by *lightness*, not shadow.

| Step | Hex | Token |
|---|---|---|
| D0 | `#0F0E13` | `sunken` |
| D1 | `#111015` | `plane` |
| D2 | `#17161C` | `surface` |
| D3 | `#1F1E25` | `raised` (`hover` `#1D1C23`) |
| D4 | `#26252D` | `float` (E2 surfaces get lighter, not just shadowed) |
| D5 | `#232229` / `#2E2D36` | `line-1` / `line-2` |
| D6 | `#403E4A` | `line-3` |
| D7 | `#77748A` | `ctl-line` |
| D8 | `#8F8C9C` | `t3` |
| D9 | `#ADAAB8` | `t2` |
| D10 | `#EEECF3` | `t1` |

### 3.2 Accent — mực tím (hue ≈ 249°)

| Step | Hex | Use |
|---|---|---|
| 50 | `#F3F1FD` | light `acc-tint` — selected row, capturing row |
| 100 | `#E8E4FB` | light `acc-tint-2` — accent tag, byte chip, status dot halo |
| 200 | `#CFC7F6` | light `acc-line` — highlighted picker item |
| 300 | `#B4A8F4` | **dark** `acc-text` |
| 500 | `#6B57D8` | **dark** `acc` fill (toggle, primary, check) |
| 600 | `#5844C8` | **light** `acc` fill |
| 700 | `#4C38BC` / `#4B38B4` | light `acc-text` / light pressed |
| dark tints | `#28224A` · `#342C5E` · `#5A4D9E` | dark `acc-tint` · `acc-tint-2` · `acc-line` |
| 400 | `#8E7BEA` | **dark** `focus` and `rail` (light uses 600 for both) |
| halo | `rgba(88,68,200,.16)` / `rgba(150,130,245,.26)` | focus/capture halo (`acc-a`) |

Dark parity rule: the selected-row tint must differ from the table surface by the **same luminance step as
in light** (1.12:1 — `#F3F1FD` on `#FFFFFF`, `#28224A` on `#1F1E25`), and everything drawn *on* the tint
(rail, focus ring) uses the lighter `#8E7BEA`, because the fill accent `#6B57D8` only reaches 2.93:1 against
the dark tint. Fill accent stays `#6B57D8` so white knobs/ticks keep 5.28:1.

### 3.3 Semantic

| | Light text · tint · line | Dark text · tint · line |
|---|---|---|
| ok (jade) | `#1B7A4F` · `#E6F3EC` | `#5DCB9B` · `#13261D` |
| warn (nghệ / turmeric) | `#8A5200` · `#FCF3DF` · `#E8D2A0` | `#E9B34F` · `#251E11` · `#4A3A1B` |
| bad (sơn son lacquer) | `#B42318` · `#FCEDEB` · `#F0C8C2` | `#FF8B7D` · `#2A1615` · `#5A2C28` |

Semantic colour appears only where state is semantic: live hook dot, `Đã lưu`, recording chip, conflict,
admin/danger. Never decorative.

### 3.4 Contrast (WCAG 2.x, computed by `_src/contrast.py` on the exact hex values)

**Light — text** (AA ≥ 4.5)

| Fg | Bg | Use | Ratio |
|---|---|---|---|
| t1 `#1C1A16` | surface `#FAF9F6` | labels, titles | 16.50 |
| t1 | plane `#F2F0EB` | selected nav | 15.26 |
| t1 | raised `#FFFFFF` | table, controls, popover | 17.37 |
| t1 | key-bot `#F6F4F0` | keycap legend | 15.82 |
| t1 | acc-tint `#F3F1FD` | selected / capturing row | 15.56 |
| t1 | bad-tint `#FCEDEB` | admin notice | 15.27 |
| t1 | hover `#F2EFE9` | hovered row | 15.14 |
| t2 `#57524A` | surface | subtitles, strip chips | 7.36 |
| t2 | plane | nav items | 6.80 |
| t2 | raised | table header | 7.75 |
| t2 | sunken `#EDEAE4` | segmented, off item | 6.45 |
| t2 | warn-tint `#FCF3DF` | warning detail | 7.02 |
| t3 `#6B655B` | surface | descriptions | 5.48 |
| t3 | plane | nav counts, privacy line | 5.07 |
| t3 | raised | paths, placeholders | 5.77 |
| t3 | hover | description on hovered row | 5.03 |
| t3 | acc-tint | description on capturing row | 5.17 |
| t3 | sunken | text inside wells | 4.81 |
| acc-text `#4C38BC` | surface / plane | links, selected nav icon | 7.67 / 7.09 |
| acc-text | acc-tint / acc-tint-2 | picker exe, accent tag, byte chip | 7.24 / 6.51 |
| white | acc `#5844C8` | primary button, checkbox tick | 6.83 |
| ok `#1B7A4F` | surface / raised | live status, Đã lưu | 5.06 / 5.33 |
| warn `#8A5200` | warn-tint / surface | recording chip, conflict | 5.79 / 6.07 |
| bad `#B42318` | raised / surface | danger button, icon | 6.57 / 6.24 |

**Dark — text**

| Fg | Bg | Use | Ratio |
|---|---|---|---|
| t1 `#EEECF3` | surface `#17161C` | labels, titles | 15.35 |
| t1 | plane `#111015` | selected nav | 16.17 |
| t1 | raised `#1F1E25` | table, controls | 14.11 |
| t1 | float `#26252D` | popover, toast, keycap | 12.94 |
| t1 | acc-tint `#28224A` | selected row | 12.64 |
| t1 | bad-tint `#2A1615` | admin notice | 14.65 |
| t2 `#ADAAB8` | surface / plane / raised | secondary | 7.89 / 8.31 / 7.26 |
| t2 | sunken `#0F0E13` / warn-tint `#251E11` | segmented / warning detail | 8.44 / 7.24 |
| t3 `#8F8C9C` | surface / plane / raised | descriptions, counts, paths | 5.48 / 5.77 / 5.04 |
| t3 | hover / acc-tint / sunken | hovered, capturing, wells | 5.15 / 4.51 / 5.86 |
| t3 | float | popover meta | 4.62 |
| acc-text `#B4A8F4` | surface / plane | links, nav icon | 8.43 / 8.88 |
| acc-text | acc-tint / acc-tint-2 | picker exe, tag, byte chip | 6.95 / 5.90 |
| white | acc `#6B57D8` | primary button, tick | 5.28 |
| ok `#5DCB9B` | surface / raised | live, Đã lưu | 8.97 / 8.25 |
| warn `#E9B34F` | warn-tint / surface | recording, conflict | 8.66 / 9.44 |
| bad `#FF8B7D` | raised / surface | danger | 7.27 / 7.91 |

**Non-text (SC 1.4.11, ≥ 3:1)** — full table in `_src/contrast-report.md` (regenerate with
`python design/_src/contrast.py`), zero failures:

| Boundary | Light | Dark |
|---|---|---|
| Toggle **off** — 1 px `ctl-line` outline vs surface | **3.34** | **3.98** |
| Toggle off outline vs hovered row | 3.06 | 3.74 |
| Toggle on track vs surface / raised | 6.48 / 6.83 | 3.41 / 3.13 |
| Focus ring vs surface / raised (Thử gõ) | 6.48 / 6.83 | 5.28 / 4.86 |
| Selected-row rail vs its tint | 6.12 | 4.35 |
| Checkbox / radio border | 3.52 | 3.66 |
| Danger toggle on | 6.24 | 7.91 |

Round 1 shipped the off track at 1.46 / 1.56 with no outline; fixed: `.tg` now carries
`box-shadow: inset 0 0 0 1px var(--ctl-line)` and the track fill was lightened (`#E2DDD4` / `#2A2931`) so the
outline, not the fill, reads as the edge. On-state drops the outline (accent fill is its own boundary).
`t4` (`#A49E92` / `#5F5C6A`) is used **only** for disabled text and the decorative `+` between keycaps —
exempt from 1.4.3. Placeholders use t3.
Forced colours (`@media (forced-colors: active)`): keycaps `border: 1px solid ButtonText`, toggles on =
`Highlight`, selected rows `outline: 1px solid Highlight`, remove tints.

## 4. Shape, border, elevation, gradient

| Radius | Where |
|---|---|
| 4.5 | checkbox · logo tile (16 grid) |
| 5 / 6 | small keycap / keycap, tags |
| 7 | strip chips, segmented item, small buttons |
| 8 | buttons, selects, inputs, wells, rows, nav items |
| 9 | segmented track (8 + 1 so the inner 7 is concentric at 2 px inset) |
| 10 / 12 | toast, table frame / popover |

Borders are 1 px, drawn as `inset 0 0 0 1px` shadows so they never change box size. Three line strengths:
`line-1` dividers, `line-2` containers/controls, `ctl-line` interactive boundaries that must be 3:1.

Elevation — exactly two levels, plus "flat":

| Level | Recipe (light) | Recipe (dark) | Used by |
|---|---|---|---|
| 0 flat | none; separation by plane tone + 1 px line | same | caption strip, sheet, table frame, wells |
| E1 | `0 1px 1px rgba(41,33,20,.04), 0 1px 2px rgba(41,33,20,.06)` + 1 px `line-2` | `inset 0 1px 0 rgba(255,255,255,.035), 0 1px 2px rgba(0,0,0,.45)` | controls, selected tab chip, segmented thumb |
| E2 | `0 0 0 1px rgba(41,33,20,.07), 0 4px 10px -2px rgba(41,33,20,.08), 0 16px 36px -8px rgba(41,33,20,.20)` | `0 0 0 1px #34333D, inset 0 1px 0 rgba(255,255,255,.05), 0 4px 12px rgba(0,0,0,.35), 0 18px 44px -8px rgba(0,0,0,.65)` on `float` `#26252D` | popover, toast, bulk-action bar, tooltip, menus |

Depth model: the **caption strip is the ground plane** (window colour, no border, no shadow); the **content is a
sheet** one tone lighter, joined by a single hairline — it does not float. Only things that are temporarily
above the work (E2) cast a real shadow. Shadows are tinted with the neutral hue (`41,33,20`), never black,
in light mode.

**The one gradient:** keycap face, `linear-gradient(key-top → key-bot)` = `#FFFFFF → #F6F4F0` /
`#2C2B33 → #26252D`, plus a 2 px inner bottom lip (`key-lip`). It exists to make a key read as convex
against the concave well (`sunken` + `inset 0 1px 1.5px rgba(0,0,0,.06)`). Nowhere else.

## 5. Motion

| What | Duration | Easing | Notes |
|---|---|---|---|
| Hover / pressed background | 80 ms | linear | rows, nav, buttons |
| Toggle knob + track | 160 ms | `cubic-bezier(.2,.8,.2,1)` | |
| Popover / menu enter | 140 ms | same, opacity 0→1 + translateY(4px→0) | exit 100 ms, no movement |
| Toast enter | 200 ms | same, translateY(8px→0) | auto-dismiss after the undo window (5 s; 10 s for reset-all) |
| `Đã lưu` | fade in 120 ms, hold 1.2 s, fade out 300 ms | ease-out | |
| Capture well halo (`pulse`) | 1.6 s loop | same | 3 px → 6 px halo fading to 0; stops the moment a chord completes |
| Caret, recording dot | 1.06 s | `steps(1)` blink | system caret rate |
| Tab switch | none | — | settings are not a story; content swaps instantly |

`prefers-reduced-motion: reduce` → all animations and transitions off; caret and dot stay solid.


## 6. Components (v3)

**Strip / tabs** — tab 28 h, radius 7, padding 0 12, 13/500 t2; selected = `raised` chip + E1 + 1 px
`line-2`, t1 (the same chip v2 used for the selected nav item). Gõ tắt carries its count (12/500 t3,
tabular). `Ctrl+1…4` switch tabs. State chip: 8 px accent dot with 3 px `acc-tint-2` halo, `Tiếng Việt`
600 t1, `· Telex · Unicode` t2.

**Status row** (Gõ only) — toggle 36 × 20 · `Tiếng Việt` 14/600 · hotkey as small keycaps · right-aligned
`Kiểu gõ` (12 t3) + 92 px select · `Bảng mã` + 164 px select. Selects 30 h so the row reads as one line of
state rather than a form.

**Setting row** — 46 h; label 14/20/500 t1 over one-line note 12/18 t3 (ellipsis is a bug: rewrite the
note). Controls right-aligned, 12 gap. Hover: `hover` fill bleeding 12 px with radius 8, dividers hidden
around it. Groups are separated by a `line-2` divider instead of a heading (zero height): Dấu (3 rows) ·
Chính tả và khôi phục (4) · Gõ tắt (1).

**Toggle** — 36 × 20, off = `tg-off` + 1 px `ctl-line` outline (3.34 / 3.98 : 1), on = `acc`, danger
on = `bad`, disabled = opacity .4 with the reason in the note line + an inline accent fix link
(`Cần bật Kiểm tra chính tả · Bật`); the label drops to t3.

**Tone pills** (Kiểu đặt dấu) — one segmented control, two single-line pills
`Kiểu mới · hoà, thuý` / `Kiểu cũ · hòa, thúy`: name 13/500 sans (t3; `acc-text` when selected), a t4 `·`,
example 15/20 Sitka (t2; t1 selected). Track 32 h, pills 28 h. The *name* is what the eye reads; the example
is the proof. Default selected: Kiểu mới.

**Disclosure** (Nâng cao) — 40 h row with `line-2` top: chevron-right 20, `Nâng cao` 14/500, neutral count
tag `8 tuỳ chọn`, a 12 t3 preview of what is inside. Because the panel cannot grow, opening it **replaces
the 8 rows in place** with the advanced rows (same 46 rhythm, `‹ Tuỳ chọn chính` back row at the top);
the status row and Thử gõ strip stay. Contents: f j w z làm phụ âm · gõ tắt khi tắt tiếng Việt · giữ
hoa/thường theo từ tắt · tắt khi bàn phím không phải US · sửa lỗi dấu khi có gợi ý · hỗ trợ ứng dụng Metro ·
gửi qua Clipboard · độ trễ giữa các phím = 8.

**Thử gõ strip** — 36 h, radius 8, one line: `Thử gõ` 12/500 t3 + 1 px divider · Sitka 16/24 output with
caret · `Telex · Unicode` 12 t3 (echo of the current method/charset, so the strip proves them). Unfocused:
`sunken` well (`inset 0 1px 1.5px` shadow + `line-1`) — the signature again. Focused (mock): `raised` +
1 px `focus` + 3 px halo. Never persisted, cleared on close (C5).

**Gõ tắt toolbar** — 36: search (flex, 32 h, `Ctrl F` keycaps) · `Nhập…` · `Xuất…` · primary `Thêm`.
**Table** — 1 px `line-2` frame, radius 10. Header 36 shows column titles; **with a selection it becomes
the bulk bar in place** (`acc-tint`, `acc-line` bottom): indeterminate checkbox in the checkbox column ·
`Đã chọn 1 trong 148` · `Sửa` `Xuất` `Xoá` · `Bỏ chọn Esc`. No floating bar, no footer bar, so no row is
ever covered. Rows 46: checkbox · trigger keycap (mono) · expansion 14/20 (multi-line → `↵ +1 dòng` tag) ·
hover actions (raised 2 px to sit on the trigger legend's optical centre). Selected: `acc-tint` + 2 px
`rail`. Footnote 12/18: `Hiện 8 trong 148 mục · gõ để lọc` + `Mở bảng đầy đủ ›` (separate window).

**Capture well** (Phím tắt) — 236 × 32, `sunken` well, 22 px keycaps, states as v2: capturing (raised,
pulsing `focus` halo, held keys + `+` + caret, `Esc huỷ`, row tinted, ✕ in the slot, strip chip stays),
warning (`warn-line` + `Trùng` tag, explanation as a third line in the row → 66 h), modified (reset icon in
the 28 px slot), empty (dashed `line-3`, `Chưa đặt` / `Đặt phím`), split preset chevron + `Nhả để chuyển`
tag. **Preset chips** (34 h line): `Mẫu cho Bật/tắt:` + four outlined chips with small keycaps —
`Ctrl+Shift mặc định` (selected: tint + `acc-line`), `Alt+Z như UniKey`, ``Ctrl+` ``, `Right Ctrl`.

**Hanging section** (Hệ thống) — 120 px label column: 13/600 t1 aligned to the first row's label line,
optional 12 t3 sub-line and a 12/500 accent link (`6 ứng dụng · Quản lý ›`). Both columns start with the
same `line-2` hairline.

**Exclusion mini-list** — 3 × 40: 24 px monogram tile · app name 13/500 · mode 12 t3 right-aligned
(`Tắt tiếng Việt`, `Bảng mã TCVN3`). Full editing is the separate `Quản lý` window.

Toast, tooltip, popover (E2) keep their v2 anatomy; at 720 × 560 a toast sits 16 px from the right/bottom
over the Thử gõ strip's row, never over a control.

## 7. Vietnamese diacritics — rules

Test string `Ấ Ầ Ẩ Ẫ Ậ Ế Ổ Ở Ữ Ỹ người Đường thuở khuỷu Gõ tắt` is rendered for every ramp size in
`index.html` inside `overflow: hidden` boxes; nothing clips.

1. **Line-height floor.** Notes, captions and anything that may truncate: ≥ 1.5 (12/18). Labels 14/20
   (1.43) are allowed because they are single-line, never inside `overflow: hidden`, and Segoe UI reserves
   ≈ 1.33 em for a capital with a double mark; the 46 px row leaves 4 px above the label line on top of that.
2. **No uppercase anywhere.** `GÕ TẮT` puts breve + acute stacked above the cap height (≈ +0.35 em); in the
   usual 11 px / line-height 1 / tracked eyebrow style the marks collide with the line above, get cut by any
   clipping parent, and `TẮT` and `TẤT` become one blot. Group heads are sentence-case 13/600.
3. **Per size.** 15–16 px Sitka (output) and 14 px labels: default. 12–13 px: Segoe UI Variable **Small**
   at 12, the floor for any Vietnamese string; 11.5 px only for ASCII key names in small keycaps.
   Truncation only on 12/18 lines (ellipsis box = full line box), so `Ẫ` is never shaved.
4. **Features.** `<html lang="vi">`; `font-kerning: normal`; ccmp/mark/mkmk untouched; tabular-nums on
   numbers only; store and render NFC.
5. **Keycaps** 22 px for a 12 px legend (1.83×).

## 8. Screens

| File | Content | Deliberate states |
|---|---|---|
| `go.html` | status row; Kiểu đặt dấu (pills), Bỏ dấu ở cuối từ, Tự sửa ưo thành ươ · Kiểm tra chính tả, Tự động khôi phục, Bảo vệ đường dẫn và mã, Viết hoa sau dấu câu · Gõ tắt (`148 mục ›`); Nâng cao; Thử gõ `Nguyễn Thu Hà ở Đường Nguyễn Văn Linh, Quận 7` | hovered row, disabled row with reason + fix, focused Thử gõ with caret, Kiểu mới selected |
| `macro.html` | toolbar; 8 alphabetical macros; footnote with `Mở bảng đầy đủ` | header-as-bulk-bar, 1 selected (tint + rail), hovered row with actions, multi-line tag |
| `hotkey.html` | 6 actions in 2 groups + presets + footer | capturing `Ctrl + Shift +` caret, conflict (`Ctrl Shift Z` vs Word's Làm lại), modified with reset, unbound |
| `system.html` | Khởi động (Windows, Admin) · Giao diện (âm thanh, ngôn ngữ, chủ đề) · Cập nhật (tự kiểm tra, tệp cấu hình) · Loại trừ ứng dụng (3 + `Quản lý ›`) | Admin on in danger colour with `Khởi động lại ngay`, segmented selections |

Telex for the Thử gõ text (verified per token in v2 and unchanged): `Nguyeenx Thu Haf owr DDuwowngf Nguyeenx
Vawn Linh, Quaanj 7`. In v3 the strip shows only the produced line; the keystroke echo was cut for height.

## 9. What v2 had and why it is gone

| v2 (1040 × 720, 8 screens) | v3 | Why |
|---|---|---|
| 232 px sidebar, 8 destinations | 4 tabs in the 40 px caption strip | a quick panel needs ≤ 4 places; the sidebar cost 32 % of the width |
| Tổng quan dashboard (instrument card, hero 32 px state, hay dùng) | gone; state = Gõ status row / strip chip | the owner opens the panel to change typing, not to read a dashboard |
| Gõ tiếng Việt, 16 options in a 2-column grid + open Nâng cao | tab Gõ: 8 daily options + collapsed Nâng cao (8) | the 8 that matter daily are the priority; the rest are one click away, not on screen |
| Bảng mã screen (6 charsets with mojibake samples + bytes, compat section) | bảng mã select in the status row; compat options moved into Nâng cao | the comparison table is a teaching tool → candidate for the future converter window |
| Gõ tắt: 148-row scrolling table, docked bulk footer, empty-state screen | 8-row filtered view, header turns into the bulk bar, `Mở bảng đầy đủ ›` | a 5 000-row table cannot live in a 720 × 560 panel; it gets its own window (C10) |
| Loại trừ ứng dụng screen with running-app picker popover | 3-row list inside Hệ thống + `Quản lý ›` | rules are set once; the editor becomes a separate window |
| Phím tắt with 7 rows | 6 rows + preset chips | `Bật/tắt gõ tắt` hotkey moved to the full list in the macro window |
| Hệ thống: notifications, V/E overlay, tray click action, export/import | Khởi động, Giao diện (âm thanh, ngôn ngữ, chủ đề), Cập nhật, Loại trừ | kept what a user changes; tray/overlay/export defaults are fine and live in `config.toml` |
| Page titles 26 px, subtitles, keystroke echo under Thử gõ, toast/popover screens | removed | every 4 px counts at 560 px; the tab name is the title |

v2 HTML was removed from `design/light|dark/` as instructed. Note: `design/` is **untracked** in git
(`?? design/`), so git does not hold v2. The v2 generator is kept as `_src/build_v2.py`, and it
reproduces all 16 v2 screens; delete it to drop v2 for good. `design/_shots/` (v2 renders, not made by this
pass) is also stale.

## 10. Wording (kept from round 2, still in force)

| Was | Now | Why |
|---|---|---|
| `nhấn–nhả` | `Nhả để chuyển` | natural; the chord fires on release |
| `Trả lại đúng các phím đã gõ cho từ vừa xong` | `Đưa từ vừa gõ về đúng các phím đã bấm` | sentence, not calque |
| bare `hòa, thúy` / `hoà, thuý` | `Kiểu cũ · hòa, thúy` / `Kiểu mới · hoà, thuý` | named choices |
| `Âm thanh và giao diện` (section) | `Giao diện` | fits the 120 px label column on one line |
