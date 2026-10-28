# VietKB — Design spec, v4 (EVKey/UniKey-simple)

**v4 supersedes the v3 layout (§0–§1, §6 screen anatomy, §8).** The owner rejected v3 as "quá khó dùng và
quá phức tạp". Colour (§3), contrast (§3.4), shape/elevation (§4), motion (§5), the diacritic rules (§7) and
the control anatomy (toggle, select, keycap well, checkbox) still apply. The v3 sections are kept below as
history. Screens: `design/{light,dark}/{quick,hotkeys,advanced,tray}.html`; tray preview `design/tray.png`;
gallery `design/index.html`. Generator `_src/build.py` (v3 kept as `_src/build_v3.py`); audit `_src/measure.py`.

## v4.1 What we learned from EVKey and UniKey

- **One screen.** EVKey has three dropdowns and 18 checkboxes on one dialog; UniKey has two dropdowns and
  about 6 checkboxes. Nobody has to look for a setting, because everything is on the one screen.
- **The tray menu is the daily surface.** Users switch on/off, kiểu gõ and bảng mã from the tray menu and
  almost never open the window.
- **`Đóng` and `Thoát` sit side by side.** Every EVKey/UniKey user knows the difference: close the window
  versus quit the program.
- **Weakness we do not copy:** 18 unlabelled checkboxes with equal weight. v4 puts only the 4 options that
  change typing for a normal user in the main window. The rest goes in `Nâng cao`, laid out as EVKey's own
  checkbox grid, so EVKey users recognise it at once.

## v4.2 Windows and arithmetic (4 px grid)

All windows share: frameless, 36 px caption (16 px Móc mark, title 13/600, drag area = everything between
the title and the buttons, minimise + close 44 × 36 with visible hover: `hover-plane`, and close turns `bad`
red with a white glyph). Body = `surface` sheet with 20 px side padding, and one pinned 56 px footer
(1 px `line-2` top, controls 32 high).

**Main window `quick.html` — 480 × 384** (UniKey-sized; the 560 × 420 ceiling is not needed)

| y | + | Item |
|---|---|---|
| 0 | 36 | caption `VietKB` |
| 36 | 1 + 4 | sheet hairline + top air |
| 41 | 44 | `Bật tiếng Việt` + `Ctrl + Shift` as plain t3 text · toggle |
| 85 | 44 | `Kiểu gõ` [Telex] · `Bảng mã` [Unicode dựng sẵn] (labels 13 t2, selects 30 high) |
| 129 | 9 | 4 + 1 px `line-2` + 4 |
| 138 | 4 × 44 = 176 | `Đặt dấu kiểu mới` (hint `hòa, thúy` in Sitka 15) · `Kiểm tra chính tả` · `Cho phép gõ tắt` [`Bảng gõ tắt…`] · `Loại trừ ứng dụng` (hint `6 ứng dụng`) [`Quản lý…`] |
| 314 | 14 | flexible gap |
| 328 | 56 | footer: `Phím tắt…` `Nâng cao…` (quiet links) · `Thoát VietKB` (ghost) · `Đóng` (primary) |
| **384** | | = 36 + 5 + 44 + 44 + 9 + 176 + 14 + 56 |

**12 interactive controls** (the 2 caption buttons not counted): 1 toggle + 2 selects + 3 toggles + 2 buttons
+ 4 footer controls. **2 hints** (`hòa, thúy`, `6 ứng dụng`); no description lines. Rows 44, one column,
labels left, every control ends on x = 460.

**`hotkeys.html` — 520 × 460**: 36 caption + 1 + 44 intro (`Bấm vào ô, nhấn tổ hợp mới. Esc để huỷ,
Backspace để xoá.`) + 7 × 44 rows (6 actions with 248 px keycap wells + `Kiểu chuyển bật/tắt` segmented)
= 389, 15 flexible, footer 56 (`Đặt lại tất cả` link · `Đóng`) = **460**. No per-row descriptions. The mock
shows one capturing row (`Đổi bảng mã`), one warning (`Dễ trùng`) and one unbound row.

**`advanced.html` — 560 × 496**: 36 caption + 1 + three EVKey-style sections. Each heading is 33 px
(13/600, padding 10/2, `line-2` underline). Checkboxes are 16 px in a 2-column grid of 28 px rows, plus 36 px
select rows.
`Gõ` 33 + 4 × 28 = 145 · `Tương thích` 33 + 2 × 28 + 36 = 125 · `Hệ thống` 33 + 2 × 28 + 36 = 125.
Sum: 37 + 145 + 125 + 125 = 432, then 8 flexible + 56 footer (`Đặt lại tất cả…` · `Đóng`) = **496**.
Denser and deliberately one step away from the main window.

**`tray.html` / `tray.png` — menu at the clock**: 236 px menu (E2, radius 10, 30 px items, 13/20).
Items: `✓ Bật tiếng Việt  Ctrl+Shift` · ─ · `Kiểu gõ ▸` (submenu, ● Telex / VNI / VIQR) · `Bảng mã ▸`
(the 6 verified charsets) · ─ · `Mở cài đặt…` · `Phím tắt…` · ─ · `Thoát VietKB`. The tray icon stays `V` / `E`.

Audit (`measure.py`, light + dark): zero text overflow and zero elements outside the window in all 8 files.

**Component rule — rows:** controls sharing a row share one height and one centre line; a mixed row (select
+ number input, button + toggle) is never aligned by nudging a single element. Implementation: one rule,
`.sel, .numf, .btn, .seg, .well, .lk { height: 32px }` (no per-element heights); rows are `align-items:
center` (`.r` flex; `.sr` grid `auto auto 1fr auto auto`). Labels use the same 20 px line box as the control
text, so they share its baseline. Toggles (24) and checkboxes (16) sit on the same centre line.
`measure.py` reports any row whose control or label centres differ by more than 0.6 px, or whose box
controls differ in height.

## v4.3 What moved where

| v3 control | New home | Why |
|---|---|---|
| Status row on/off + hotkey | main row 1 (+ tray item 1) | the one thing everyone needs; the hotkey shown as text teaches it |
| Kiểu gõ, Bảng mã selects | main row 2 (+ tray submenus) | daily switching lives in the tray; the window keeps them for first setup |
| Kiểu đặt dấu pills | main toggle `Đặt dấu kiểu mới` with Sitka hint `hòa, thúy` | EVKey's own wording; one toggle instead of a 2-way segmented control |
| Kiểm tra chính tả | main | changes what every word looks like |
| Gõ tắt toggle + `148 mục` | main `Cho phép gõ tắt` + `Bảng gõ tắt…` | EVKey's pattern: toggle + button to the table |
| Gõ tắt tab (search, table, bulk, import/export) | `Bảng gõ tắt` window (existing C10 window) | a table is not a setting |
| Loại trừ list (3 rows) | main row `Loại trừ ứng dụng · 6 ứng dụng` + `Quản lý…` → app-rules dialog | count + one button; editing is rare |
| Gõ dấu ở cuối từ, Tự sửa ưo thành ươ, Viết hoa đầu câu, Giữ nguyên email…, f j w z, Tự động khôi phục (still disabled while spell check is off), Gõ tắt cả khi tắt tiếng Việt, Viết hoa theo từ tắt | `Nâng cao` → Gõ | defaults are right for almost everyone |
| Metro, Clipboard, Tắt khi đổi bàn phím khác, Sửa lỗi dấu gợi ý, Độ trễ | `Nâng cao` → Tương thích | only for broken apps; deliberately hard to reach |
| Hệ thống tab: Khởi động cùng Windows, quyền Admin, Âm thanh, Cập nhật, Ngôn ngữ hiển thị, Màu nền | `Nâng cao` → Hệ thống | set once (autostart is on by default) |
| Phím tắt tab (6 wells, presets, footer) | `Phím tắt` window (footer link + tray) | changed once in a lifetime; presets dropped |
| Kiểu chuyển bật/tắt (was in Nâng cao) | `Phím tắt` window, last row | it describes how the hotkey fires |
| Đặt lại tất cả | footer link of `Phím tắt` (hotkeys only) and `Nâng cao` (everything; per 27-decisions #10) | scope is stated by where it sits |
| Thử gõ strip, state chip, tab strip, hotkey presets, `Tệp cài đặt` path + `Mở thư mục` | removed | test in any app; state is in the tray icon; the config file is for power users who know where to look |

## v4.7 Dark theme — Windows 11 Settings layering (supersedes the §3.1 dark ramp and the dark half of §3.4)

The owner said the old dark ramp (`#111015 / #17161C / #1F1E25`) was "quá tối". The new ramp copies Windows 11
Settings: neutral greys, and layers are separated by lightness, not shadow. The light theme is unchanged.

| Token | Hex | Role |
|---|---|---|
| `plane` | `#202020` | caption strip, desktop (= Settings page background) |
| `surface` | `#272727` | window sheet (= Settings card) |
| `raised` | `#333333` | selects, buttons, keycap face, toggle-off track (= Settings control) |
| `hover` / `hover-plane` | `#3A3A3A` / `#2D2D2D` | hovered control / hovered caption button |
| `float` | `#2C2C2C` | menus, tray menu (+ 1 px `#3A3A3A` + one real shadow: flyouts are E2) |
| `sunken` | `#1C1C1C` | keycap wells, segmented track |
| `line-1` · `line-2` · `line-3` | `#333333` · `#3A3A3A` · `#4A4A4A` | row dividers · section dividers, control edges · dashed empty well |
| `t1` · `t2` · `t3` · `t4` | `#FFFFFF` · `#C5C5C5` · `#A8A8A8` · `#8A8A8A` | primary · secondary · hints · disabled |
| `ctl-line` | `#8A8A8A` | checkbox and toggle-off border |
| `acc` (fill) · `acc-hover` | `#8B7BFF` · `#9C8CFF` | toggle on, checked box, primary button |
| `on-acc` · `knob-on` | `#1C1C1A` | text, tick and knob on the accent fill (Windows dark style: dark ink on a light accent) |
| `acc-text` · `focus` | `#B9AEFF` | links, tray check, focus ring |
| `acc-tint` · `acc-tint-2` · `acc-line` | `#3A3458` · `#463E6B` · `#6A5FB0` | capturing row, tray highlight |
| `ok` · `warn` · `bad` | `#6CCB9F` · `#E9B34F` · `#FF99A4` | semantic (tints `#1F3A2E` · `#3A3020` · `#442726`) |
| E1 | `inset 0 1px 0 rgba(255,255,255,.05)` | a hairline highlight; no drop shadows on controls |

**Accent decision:** `#6B57D8` is only 2.83 : 1 against `#272727` and 2.39 against `#333333`, so it fails
the 3 : 1 boundary rule on Windows-grey surfaces. The dark fill is now **`#8B7BFF`** (4.53 against the sheet,
3.84 against controls). White on it would be 3.29, so fills carry dark ink `#1C1C1A` (5.18 : 1). This is
what Windows 11 does with its light accent in dark mode. Close-button hover is Windows' own `#C42B1C`
with white, in both themes.

**Contrast, dark (WCAG 2.x, measured on the exact hex values):**

| Text | On | Use | Ratio |
|---|---|---|---|
| t1 `#FFFFFF` | plane / surface / raised / float | titles, labels, control text, menu | 16.29 / 14.94 / 12.63 / 13.97 |
| t1 | hover / hover-plane / acc-tint / key-bot | hovered items, capturing row, keycap legend | 11.37 / 13.77 / 11.59 / 12.63 |
| t2 `#C5C5C5` | plane / surface / sunken | caption icons, inline labels, segmented off | 9.44 / 8.65 / 9.87 |
| t3 `#A8A8A8` | plane / surface / raised / sunken / float | sub-title, hints, chevron, well placeholder, tray shortcut | 6.85 / 6.28 / 5.31 / 7.17 / 5.87 |
| acc-text `#B9AEFF` | surface / float | `Đặt phím`, tray check | 7.51 / 7.02 |
| on-acc `#1C1C1A` | acc `#8B7BFF` | `Đóng`, tick, toggle knob — **lowest text pair** | **5.18** |
| warn `#E9B34F` | warn-tint `#3A3020` | `Dễ trùng` tag | 6.79 |

| Boundary (≥ 3 : 1) | On | Ratio |
|---|---|---|
| acc `#8B7BFF` (toggle on, checked) | surface / raised / float | 4.53 / 3.84 / 4.24 |
| ctl-line `#8A8A8A` (checkbox, toggle off) | surface / raised / hover | 4.33 / 3.66 / **3.29** (lowest) |
| focus `#B9AEFF` | surface / raised | 7.51 / 6.35 |

## v4.8 Theme control (title strip)

- **Position:** in the caption strip, immediately left of minimise: a fixed 24 × 24 slot, 6 px gap to the
  44 px caption buttons, in all three windows. The slot never changes size, so minimise and close never move.
- **States, cycled on click and on `Enter` / `Space`:** `Theo Windows` → `Sáng` → `Tối` → `Theo Windows`.
  The icon shows the *current* state, as 16 px strokes at 1.5: a half-filled circle (Theo Windows), a sun
  (Sáng), a moon (Tối). Tooltip and aria-label read `Màu nền: <state>` (`role="button"`, `tabindex="0"`).
  Colour is `t2`; hover uses `hover-plane` + `t1`, radius 6; focus is the standard 2 px ring.
- **Behaviour:** writes `system.theme` = `system | light | dark`. Every open window re-themes at once (no
  transition). The `Màu nền` select in `Nâng cao` shows the same value. In the mocks the light files show
  `Sáng` and the dark files `Tối`; the hover state is shown on `hotkeys.html`.
- Nothing was dropped from the strip: mark · title · drag area · theme · minimise · close all fit at
  480 px wide.

## v4.5 Logo — four concepts (owner picks)

Files: `design/logo/{A,B,C,D}/` (`logo.svg`, `logo-dark.svg`, `tile.svg`, `tile-dark.svg`, `tray-on.svg`,
`tray-off.svg`, `tray-off-dark.svg`, PNG renders 16/20/32/64/128/256 and tray 16/20). Contact sheet:
`design/logo/index.html`, rendered as `design/logo/sheet.png`. Generator: `design/logo/_build.py`.
All glyphs are drawn with round-capped strokes on a 24 grid. Masters use stroke 2. The tile and tray use
stroke 2.5, and the tile scales the glyph to 75 %, centred, on a radius-6 ink tile.

**Tray states, the same rule for every concept:** *on* = solid rounded tile with the glyph knocked out in
white; *off* = the bare glyph in neutral ink (`#3A362F` on a light taskbar, `#DCD9E3` on a dark one),
no tile. A filled square versus an open line drawing reads at 16 px without colour.

| | Concept | Idea | 16 px verdict |
|---|---|---|---|
| A | Móc | dấu hỏi above a baseline | survives as a shape, but reads as a **question mark / help icon**. Rejected for meaning. |
| B | Ơ | the letter ơ, unique to Vietnamese | **survives**: circle + short horn stays an ơ at 16 px in both states. The horn was shortened to avoid the ♂ sign. |
| C | Đ | lowercase đ with its crossbar | survives; bowl and stem merge slightly at 16 px in the tile |
| D | Con trỏ ngã | text caret carrying a ngã | **fails at 16 px**: the tilde collapses into a bar and the mark reads as `T`/`I` |

Designer's recommendation: **B (Ơ)**. It is the one letter only Vietnamese writes, so it says "Vietnamese
typing" without a word. It stays legible from 256 px down to the 16 px tray in both states, and it replaces
the current Móc mark, which reads as "help".

## v4.14 Logo v8 — master + optical size (standing rule)

**Two-size rule:** every mark we ship has (1) a **master on a 512-unit grid**, used at ≥ 64 px, with designed
terminals, a clean join and optically even strokes; and (2) a **separately drawn 16 px optical size**
(rendered at 16 / 20 / 32). The small size has heavier relative strokes, no corner radii or angled cuts,
and every edge on whole pixels. A 16 px asset scaled up is never a logo. `design/logo8/` holds the first
application (d3 / d4); v9 below supersedes its colours.

## v4.17 Logo v11 — perspective keycap (supersedes the v10 tile anatomy; colours and letters unchanged)

Files: `design/logo11/keycap/` (`v|e-master.svg` 512, `_v|e-reduced.svg`, `_v|e-minimal.svg`, `v|e-16.svg`,
PNGs `v|e-512/256/192/128/96/64/48/32/20/16`, `_notes.md`). Proof sheet: `design/logo11/index.html` rendered
as `sheet.png` (1600 wide). Generator `design/logo11/_build.py`.

| Tile | Face | Bottom band | Band step | Left hairline (lit) | Right wall (darkest) | Bevel | Rim |
|---|---|---|---|---|---|---|---|
| V | `#FFEBDD` | `#F2BE98` | `#F7CEAF` | `#EFB68C` | `#E7A579` | `#FFFAF6` | `#C94810` |
| E | `#E0F1FD` | `#A4CDF0` | `#B8DAF5` | `#97C5EE` | `#88BBE9` | `#F7FBFF` | `#1E72D2` |

Only the front face moved, one step darker than v10's lip, plus new wall shades. Face, rim and letters are
unchanged, so every ratio holds: rims 4.28 / 4.31 on `#F3F3F3` and 3.43 / 3.41 on `#202020`; letters pink
4.56, blue 4.53, orange 3.65 on the face.

**Rule (v11c, standing): the face is square; the offset belongs to the body.** This supersedes the v11b
tapered, shifted face, which stopped reading as a keycap.
- **Face:** a perfect square, 416 × 416 units, corner radius 96 (23 %), with no taper, skew or shift. The letter
  is centred on it by its bounding box. It must read as a square at every size; measured at 512 it is
  416 × 416 = **1.000**.
- **Body:** the same rounded-square construction behind the face, larger, offset down and right. Overhangs are
  left 2.5, right 30, bottom 40, top **0** (the face's top edge is the key's top edge). The body's top-left
  corner is the face's own arc, and its other corners lie on the silhouette.
- **Strips:** only strips of the body may show — a lit left hairline, the right wall (darkest) and the bottom
  band with one lighter step in its top 28 %. Diagonals from the face's corner 45° points split the walls from
  the band. No body corner sits inside the silhouette, and no inner outline is drawn. The rim is the outer half
  of a stroke behind both fills, so it traces only the union.
- **Bevel:** 4 units on the face's top edge and upper-left corner, fading to 1.5 at the upper-right.
- **Light and shadow:** light from above-left; the shadow sits under the body, +20 right and +10 down.

**Size ladder (geometry in 512 units):**

| Step | Sizes | Face | Body overhang L / R / B / T | Bevel | Shadow | Dropped |
|---|---|---|---|---|---|---|
| full | 192–512 | 416 square, r 96 | 2.5 / 30 / 40 / 0 | 4 → 1.5 at UR | blur 9, 28 %, +20 / +10 | — |
| reduced | 96–128 | 416 square, r 96 | 1.5 / 18 / 24 / 0 (60 %) | 8 → 4 | 14 %, +12 / +6 | — |
| minimal | 48–64 | 416 square, r 96 | none | none | none | body, bevel, shadow |
| flat | 16 / 20 / 32 | 15 px square, r 3.5 | none | none | none | everything except face, rim, letter |

**Letter centring:** the glyph's bounding box is centred on the face (±2 units at 512).

**Flat rule (16 / 20 / 32):** one rounded square, one face colour, 1 px rim, the letter — **nothing else**: no
front band, no side shade, no bevel, no shadow. Letter rows 3–12, centred (V heavy 2.6 → 2.3 px, thin 2 px; E 2 px
strokes and counters). Palette: V face `#FFEBDD`, rim `#C94810`, letter `#CF1A65` + `#CF5605`; E face `#E0F1FD`,
rim `#1E72D2`, letter `#1C6CC6`. Tile edge: rim 4.28 / 4.31 on `#F3F3F3`, 3.43 / 3.41 on `#202020`; on `#808080`
the rim is 1.20 and the face carries the edge at 3.42. Letters on face: 4.56 / 3.65 / 4.53.

## v4.16 Logo v10 — retro keycap (supersedes the transparent v9 mark)

Files: `design/logo10/keycap/` (`v|e-master.svg` 512, `v|e-16.svg`, `_v|e-flat.svg` used for 48–128,
PNGs `v|e-512/256/128/96/64/48/32/20/16`, `_notes.md`). Proof sheet: `design/logo10/index.html` rendered as
`sheet.png` (1600 wide). Generator `design/logo10/_build.py`. The letter construction is the same as v9
(512 master + separately drawn 16 px).

| Tile | Face | Lip | Highlight | Rim | Letter |
|---|---|---|---|---|---|
| V (bật) | `#FFEBDD` | `#F5C5A3` | `#FFF8F3` | `#C94810` deep orange | pink `#CF1A65` heavy + orange `#CF5605` thin |
| E (tắt) | `#E0F1FD` | `#A7D0F2` | `#F4FAFE` | `#1E72D2` deep blue | blue `#1C6CC6` |

The lip is the face's hue two steps darker. There is no gradient anywhere; the only soft element is the
contact shadow below the tile.

**Contrast.** Rims: V 4.28 on `#F3F3F3` / 3.43 on `#202020`, E 4.31 / 3.41, so both clear 3 : 1 and one
tile serves both taskbars (**no dark variant needed**). Letters on their face: pink 4.56, blue 4.53,
orange **3.65**. The orange is the weakest pair, kept above 3 : 1 but below 4.5: darkening it further makes
it the same rust as the rim. On mid-grey `#808080` the rims fail (1.2), but the light faces are 3.42 : 1, so
the opaque tile separates where the transparent mark could not.

**Keycap anatomy — rule: one outer silhouette; interior divided at 82–84 % by a horizontal line; no inner
rounded corners; no offset copies.** This replaces two failed attempts: two offset outlines ("two cards"),
then a face and a base as two overlapping rounded rectangles ("two sheets of paper").
1. The key is **one rounded rectangle**: 456 × 462 units, radius 88, at y 14.
2. Its interior is split by a straight horizontal edge at **y 398 (83 % of the height)**. The face is above
   it. Below it is the **front lip**: 78 units, which is 64 px at 420 and 29 px at 192. The lip fills to the
   outer contour through a clip, so its inner corners are square. It carries one lighter tonal step (`#F8D2B6`
   on V, `#B8D9F4` on E) in its top 26 units, which is not a gradient.
3. The 8-unit rim strokes **only the outer silhouette**. The 4-unit highlight runs along the top edge only.
   The soft shadow sits under the whole tile.

**Reductions by size.**
- 512 / 256: full keycap per the rule above (one silhouette, divided interior, outer rim, highlight, shadow).
- 48–128: flat keycap: the lip, highlight and shadow are dropped, and the rim is thickened to 12 units.
- 16 / 20 / 32: a redrawn whole-pixel tile: radius 3.5, **rim 1 px** (a scaled rim would be 0.3 px). The
  letter is redrawn in a 10 px field: V heavy 2.7 → 2.3 px, thin 2.2 px; E strokes and counters 2 px.

## v4.15 Logo v9 — owner palette (final hex values)

Files: `design/logo9/yellow-orange/` (`v|e-master(-dark).svg`, `v|e-16(-dark).svg`, PNGs
`v|e-512/256/128/96/64` from the master, `v|e-32/20/16` from the 16 px drawing, `-dark` sets, `_notes.md`).
Proof sheet: `design/logo9/index.html` rendered as `sheet.png` (1600 × 1360). Generator `design/logo9/_build.py`.
V = bật (heavy stroke **vàng**, thin stroke **cam**); E = tắt (solid **xanh trời nhạt**). Transparent.

| Background | Heavy (vàng) | Thin (cam) | E (xanh trời) |
|---|---|---|---|
| Dark (`#202020` taskbar, `#272727` window) — owner's colours | `#FACC15` 10.64 / 9.75 | `#FB923C` 7.20 / 6.60 | `#7DD3FC` 9.77 / 8.96 |
| Light (`#F3F3F3` taskbar, `#F7F7F5` window) — **compromise** | `#AD8400` 3.11 / 3.22 | `#CF5605` 3.80 / 3.93 | `#1192D3` 3.11 / 3.22 |

The owner's colours fail on light backgrounds (`#FACC15` 1.38, `#7DD3FC` 1.50, `#FB923C` 2.04 on `#F3F3F3`).
They are darkened along the same hue only as far as 3 : 1 requires. The cost: on light, vàng reads as
**dark gold / mustard**, not yellow, and xanh trời nhạt reads as a **medium sky blue**, not light; cam stays
orange. On mid-grey `#808080` no colour of either set reaches 3 : 1. A transparent mark assumes a light or a
dark taskbar.

**Geometry.** Master: heavy stroke 113 → 95 units, thin about 71 units, every terminal cut at the same ~11°
rise with 8–14-unit radii. The thin stroke is drawn under the heavy one and ends exactly at the heavy foot
corner: no cap and no gap. E: one path, a 104-unit stem, 72-unit bars and equal 100-unit counters. Both
letters are optically centred. 16 px: 13 px tall on whole pixels, heavy 3.55 → 3.2 px, thin 2.4 px, V
interior 2.0 px at mid-height; E has a 3 px stem and bars with 2 px counters.

**Risk, reported not fixed:** yellow and orange are the conventional caution hues, so a yellow-orange tray
glyph can be read as a warning badge. Mitigations: it is a letter V rather than a triangle or exclamation
mark, it is the default "on" state seen all day, and the off state E is in a different hue family (blue).

## v4.13 Logo v7 — duotone, bolder

Files: `design/logo7/d1…d5/`: `v.svg`, `v-dark.svg`, `e.svg`, `e-dark.svg`, PNGs `v|e-16/20/32` plus the
`-dark` sets, `titlebar-20.png` / `titlebar-20-dark.png`, and `notes.md` with the full table. The `_old*`
files are the v6 thin construction in the same colours, kept for the weight-comparison row. Contact sheet
`design/logo7/index.html` rendered as `sheet.png` (1100 × 1800): a mid-grey `#808080` composite, a v6-vs-v7
weight row, then one row per palette. Generator `_build.py`.

**Construction at 16 px** (vector, so the ratio holds at 20 / 32 / 128):
- Heavy downstroke: a polygon 3.9 → 3.4 px wide horizontally, which is **3.6 → 3.2 px** measured
  perpendicular (v6: ~2.9 → 2.1).
- Thin upstroke: a **2.4 px** round-capped line (v6: 1.9). It is drawn *under* the heavy stroke, so the join
  is the heavy stroke's clean edge: no gap, no colour smear.
- The V's interior stays open: **2.5 px** at mid-height, 4.2 px at y = 6.
- E: a 3.2 px stem plus three 2.4 px drawn bars, leaving 3 px counters. E is solid in hue B.
- Colour rule: in the light-taskbar variant the heavy stroke sits at relative luminance ≈ 0.045 and the
  thin stroke at 0.17–0.22. In the dark-taskbar variant the same hues are lightened, heavy ≈ 0.45 and thin
  0.30–0.42. One variant cannot serve both taskbars (see §v4.12).

| | Pairing | Light: heavy · thin/E | Dark: heavy · thin/E | Lowest at 16 px | Notes |
|---|---|---|---|---|---|
| d1 | Lạnh/Ấm — chàm × đỏ son | `#253581` · `#DD281B` | `#A6B0E5` · `#EC6E65` | 4.31 | **E in solid red reads as an error/alert badge** |
| d2 | Rêu/Vàng — rêu × nghệ | `#0F4345` · `#AE7607` | `#2BC6CB` · `#E99F0A` | 3.51 | passes; on the light taskbar the gold leans mustard |
| d3 | Tím/Ngọc — tím × ngọc | `#492684` · `#108265` | `#BFA9E6` · `#15AD87` | 4.30 | passes; the jade E reads calm, not as a warning |
| d4 | Đất/Trời — nâu đất × chàm sáng | `#5D301F` · `#3B6CDF` | `#DBA794` · `#6F93E8` | 4.30 | passes |
| d5 | Mực/Nắng — mực xanh than × cam nắng | `#1F377D` · `#C95C08` | `#A0B2E7` · `#F78936` | 3.77 | passes the numbers, but **the orange E reads as a warning** |

Every stroke clears 3 : 1 on its own taskbar. Every heavy stroke is ≥ 7.7 : 1. On the app's window
surfaces the lowest is 3.63 (d2 thin on `#F7F7F5`). No pairing fails at 16 px on numbers; d1 and d5 fail
on meaning, because their off state looks like an alert.

Designer's recommendation: **d3 Tím/Ngọc**. The heavy stroke keeps the app's mực tím, the jade stroke
makes the two hues unmistakable, and the jade E is bright without looking like an error.

## v4.12 Logo v6 — transparent Ink

Files: `design/logo6/p1…p5/`: `v.svg`, `e.svg`, `v-dark.svg`, `e-dark.svg` (transparent, no tile), PNGs
`v|e-16/20/32` plus the `-dark` sets, `titlebar-20.png` (on `#F7F7F5`) and `titlebar-20-dark.png` (on
`#272727`), and `notes.md` with the full table. Contact sheet `design/logo6/index.html` rendered as
`sheet.png` (1100 × 1700), with a mid-grey `#808080` composite strip at the top. Generator `_build.py`.
The construction is unchanged from logo4/5 C (heavy downstroke + 1.9 px upstroke). There is no black: both
strokes are colour.

**Two-variant rule.** Light-taskbar variant: heavy stroke at relative luminance ≈ 0.05, thin stroke ≈ 0.17
(both ≥ 3 : 1 on `#F3F3F3`, and clearly two tones). Dark-taskbar variant: the same two hues lightened to
heavy ≈ 0.55 and thin ≈ 0.27 (both ≥ 3 : 1 on `#202020`). *Why one variant cannot serve both:* a colour
clears 3 : 1 on both taskbars only for L ≈ 0.14–0.25, which leaves no room for a dark heavy and a light thin
stroke. Crossing variants shows it: the light heavy stroke on the dark taskbar is 1.55 : 1, and the dark
heavy stroke on the light taskbar is 1.57 : 1.

**E (off).** The whole letter is drawn in the thin-stroke accent colour, at the same weight and
construction. V is two-tone and E is one solid accent, so E stays saturated and never reads as disabled.
It clears ≥ 4.28 : 1 on both taskbars in every palette.

| | Palette | Light taskbar (heavy · thin = E) | Dark taskbar (heavy · thin = E) | Lowest ratio (tray / window) |
|---|---|---|---|---|
| p1 | Mực tím (lightness fixed) | `#482A8F` · `#735BE8` | `#CBBDEC` · `#907EED` | 4.31 / 4.46 |
| p2 | Mực xanh ngọc | `#0D4745` · `#0E826B` | `#29D9D3` · `#12A084` | 4.28 / 4.43 |
| p3 | Mực đỏ son | `#791D1A` · `#D93218` | `#EFB6B4` · `#EB624D` | 4.28 / 4.43 |
| p4 | Mực chàm | `#29368C` · `#426BDE` | `#BBC2EB` · `#6A8AE5` | 4.31 / 4.46 |
| p5 | Mực nâu đất | `#643121` · `#B75727` | `#E4BBAE` · `#D77341` | 4.30 / 4.44 |

No palette falls below 3 : 1 at 16 px. Every heavy stroke is ≥ 9.3 : 1 on its own taskbar, and every pair
clears ≥ 4.43 : 1 on the app's window surfaces (`#F7F7F5` / `#272727`). On the `#808080` worst case every
mark keeps its edge; the light variants' thin stroke is the faintest element there, as expected.

Designer's recommendation: **p1 Mực tím**. With the lightness fixed it passes every pair, it keeps the hue
the app already uses, and so the UI accent tokens (`#5844C8` / `#8B7BFF`) stay in the same family without a
brand-colour change.

## v4.11 Logo v5 — Ink palettes (concept C, colour only)

Files: `design/logo5/p1…p6/`: `lockup(-dark).svg`, `tile(-dark).svg`, `tray-v|e(-dark).svg`, PNGs
(`lockup-480`, `tile-128/64/32`, `titlebar-20(-dark)`, `tray-v|e-16/20(-dark)`) and `notes.md` with the full
contrast table. Contact sheet `design/logo5/index.html` rendered as `sheet.png` (1100 × 1900); generator
`_build.py`. The construction is copied unchanged from logo4 C. Ink is `#1C1C1A`, the light tile is paper
`#F7F7F5`, the dark tile is ink with a paper stroke `#F2F2F5`, and the off state is the E in grey
`#6B6B70` / `#A8A8A8` (no accent = off).

**Why the accent lightness is fixed.** The accent stroke must clear 3 : 1 against *both* the ink stroke and
the paper. On paper it must also clear 4.5 : 1 if it is to be the UI text accent. That holds only for
relative luminance ≈ 0.14–0.17 on light and ≈ 0.23–0.26 on dark, so p2–p6 are tuned to L ≈ 0.15 / 0.245.

| | Palette | Light / dark accent | accent : paper / ink | dark : ink / paper | 16 px verdict |
|---|---|---|---|---|---|
| p1 | Mực tím (baseline) | `#5844C8` / `#8B7BFF` | 6.37 / **2.50** | 5.18 / **2.95** | the upstroke merges with the ink at the join; tuned same-hue fix `#6D57D8` / `#8A79E0` |
| p2 | Mực xanh ngọc | `#0E7A6B` / `#119986` | 4.88 / 3.26 | 4.82 / 3.17 | passes, the most distinct from the ink |
| p3 | Mực đỏ son | `#CD2D21` / `#E35C52` | 4.90 / 3.25 | 4.81 / 3.18 | passes; red reads as "error" in UI controls |
| p4 | Mực chàm | `#4863D1` / `#6D83DA` | 4.92 / 3.24 | 4.80 / 3.18 | passes |
| p5 | Mực nâu đất | `#AE502E` / `#CF6C49` | 4.91 / 3.24 | 4.79 / 3.19 | passes; warm, but close to the warn amber |
| p6 | Mực vàng kim | `#8A660B` / `#AE800E` | 4.91 / 3.24 | 4.79 / 3.19 | **fails by eye**: olive "mud" on a light taskbar; a lighter gold saves the mark but cannot be UI text |

**UI consequence.** Any palette other than p1 changes the product accent: `--acc`, `--acc-text`, `--focus`
and `--rail` become the light value, dark `--acc` / `--rail` become the dark value with ink text on fills,
and the tints are re-derived from the same hue. Measured against the app surfaces: every light accent clears
4.5 : 1 as text on `#F7F7F5` and as white button text. Every dark accent clears 3 : 1 on `#202020` and
`#272727` (lowest 4.19) and carries ink text at ≥ 4.79.

Designer's recommendation: **p4 Mực chàm**. It passes every pair, and its hue stays in the cool blue-violet
family of the current UI, so adopting it shifts the accent token rather than changing the brand's
temperature. It also carries a real Vietnamese story (the northern highland indigo dye).

## v4.10 Logo v4 — colour systems (logo first, tray derived)

Files: `design/logo4/{A,B,C,D}/`: `lockup.svg` (mark + drawn `VietKB`), `tile.svg`, `tray-v.svg`,
`tray-e.svg`, `tray-v-dark.svg`, `tray-e-dark.svg`, PNGs (`lockup-480`, `tile-128/64/32`, `titlebar-20`,
`tray-v|e-16`, `tray-v|e-20`, `tray-v|e-16-dark`, `tray-v|e-20-dark`) and `notes.md`. Contact sheet
`design/logo4/index.html` rendered as `sheet.png` (1100 × 1700); generator `_build.py`. No secondary hue was
needed in any concept. Every concept ships a light-taskbar and a dark-taskbar tray variant.
**V = on, E = off** in all four; the letter shape carries the state and colour only reinforces it.

| | Concept | Palette | Tray derivation | 16 px in colour |
|---|---|---|---|---|
| A | Tile — mực tím rounded square, white block V | `#5844C8` tile (light) / `#8B7BFF` tile with ink V (dark) · `#FFFFFF` · off grey `#B4B1BC` / `#55545B` | the tile itself; off = E on a grey tile | **passes**: crisp, solid, the off state is obvious |
| B | Ligature — V cut into a low tile, arms breaking its top edge | `#5844C8` / `#8B7BFF` · off `#8A8794` / `#A6A4AD` | the same mark; off = E cut into a narrow tile, bars poking out | **fails**: at 16 px the V reads as a cup with ears and the E as a comb. Kept for the record, not recommended |
| C | Ink — brush V: heavy ink downstroke, mực tím upstroke | `#1C1C1A` / `#F2F2F5` ink · `#5844C8` / `#8B7BFF` stroke · paper tile `#F7F7F5` | the same two inks; off = brush E in single grey `#6B6B70` / `#A8A8A8` | passes; the 1.9 px upstroke is at the limit |
| D | Monogram — block V + mực tím dấu sắc | `#1C1C1A` / `#F2F2F5` V · `#5844C8` / `#8B7BFF` sắc · ink tile | V + sắc; off = block E with no mark, grey `#57524A` / `#C5C5C5` (rule: no tone mark = off) | passes; the sắc is 0.7 px clear of the V |

Designer's recommendation: **A — Tile**. The tray icon is literally the logo at 16 px, and a white V on a
mực tím tile stays crisp on both taskbars. The grey E tile reads as "off" before you even see the letter.

## v4.9 Logo v3 — V/E tray

Files: `design/logo3/s1…s6/` (`v.svg`, `e.svg`, PNG `v|e-16/20/32` with light-taskbar ink `#1C1C1A`,
`v|e-16/20-dark` with dark-taskbar ink `#F2F2F5`, and `tile-128.png` where the style supports a tile),
contact sheet `design/logo3/index.html` rendered as `sheet.png` (1100 × 1600), generator `_build.py`. Every
glyph is a set of paths on a 16 px grid, about 14 px tall and centred, in one colour. Accent appears only on
the app tile, never in the tray. In every style **on = V, off = E**: a pointed letter with an open top versus
a letter with a vertical stem and three bars. The difference is pure shape, so it does not depend on colour.

| | Style | Construction | 16 px result |
|---|---|---|---|
| 1 | Grotesque | 2 px stems, square cuts, pixel-aligned E bars, 4 px counters | crisp, passes |
| 2 | Rounded humanist | 2.2 px round-capped strokes, round joins | passes; edges a little soft |
| 3 | Monoline outline | one 1.5 px stroke, edges aligned to the pixel grid | readable, but **breaks the ≥ 2 px stem rule by definition** and looks faint; off-state candidate only |
| 4 | Heavy | 3 px block stems, E counters 2 px, V opening 6 px | the most legible, passes |
| 5 | Negative space | V knocked out of a filled rounded square; E solid inside an open 1.5 px square | passes, but busy at 16 |
| 6 | Ink / calligraphic | brush V (heavy left stroke, 1.8 px right stroke that overshoots the join); E with three drawn horizontals | passes the size rules; the softest of the six because its curved bars antialias |

Designer's recommendation: **4 — Heavy**. V and E stay unmistakable when squinting at a real 16 px tray
on both taskbars, and its counters stay open without antialiasing tricks.

## v4.6 Logo v2 — lettering (supersedes the v4.5 glyph concepts, which the owner rejected)

Files: `design/logo2/{A,B,C,D}/`, contact sheet `design/logo2/index.html` rendered as `design/logo2/sheet.png`
(1100 × 1500), generator `design/logo2/_build.py`. Letterforms are drawn paths (monoline, round caps, stroke
3 on a 14 px cap height, no font). Wordmarks use ink `#1C1C1A` / paper `#F7F7F5`, with the Vietnamese
detail in `#5844C8` / `#6B57D8`. Tiles: white lettering on an accent tile with radius 6.

**Tray (owner decision):** `V` = tiếng Việt on, `E` = off. Drawn on a 16 px grid, 2 px stems, round caps,
the same construction as the wordmarks, and open counters (3.5 px between the E bars). Ink on a light
taskbar, paper on a dark one. The state is the letter shape, never the colour. Files per direction:
`tray-v(-dark).svg`, `tray-e(-dark).svg`, PNG 16/20/32 plus dark 16/20/32.

| | Spelling · idea | Compact form (how it derives) | Small-size verdict |
|---|---|---|---|
| A | `VKB` · geometric, V carries a nặng dot | the dotted V, the wordmark's first letter | tile reads at 20–32; in the 16 px tray the dot fuses with the V point into a `Y`, so the tray uses a plain V |
| B | `ViKB` · lowercase i whose dot is a dấu sắc | `Ví`, the wordmark's first two letters | **survives everywhere**: tile `Ví` reads at 20 px |
| C | `Viet` · V with a hook (dấu hỏi) on its left arm | the hooked V, the wordmark's first letter | tile reads at 32; the hook blurs at 16, so the tray uses a plain V |
| D | `VKB` ligature · V's right arm is K's stem, K's arms meet B's stem, đ bar on B | the whole ligature | **fails at 20–32 px**: K disappears, reads as "VB" with a slab |

Designer's recommendation: **B (`ViKB`)**. It is the only direction whose Vietnamese detail (the í)
survives from the 128 px tile down to the 20 px title bar. Its compact form is literally the first two
letters of the wordmark, and its V matches the owner's tray V without any simplification.

## v4.4 Toggle correction for the dev

40 × 24 track, 16 px knob, 4 px inset gives **16 px travel** (40 − 2·4 − 16), not 20. The shipped
`.tg.on::after { transform: translateX(20px) }` pushes the knob flush against the right edge. Fix:
`translateX(16px)`. (§11's "travel 20" was my arithmetic error; the v4 mocks use 16.)

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
- Tone style default **kiểu mới** (`hòa, thúy`). Kiểm tra chính tả is off in the mock, which is why
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
| Status row (Gõ) | 40 incl. 1 px bottom line | toggle 40 × 24, selects 30 |
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
| 457 | 40 | `Nâng cao · 9 tuỳ chọn` disclosure (collapsed) |
| 497 | 11 | flexible gap (`margin-top: auto` on the strip) |
| 508 | 36 | Thử gõ strip |
| 544 | 16 | bottom padding |
| **560** | | = 40 + 1 + 40 + 8 + 368 + 40 + 11 + 36 + 16 |

**Gõ tắt (`macro.html`)**: 40 strip + 1 + 16 + 36 toolbar + 12 + table (1 + 36 bulk/header + 8 × 46 + 1 =
406) + 10 + 18 footnote = **540**, 20 px spare at the bottom. 8 rows, never a partial row.

**Phím tắt (`hotkey.html`)**: 40 + 1 + 14 + 20 intro + 45 group head + 3 × 46 rows + 34 presets + 45 group
head + 64 (row with warning line) + 2 × 46 = **493**; 14 gap; footer (37) 507–544; 16 padding = **560**.
(Round 4 moved the old 20 px of air above the footer into the two group heads, padding 18/6.)

**Hệ thống (`system.html`)**: 40 + 1 + 16 + Khởi động 2 × 46 + 14 + Giao diện 3 × 46 + 14 + Cập nhật 2 × 46
+ 14 + Loại trừ 3 × 40 = **545**; 15 bottom, matching Gõ's 16.

Measured with `_src/measure.py` (injects a script into a copy of each screen, reads `getBoundingClientRect`
through headless Edge `--dump-dom`; light and dark identical): last element bottom Gõ 544 · Gõ tắt 539 ·
Phím tắt 544 · Hệ thống 545. Zero text overflow (no ellipsis, no wrap) in all 8 files. No tab scrolls.

### 1.3 Bottom bands — one rule for all four tabs

**Footers are pinned bands, not content.** Each tab has at most one bottom band, and it always ends exactly
16 px above the window edge (y = 544). Gõ: the Thử gõ strip (508–544). Gõ tắt: the table footnote (526–544).
Phím tắt: `.hkfoot` (507–544, with a 1 px `line-2` divider on top and 8 px above its buttons). Hệ thống has
no band; its last section ends at 545.
- Content above the band is **top-aligned on its fixed rhythm**: 46 rows, 40 short rows, group heads
  `.sh` 18/6, Hệ thống gaps 16/14. These numbers never stretch.
- The surplus collects in **one flexible gap directly above the band** (`margin-top: auto` on the band).
  It is never spread into rows or section gaps. A warning row (+18 px) appears and disappears while the user
  edits a hotkey; if the surplus were spread out, every row below the warning would jump. With a single gap,
  only that gap changes.
- Minimum gap: 10 px (Gõ tắt guarantees it with `.tbl { margin-bottom: 10px }`). The budgets in §1.1 keep
  every tab above that minimum.
- Phím tắt with the spec values: no warning → last row ends at 475, gap **32 px**; warning visible → ends
  493, gap **14 px**. The divider makes either gap read as a deliberate edge. The build measurement
  (last row 951, band 1011, physical px at 125 %) is 60 px physical = **48 CSS px** of gap, not 99. It is
  larger than 32 because the app still uses `.sh` 12/6 (12 px shorter in total). With the §11 defect values
  it drops to 32 px (14 px when a warning shows).

**Alignment rules (round 4, verified by `measure.py`):**
- **One right edge, x = 696**, for every control on every tab: toggles (656–696), selects, segmented,
  buttons, capture wells, toolbar, table frame. A row with two controls puts the secondary one on the left
  (`Kiểm tra ngay` before the toggle) so the toggle stays on the axis.
- **No slot column.** Capture-row actions (✕ huỷ, ↺ đặt lại mặc định, ▾ chọn nhanh) live inside the well as
  24 px icon buttons; wells are 276 wide (420–696).
- Rows without a control leave the axis empty: the Nâng cao preview stops 64 px short of 696.
- Label + description are centred as one block in each 46 px row (measured offset 0.0 on every row).
- The strip state chip has a fixed 236 px slot on every tab (hidden on Gõ), so tabs never move:
  x = 56 / 98.8 / 187.8 / 263.7 on all four tabs.
- The table header (`Từ tắt · Nội dung`) and the bulk bar that replaces it are both 36 px (measured
  106–142), so selecting a row never moves the rows.

### 1.2 Horizontal

Tab Gõ rows: label column flexible, control column right-aligned at x = 696. Phím tắt: label · 276 px capture
well (actions inside). Gõ tắt table: 40 checkbox · 112 trigger · flexible expansion · 68 actions. Hệ thống:
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

**Status row** (Gõ only) — toggle 40 × 24 · `Tiếng Việt` **600 14/16** (the 16 px line box sits level
with the 20 px keycaps; allowed under §7 rule 1 because it is single-line, unclipped and its only marks are
on lowercase letters) · hotkey as small keycaps · right-aligned `Kiểu gõ` + 92 px select · `Bảng mã` + 164 px
select. Sub-labels **400 12/16 `t2`** (7.36 : 1 light, 7.89 : 1 dark), **8 px** to their own select and
**16 px** (8 margin + 8 gap) from the previous select, so each label groups with its control. Selects 30 h so
the row reads as one line of state rather than a form.

**Setting row** — 46 h; label 14/20/500 t1 over one-line note 12/18 t3 (ellipsis is a bug: rewrite the
note). Controls right-aligned, 12 gap. Hover: `hover` fill bleeding 12 px with radius 8, dividers hidden
around it. Groups are separated by a `line-2` divider instead of a heading (zero height): Dấu (3 rows) ·
Chính tả và khôi phục (4) · Gõ tắt (1).

**Toggle** — **40 × 24, radius 12, knob 16 inset 4, travel 16** (was 36 × 20: the larger track is the
better hit target and still fits the 46 px row and the 40 px status row on the 4 px grid). Off = `tg-off` +
1 px `ctl-line` outline (3.34 / 3.98 : 1), on = `acc`, danger on = `bad` with a white knob, disabled = opacity .4 with the reason in the note line + an inline accent fix link
(`Cần bật Kiểm tra chính tả · Bật`); the label drops to t3.

**Tone pills** (Kiểu đặt dấu) — one segmented control, two single-line pills
`Kiểu mới · hòa, thúy` / `Kiểu cũ · hoà, thuý`: name 13/500 sans (t3; `acc-text` when selected), a t4 `·`,
example 15/20 Sitka (t2; t1 selected). Track 32 h, pills 28 h. The *name* is what the eye reads; the example
is the proof. Default selected: Kiểu mới.

**Disclosure** (Nâng cao) — 40 h row with `line-2` top: chevron-right 20, `Nâng cao` 14/500, neutral count
tag `9 tuỳ chọn`, a 12 t3 preview of what is inside. Because the panel cannot grow, opening it **replaces
the 8 rows in place** with the advanced rows (**40 px short rows**, label 14/20 + note 12/18, so that the
back row + 9 options = 400 px fit between the status row and the Thử gõ strip — 9 × 46 would overflow; the
`‹ Tuỳ chọn chính` back row carries the `9 tuỳ chọn` tag on the right);
the status row and Thử gõ strip stay. Contents: f j w z làm phụ âm · gõ tắt khi tắt tiếng Việt · giữ
hoa/thường theo từ tắt · tắt khi bàn phím không phải US · sửa lỗi dấu khi có gợi ý · hỗ trợ ứng dụng Metro ·
gửi qua Clipboard · độ trễ giữa các phím (`0` µs) · **Kiểu chuyển bật/tắt** (segmented `Khi nhả phím` /
`Khi nhấn phím`, default nhả, note `Nhả phím mới chuyển, hoặc đổi ngay khi nhấn.` — moved here in round 5
from the hotkey well) = 9.

**Thử gõ strip** — 36 h, radius 8, one line: `Thử gõ` 12/500 t3 + 1 px divider · Sitka 16/24 output with
caret · `Telex · Unicode` 12 t3 (echo of the current method/charset, so the strip proves them). Unfocused:
`sunken` well (`inset 0 1px 1.5px` shadow + `line-1`) — the signature again. Focused (mock): `raised` +
1 px `focus` + 3 px halo. Never persisted, cleared on close (C5).

**Gõ tắt toolbar** — 36: search (flex, 32 h, `Ctrl F` keycaps) · `Nhập từ tệp…` · `Xuất ra tệp…` · primary `Thêm`.
**Table** — 1 px `line-2` frame, radius 10. Header 36 shows column titles; **with a selection it becomes
the bulk bar in place** (`acc-tint`, `acc-line` bottom): indeterminate checkbox in the checkbox column ·
`Đã chọn 1 trong 148` · `Sửa` `Xuất…` `Xoá` · `Bỏ chọn Esc`. No floating bar, no footer bar, so no row is
ever covered. Rows 46: checkbox · trigger keycap (mono) · expansion 14/20 (multi-line → `↵ +1 dòng` tag) ·
hover actions (raised 2 px to sit on the trigger legend's optical centre). Selected: `acc-tint` + 2 px
`rail`. Footnote 12/18: `Đang hiện 8 trong 148 mục · gõ vào ô tìm để lọc` + `Mở bảng gõ tắt ›` (separate window).

**Capture well** (Phím tắt) — 276 × 32 (x 420–696), `sunken` well, 22 px keycaps. Every action lives
inside the well as a 24 px icon button. States: capturing (raised, pulsing `focus` halo, held keys + `+` +
caret, `Esc huỷ` + ✕ `Huỷ`, row tinted); warning (`warn-line` + `Dễ trùng` tag — `Trùng` is reserved for
blocking conflicts); modified (↺ `Đặt lại mặc định`); empty (dashed `line-3`, `Chưa đặt` / `Đặt phím`);
split `Chọn nhanh` chevron on the toggle row. Round 5: the bare `Nhả để chuyển` tag is gone from the well;
release-vs-press is the labelled option `Kiểu chuyển bật/tắt` (`Khi nhả phím` / `Khi nhấn phím`) in
Nâng cao on tab Gõ.
**Warning row** (`Khôi phục từ gõ sai`) — 64 h = 4 + 20 label + 18 description + 18 warning + 4. The warning
is amber 12/18 with ⚠ inline before the text, starting on the label edge. It runs full width (to 696)
because the verbatim COPY.md string (79 characters ≈ 470 px) does not fit the 380 px description column.
The extra 18 px is the only exception to the 46 rhythm on the tab; the gap above the footer took it (14 px).
**Preset chips** (34 h): `Chọn nhanh:` + `Ctrl+Shift mặc định` (selected: tint + `acc-line`),
`Alt+Z như UniKey`, ``Ctrl+` `` — three, as shipped (`Ctrl phải` dropped: a side-specific lone modifier needs
CCR-9 in the hotkey grammar first).

**Hanging section** (Hệ thống) — 120 px label column: 13/600 t1 aligned to the first row's label line,
optional 12 t3 sub-line and a 12/500 accent link (`6 ứng dụng · Quản lý ›`). Both columns start with the
same `line-2` hairline.

**Exclusion mini-list** — 3 × 40: 24 px monogram tile · app name 13/500 · mode 12 t3 right-aligned
(`Tắt tiếng Việt`, `Bảng mã TCVN3`). Full editing is the separate `Quản lý` window.

**Toast** (E2) — as shipped: grid `icon · text · action · ✕`, padding 10 10 10 12, gap 8, radius 10,
13/20 500, max-width 420, 16 px from the right/bottom. Height grows with the text (error strings may run to
110 characters per COPY.md, so a fixed 44 px one-line toast is not enough). Enter 200 ms, translateY 8 → 0.

**Window-button hover** — minimise: `hover-plane`; close: `bad` fill with white glyph (Windows convention).
Hover states on tabs, chip, selects, buttons, presets and table rows use `hover` / `hover-plane`.
**Disabled button** — opacity .45 + `not-allowed` cursor (disabled controls are exempt from 1.4.3).
**State chip off** — dot turns `t4` with no halo; text `Đã tắt tiếng Việt`.

**App-rules dialog** (`Quản lý`, the C10 "separate window", shipped as an in-window modal) — 552 wide
(≤ viewport − 48), radius 12, `raised`, E2, backdrop `rgb(10 9 13 / .5)`; head 44 (title 14/20 600 + close),
body padding 8 16 12 and scrolls inside itself, foot 56 on `surface` with right-aligned buttons.

Tooltip and popover keep their v2 anatomy.

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
| `go.html` | status row; Kiểu đặt dấu (pills), Gõ dấu ở cuối từ, Tự sửa ưo thành ươ · Kiểm tra chính tả, Tự động khôi phục từ gõ sai, Giữ nguyên email, địa chỉ web, Viết hoa đầu câu · Gõ tắt (`148 mục ›`); Nâng cao; Thử gõ `Nguyễn Thu Hà ở Đường Nguyễn Văn Linh, Quận 7` | hovered row, disabled row with reason + fix, focused Thử gõ with caret, Kiểu mới selected |
| `macro.html` | toolbar; 8 alphabetical macros; footnote with `Mở bảng gõ tắt` | header-as-bulk-bar, 1 selected (tint + rail), hovered row with actions, multi-line tag |
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

## 10. Copy

Round 4: all strings come verbatim from `design/COPY.md`, overridden by
`docs/evkey-clone-work/27-copy-decisions.md`:
- `Tự động khôi phục từ gõ sai` **keeps** its disabled state when Kiểm tra chính tả is off; note
  `Cần bật Kiểm tra chính tả · Bật` (COPY.md #23 rejected).
- Off state `Đã tắt tiếng Việt`, never `English` (chip, tray).
- Typing delay default shows `0` (µs); bảng mã pickers list only the 6 verified charsets
  (no BK HCM1, VPS, UTF-8); `Đổi bảng mã` cycle reads `Unicode → TCVN3 → VNI Windows → Unicode tổ hợp`.
- Reset is always `Đặt lại…` (`Đặt lại tất cả`, `Đặt lại mặc định`); `khôi phục` only for khôi phục từ gõ sai
  / khôi phục bộ gõ.
- Window title `Cài đặt VietKB`; caption tooltips `Thu nhỏ`, `Đóng, VietKB vẫn chạy`.
No string needed shortening: every one fits its row without wrap or ellipsis at 720 px (measured).

Earlier wording decisions still in force:

| Was | Now | Why |
|---|---|---|
| `nhấn–nhả` → `Nhả để chuyển` (tag in the well) | option `Kiểu chuyển bật/tắt`: `Khi nhả phím` / `Khi nhấn phím` in Nâng cao (round 5) | a two-word tag beside keycaps was unreadable out of context; a labelled option says what it controls |
| `Trả lại đúng các phím đã gõ cho từ vừa xong` | `Đưa từ vừa gõ về đúng các phím đã bấm` | sentence, not calque |
| bare `hoà, thuý` / `hòa, thúy` | `Kiểu cũ · hoà, thuý` / `Kiểu mới · hòa, thúy` | named choices |
| `Âm thanh và giao diện` (section) | `Giao diện` | fits the 120 px label column on one line |

## 11. Changelog

- **Reconciliation with the shipped app** (`ui/src/styles/tokens.css`, `kit.css`). Colour tokens: no drift.
  - *App wins* (spec + mocks updated; `_src/build.py` reproduces it): toggle 40 × 24 r12, knob 16 inset 4,
    travel 16 — corrected in §v4.4 (bigger hit target, still on the 4 px grid and inside the 46 / 40 rows); status label
    600 14/16; status sub-labels 400 12/16 `t2` (better contrast than `t3`); Nâng cao drill-in rows 40 px
    (9 × 46 would overflow); toast auto-height grid (110-char errors); three hotkey presets, no
    `Ctrl phải`; `Kiểu chuyển bật/tắt` note; close-button red hover, hover states, disabled opacity .45,
    grey off-dot, app-rules dialog anatomy.
  - *Spec wins* (dev defects, app untouched here): capture well 276 with actions inside (app 236 + 28 slot);
    warning row 64 with full-width amber line (app 66, inline `t2`); `.sh` padding 18/6 (app 12/6);
    Hệ thống gaps 16 / 14 (app 12 / 12); state-chip fixed 236 px slot (app none); Nâng cao preview stops
    64 px before the control axis (app none); 16 px between the Kiểu gõ select and the `Bảng mã` label (app 8).
- **Bottom-band rule (§1.3).** Option chosen: pinned band + one flexible gap above it. Not (a): the footer
  would jump by 18 px every time a hotkey warning shows. Not (b) or (c): spreading the surplus over section
  gaps makes the rhythm depend on how much content there is, and moves rows while the user edits. Mock
  change: the Gõ tắt footnote is now pinned (521–539 → 526–544); Phím tắt, Gõ and Hệ thống are unchanged.
  Dev values, one selector per line:

  | Selector | Property | Value |
  |---|---|---|
  | `.hkfoot` | `margin-top` / `padding-top` / `border-top` | `auto` / `8px` / `1px solid var(--line-2)` (already shipped, keep) |
  | `.sh` | `padding` | `18px 0 6px` |
  | `.row.tall` | `height` | `64px` (the warning `.msg`: `position:absolute; left:12px; right:12px; top:42px; color:var(--warn)`) |
  | `.mfoot` | `margin-top` | `auto` (was `10px`) |
  | `.tbl` | `margin-bottom` | `10px` |
  | `.sheet` | `padding-bottom` | `16px` (already shipped, keep) |
  | `.list`, `.hang`, `.sh` | `flex-grow` | `0`; never use `justify-content: space-between` on `.sheet` |
- **v4 redesign (owner: v3 too complex).** One 480 × 384 main window (12 controls), `Phím tắt` 520 × 460, `Nâng cao` 560 × 496, tray menu. v3 mocks removed from `light|dark` (generator kept as `_src/build_v3.py`). See §v4.1–v4.4.
