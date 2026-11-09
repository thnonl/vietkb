# VietKB settings UI: control-by-control audit (2026-11-09)

Worktree `E:\vietkb-ui-b` (branch `ui-macos-b`). Method: vite dev (mock data) at 480 px wide, DPR 2 in the browser pane, dark + light, `?platform=windows` and `?platform=macos`; `getBoundingClientRect` / `getComputedStyle` per control; keyboard focus via real Tab key; hover via real pointer; WCAG contrast computed from the resolved tokens (light and dark). Real Windows 125 % capture (PrintWindow of the QA build, read-only) in `_audit-shots/real-*.png`.
Severity: **must** = visibly broken or blocks use; **should** = spec/consistency/accessibility miss; **nit** = polish.
Screenshots: `design/macos-style/_audit-shots/` (names given per row). Coordinates are CSS px at 480 wide.

## Summary

| Severity | Count |
|---|---|
| must | 6 |
| should | 21 |
| nit | 13 |

Must-fix: S1 select cap, K1 capture well hit area/layout, F1 no keyboard focus ring on primary/ghost/icon/selected-lang/selected-seg buttons, B1 dark-mode hover/pressed inverted, T1 `--t3` text contrast 2.2-2.7:1 (light/dark), X1 disabled Advanced rows dimmed twice.

Root causes worth knowing first (one fix, many rows):
- `svg.i.s16` (specificity 0,2,1, kit.css bottom) beats `.sel-wrap > .i` (0,2,0), so the cap's `height:24px` is ignored.
- `.btn.pri`, `.btn.ghost`, `.langtg button.on`, `.seg > button.on` set `box-shadow` with higher specificity than the global `button:focus-visible` ring.
- `.btn:hover` / `.sel:hover` use a translucent `--hover-plane` as the whole `background`, which replaces the opaque `--ctl-bg` instead of overlaying it.
- `.capa` (the inner button of CaptureField) has no CSS anywhere.

## 1. Select (pop-up) `Select.svelte` + `.sel`, `.sel-wrap > .i` (kit.css)

| # | Where / state | Expected | Actual (measured) | Sev | Fix |
|---|---|---|---|---|---|
| S1 | Chung "Kiểu gõ", AppRulesDialog rows, every state (`_audit-shots/real-select-zoom6x.png`, `real-chung-now.png`, `web-chung-dark.jpg`) | Cap = full height of the control (24), 16 wide, glyph readable (SPEC 6, mockup `.pop .ud` 22/24 tall x 16, padding 2) | Cap is 16 x **16**, top-aligned (y 131..147 on a 131..155 select), the 24 px height is overridden by `svg.i.s16`. Inner box 12 x 10 after `padding:3px 2px`; the `updown` path (6 x 9.5 units of a 20-unit box) renders **3 x 4.8 px**: unreadable. Same on macOS. | must | Replace the svg-as-cap with a wrapper: `<span class="sel-cap"><Icon name="updown" small/></span>`; `.sel-cap{position:absolute;top:0;right:0;bottom:0;width:16px;display:grid;place-items:center;background:var(--acc);color:var(--on-acc);border-radius:0 6px 6px 0;pointer-events:none} .sel-cap .i{width:12px;height:12px;stroke-width:1.8}`. Redraw `updown` for the cap with bigger travel (`M5 8.2 10 3.6l5 4.6M5 11.8l5 4.6 5-4.6`, about 10 x 13 units) so it renders about 7 x 9 px at 12 px. Alternatively `.sel-wrap > svg.i.s16{height:24px;padding:0}` is not enough: the square viewBox still shrinks the glyph to 12 px wide. |
| S2 | Hover / pressed (dark) | Hover lightens, as `--e1` control | `.sel:hover` background `rgba(255,255,255,.10)` replaces `#4a4a4d`: composite over `--raised` is about `#414143`, i.e. **darker** than rest. No `:active`. | must (see B1) | `.sel:hover{background:color-mix(in srgb,var(--ctl-bg),var(--hover-plane) 100%)}` or layer: `background-image:linear-gradient(var(--hover-plane),var(--hover-plane))` over `background-color:var(--ctl-bg)` (same for `.btn`). Add `.sel:active` with `--press`. |
| S3 | Focus (keyboard) | Soft halo (SPEC 12) | OK (`inset 1px --focus` + `3px --focus-soft`), but the 16 px cap is painted over the inset ring on the right edge; and in a last-row select the 3 px halo exactly fills the 3 px row padding (touches the group edge). | nit | Draw the ring on `.sel-wrap:focus-within` instead, or give the cap `box-shadow: inset 0 0 0 1px var(--focus)` while `select:focus-visible ~ .sel-cap`. |
| S4 | Open list (native, OS drawn) | SPEC 6: selected item `--acc` white 600, check mark on current | Native `<select>`; only `.sel option{background:var(--float);color:var(--t1)}` is styled. Selected/hover highlight comes from the OS (Windows system accent, not `#2876D2`), no check mark. | should | Accept as known deviation or replace with a custom listbox (`role=listbox`, arrow keys, Esc). Not verified: the OS popup is not capturable in the pane; option colours in dark should be checked in the real app. |
| S5 | Disabled | Dim like `.btn:disabled` | No `.sel:disabled` rule (select would look enabled). Not used today. | nit | `.sel:disabled{opacity:var(--dis)}` + cap opacity. |
| S6 | Two selects in one column (AppRulesDialog) | Same width | Both 112 wide (width prop) = OK. Chung Select is 96 (auto, content-sized) vs buttons 148/96: fine. | ok | n/a |

## 2. Buttons `.btn`, `.btn.pri`, `.btn.ghost`, `.btn.icon`, `.btn.sm` (kit.css)

| # | Where / state | Expected | Actual | Sev | Fix |
|---|---|---|---|---|---|
| B1 | Hover / pressed, **dark** (Chung "Bảng gõ tắt…", Advanced, footer "Thoát hẳn" excluded) | Hover lightens (`--hover-plane` is a white 10 % overlay) | `.btn:hover{background:var(--hover-plane)}` replaces `--ctl-bg #4a4a4d` with `rgba(255,255,255,.1)`: measured computed bg `rgba(255,255,255,0.1)`, composite about `#414143` = darker than rest (74 -> 65). Pressed `.14` is about `#454547`, still darker than rest. Light mode is fine (white -> `#ededed`). | must | Overlay instead of replace: `.btn:hover{background-image:linear-gradient(var(--hover-plane),var(--hover-plane))}` (keep `background-color:var(--ctl-bg)`), same for `:active` with `--press`. Apply to `.sel`. |
| F1 | Keyboard focus on `.btn.pri` (Đóng, Thêm, Lưu, Cập nhật ngay, banner "Cấp quyền…") | 3 px focus ring on buttons (SPEC 9) | **No ring.** `.btn.pri` sets `box-shadow:var(--pri-shadow)` (0,2,0) which beats `button:focus-visible` (0,1,1). Measured `sh=rgba(0,0,0,.3) 0 0 0 .5px, …`. (`_audit-shots/web-focus-primary-no-ring.jpg`: Đóng focused, nothing visible.) | must | Raise the ring specificity: `button.btn:focus-visible, button.btn.pri:focus-visible, button.btn.ghost:focus-visible, .seg > button.on:focus-visible, .langtg button.on:focus-visible{box-shadow:0 0 0 1px var(--raised),0 0 0 3px var(--focus)}` or move the ring to `outline:2px solid var(--focus);outline-offset:2px` (no clash with box-shadow). |
| F2 | Keyboard focus on `.btn.ghost` / `.btn.ghost.icon.sm` (Thoát hẳn, Đặt lại tất cả, Huỷ, sheet close, table row icons) | same ring | **No ring**: `.btn.ghost{box-shadow:none}` (0,2,0). Measured `sh=none`. | must | same fix as F1 |
| B2 | Primary hover / pressed | visible step | `--acc #2876d2`, `--acc-hover #2572cc`, `--acc-press #1f6fca`: hover differs by about 3-6 levels per channel (imperceptible), pressed 7-9. Dark `--acc-press` is darker too, while SPEC 5 gives dark `#4C97E6`. | should | Light: hover `#2169bd`, press `#1b5dab` (white stays above 5:1). Dark: hover `#3a85dc`, press `#4c97e6` as SPEC 5, check white text >= 4.5:1. |
| B3 | Heights | 24 for desktop buttons (SPEC 12) | 24 everywhere measured (Chung, footer, banner, About, sheet). `.btn.sm` 20, `.btn.icon.sm` 20 x 20. | ok | n/a |
| B4 | Hit area | >= 24 x 24 (SPEC 9; `::before` extension) | `.btn.icon.sm` 20 x 20 (table row Sửa / ⋯, sheet close, capture reset/cancel), `.btn.sm` 20 high (bulk bar Sửa/Xuất/Xoá/Bỏ chọn). SPEC 9's `::before` extension is not implemented anywhere. | should | `.btn.sm,.btn.icon.sm{position:relative}` + `.btn.sm::before,.btn.icon.sm::before{content:"";position:absolute;inset:-2px}` (and the same on `.cb`, `.tg`, `.langtg button`, `.tslot`, `.toast .x`). |
| B5 | Icon size inside buttons | `.btn .i` = 14 | `svg.i.s16` (0,2,1) beats `.btn .i` (0,2,0): icons are 16 (e.g. bulk-bar Sửa/Xóa) while `.input .i` is 14 and `.toast .gi .i` 14. Three icon sizes (14/16/20) with no rule. | nit | Decide: 16 in 20-24 px buttons, 14 in fields/toasts; make `.btn .i` selector `svg.i.s16` aware or drop the dead `.btn .i`. |
| B6 | Press state of `.btn.ghost` | has `:active` | `.btn:active` ok; `.btn.ghost` only hover (`:active` falls to `.btn:active` background, ok). | ok | n/a |
| B7 | Disabled (`Bảng gõ tắt…` when macro off, export) | opacity .42 | OK; text contrast naturally low. `pointer-events:none` also removes the tooltip (`title` on export button). | nit | Keep hit-testing for `title`, block via `aria-disabled` + `cursor:not-allowed` if tooltips matter. |
| B8 | Row height vs control | SPEC 12: 28 px row holds a 24 px control | Rows with a 24 px control are **30** px (`min-height:28; padding:3px 12px`). Measured Chung group 2 = 116 = 28+28+30+30; group 1 = 58. Toggle rows are 28. Mixed row heights inside one group. | should | `.row,.v4row{padding:2px 12px}` (28 holds a 24 control) or accept 30 and update SPEC. Also fixes the 1 px focus-ring clip in B9. |
| B9 | Focus ring clipped by group edge | Ring fully visible (SPEC 9) | `.v4grp{overflow:hidden}`; a button in the last row has 3 px to the edge, ring needs 4 px (1 gap + 3): `Danh sách ứng dụng…`, Advanced `Mở thư mục cấu hình`. Toggles: ring touches edge exactly. | nit | `padding:2px 12px` row + `overflow:clip; overflow-clip-margin:4px` or `.v4grp{overflow:visible}` with `border-radius` on first/last row (already set). |

## 3. Switch `Toggle` / `.tg`

| # | State | Expected | Actual | Sev | Fix |
|---|---|---|---|---|---|
| T-1 | Size, alignment (all tabs) | 36 x 20, right edge 12 from row, centred | OK: 36 x 20, right edge 12, vertical offset 0 in every row measured. | ok | n/a |
| T-2 | Hover / pressed | SPEC 6: hover `--acc-hover`, pressed `--acc-press` | `.tg.on:hover/:active` = same `--acc-switch`; off state has no hover/active. No feedback at all. | should | `.tg:hover{filter:brightness(1.05)}` / `.tg:active::after{width:18px}` (macOS squash) or the SPEC colours. |
| T-3 | Off track, light | non-text boundary >= 3:1 (SPEC 2b) | `--tg-off` over `--raised` = **1.44:1** (dark 1.79:1). The white knob carries it, but the rule is not met. | nit | Light `--tg-off` `rgba(120,120,128,.45)` (about 2.2:1) + 0.5 px inset ring `--ctl-edge`; or note exception in SPEC. |
| X1 | Disabled toggle inside disabled row (Advanced: "Tự động khôi phục từ gõ sai", "Vẫn dùng gõ tắt…") | Dim once (.42) | `.v4row.dis` opacity .42 **and** `.tg:disabled` opacity .42 => effective **.18**, toggle nearly invisible (`web-advanced-dark.jpg`). Row has no `aria-disabled` (SPEC 6). | must | In `Toggle`: do not apply `.tg.dis` dimming when inside `.dis` (`.row.dis .tg,.v4row.dis .tg{opacity:1}`) and add `aria-disabled="true"` to the row. |
| T-4 | Label click | SPEC 9: row label toggles the switch | Clicking the label text does nothing (label is a `div`, cursor default; verified: `aria-checked` unchanged). | should | Make `.v4row .l` a `<label for=…>` or add `onclick` on the row for switch rows; `cursor:default` is fine on desktop. |
| T-5 | Focus ring | 3 px ring | OK (`0 0 0 1px --raised, 0 0 0 3px --focus`, duplicate `0 0 0 1px --raised` entry in the list). | nit | Remove the duplicated first two shadows in kit.css line 12. |

## 4. Segmented `Segmented` / `.seg` (Phím tắt "Kiểu chuyển bật/tắt")

| # | State | Expected | Actual | Sev | Fix |
|---|---|---|---|---|---|
| G1 | Height | 24 like every other control (SPEC 12 raised all controls to 24; SPEC 5 says seg 22) | Tray 22 (18 + 2 + 2), next to 24 px buttons/selects. | should | `.seg > button{height:20px}` => tray 24. |
| G2 | Hit area | >= 24 | Segment 18 high. | should | See B4 (`::before` inset -3px). |
| G3 | Focus on selected segment | Ring | `.seg > button.on` has its own `box-shadow` (0,2,1): **no focus ring** on the selected segment, ring only on the unselected one. | must (with F1) | add to the F1 selector list. |
| G4 | Text size | 13 like siblings | 12 px (buttons/selects 13). | nit | Keep 12 if intended; else 13 and padding 0 9. |
| G5 | Selected contrast | visible | Selected `--seg-on` white over tray: 1.39:1 (light), 1.8:1 (dark); carried by a 0.5 px edge and shadow. Acceptable macOS look. | nit | none |
| G6 | Hover / pressed | hover bg | hover ok (`--hover`), no `:active`. | nit | `.seg > button:active:not(.on){background:var(--press)}` |
| G7 | Radius | 6 | Tray 8, thumb 6; elsewhere controls are 6. Fine for nested radius, but tray 8 is the only 8 on a control. | nit | tray 7 or keep. |

## 5. Checkbox `.cb` / `Checkbox.svelte` (Gõ tắt table, bulk bar)

| # | State | Expected | Actual | Sev | Fix |
|---|---|---|---|---|---|
| C1 | Row alignment (Macros table) | Centred in the 28 px row | Checkbox top-aligned inside a 20 px `span.c0` line box: y 184..200 vs row centre 194 => **2 px above centre** while keycap/action buttons are centred (offset -2 in the audit). | should | `.tr .c0{display:flex;align-items:center}` (or `display:grid;place-items:center`). |
| C2 | Hit area | >= 24 | 16 x 16 (Macros). | should | B4 pattern; make the whole `.c0` cell the target. |
| C3 | Disabled | dim | `Checkbox.svelte` has `disabled` prop but kit has no `.cb:disabled`. Component is **unused** (Macros writes `.cb` buttons directly, mixed state `.cb.mix` is not in `Checkbox.svelte`). | nit | Delete `Checkbox.svelte` or add `mix` + disabled style and use it. |
| C4 | Unchecked edge | 3:1 | `inset 0 0 0 1px --ctl-edge` over `--ctl-bg`: 1.41:1 (light), 1.33 (dark). | nit | `--ctl-edge` alpha .28 for checkbox only. |
| C5 | Checked | `--acc`, white check | OK. Mixed (bulk bar) draws a dash, OK. | ok | n/a |

## 6. Keycap `Keycap` / `.k`

| # | State | Expected | Actual | Sev | Fix |
|---|---|---|---|---|---|
| K0 | Size | 18, mono 20, sm 16 | OK (18 / 20 / 16, radius 4, font 11 / 12 / 10). `.k.sm` 10 px text is below the 11 px floor used elsewhere. | nit | `.k.sm{font-size:10.5px}` or keep 11 with height 16. |
| K0b | Mono keycap in Gõ tắt table vs header | text aligned | Header "Từ tắt" text starts at x 72; mono keycap box starts at 72 but its text at 77 (5 px padding). | nit | Header `padding-left:5px` or keycap `margin-left:-5px`. |
| K0c | Row menu / bulk bar `Esc` keycap | fine | OK. | ok | n/a |

## 7. CaptureField `.cap` / `.capa` (Phím tắt)

| # | State | Expected | Actual | Sev | Fix |
|---|---|---|---|---|---|
| K1 | Default (set / unset / clash), `web-hotkeys-dark.jpg` | The whole 248 x 24 well is the control; keycaps spaced 3 px; hint/tag at the right (`.cap .end{margin-left:auto}`) | `.capa` has **no CSS** (grep: only the class attribute exists). It is a default `<button>` shrink-wrapped to its content: widths 59 / 96 / 77 / 69 px of a 248 px well. Consequences: (a) clicking the empty 75 % of the well hits the `div.cap` and does nothing (`elementFromPoint(440,143)` = `div.cap`); (b) keycaps have **0 gap** (Ctrl x210..237, Shift x237..) because `gap:3px` is on `.cap`, not on the button; (c) `.end` ("Đặt phím", "Dễ trùng" tag) is glued to the text instead of right-aligned (measured x 261..306, well ends 452); (d) text `Chưa đặt` 12 px and `Đặt phím` 11 px next to each other. | must | `.cap .capa{flex:1;min-width:0;height:100%;display:flex;align-items:center;gap:3px;padding:0;text-align:left;background:none;overflow:hidden;color:inherit;font:inherit} .cap .capa:focus-visible{outline:none}` (focus => recording state already styles the well). Keep `.end{margin-left:auto}`. |
| K2 | Recording (`web-hotkeys-recording.jpg`) | caret blinks, hint fits | `.caret` is an inline `span`: width/height ignored => **0 x 17 px, invisible**. Held keys + `+` + `Esc huỷ` + the cancel button all in one 248 well with `overflow:hidden`: "huỷ" is at the clip edge with 2 held keys, clipped with 3. | should | `.cap .caret{display:inline-block;flex:none}`; drop the Esc hint text when >= 2 keys held, or `min-width:0` + ellipsis on `.end`. |
| K3 | Recording border / colour | 2 px `--acc`, text `--acc-text` | OK (inset 2 px `rgb(40,118,210)`, text `rgb(106,176,242)`). | ok | n/a |
| K4 | Reset / cancel icon button inside the well | in-well, right | 20 x 20 `btn ghost icon sm` placed right after `.capa` (x 342), not at the well's right end; hit 20. | should | After K1 fix put it `margin-left:auto`; B4 hit area. |
| K5 | Tab into field | no side effect | `onfocus={begin}`: simply tabbing through Phím tắt starts recording on each stop (the page then swallows keys). Behaviour, not drawn here. | should | Start recording on Enter/Space/click only; Tab should pass. (Not verified that Tab itself is swallowed.) |
| K6 | Empty text | 4.5:1 | `Chưa đặt` uses `--t3`: 2.2:1 light / 2.7:1 dark (T1). | must (T1) | use `--t2` |

## 8. Row `Row.svelte`, `.row`, `.v4row`, groups

| # | State | Expected | Actual | Sev | Fix |
|---|---|---|---|---|---|
| R1 | `Row` note (AppRulesDialog first row "Nhớ trạng thái bật/tắt riêng…", `web-apprules-macos.jpg`) | Hint 11 px `--t2` under the 13 px label | `Row.svelte` renders the note as `<span>`; kit only styles `.row .lbl small` / `.v4row .h`. The note is **13/14 px, `--t1`**, identical to the label (measured). Row is 36 high (not 28/38). | should | In `Row.svelte` use `<small>`; or add `.row .lbl > span{font:400 11px/14px var(--ui);color:var(--t2)}`; set `min-height:38px` when a note exists. |
| R2 | Row heights | 28 / 38 | 28 (toggle), 30 (24 px control), 38 (hint) (see B8); AppRulesDialog list rows 28, first one 29 (bottom border). | should | B8 |
| R3 | Right edge alignment | controls share one right edge | OK in Chung/Advanced/Phím tắt (12 px). Gõ tắt table uses 10 px (`.tr` padding 10) and toolbar 16: three different insets. | nit | `.tr{padding:0 12px}` |
| R4 | Row hover | none required | Table rows have no hover background (only `acts` appear). Cursor default. | nit | `.tr:hover{background:var(--hover)}` |
| R5 | Divider | inset 12 | OK. | ok | n/a |
| R6 | Disabled row | `opacity .42` + `aria-disabled` + no pointer | `pointer-events:none` yes, `aria-disabled` missing. | should | add attribute in Advanced/Chung markup. |

## 9. Text fields `.input`, textarea (Gõ tắt search, sheet)

| # | State | Expected | Actual | Sev | Fix |
|---|---|---|---|---|---|
| I1 | Height / radius | 24 / 6 | 24 / 6 (search, sheet input); textarea **72** in the sheet (Macros.svelte local `height:72px`), kit and SPEC say 64. | nit | delete the local override. |
| I2 | Focus | 1 px accent + 3 px soft halo | OK (`inset 1px --focus` + `0 0 0 3px --focus-soft`). | ok | n/a |
| I3 | Error | 2 px `--bad` | `.input.err` replaces and (being later in the file) **wins over focus**: a focused erroneous field shows no focus halo. | nit | `.input.err:focus-within{box-shadow:inset 0 0 0 2px var(--bad),0 0 0 3px color-mix(in srgb,var(--bad) 30%,transparent)}` |
| I4 | Padding | 7 | Sheet inputs 10 px (local `.field .input{padding:0 10px}`), search 7. Two paddings for one control. | nit | remove the local override. |
| I5 | Placeholder / hint contrast | 4.5:1 | placeholder, `.hint` ("Từ tắt viết liền…", "0 ký tự") use `--t3` (T1). Local `.hint` is 12 px, kit says 11. | must (T1) | tokens |
| I6 | Sheet label | 12 / 400 (kit `.field`) | local `.field` is 13 / **500** `--t2`. | nit | remove local override. |
| I7 | Search icon | 14 | 14 (OK, but the only `Icon` rendered without `small` at 14 via `.input .i`). | ok | n/a |

## 10. Toolbar `Toolbar.svelte` / `.tab`

| # | State | Expected | Actual | Sev | Fix |
|---|---|---|---|---|---|
| TB1 | Size | tab 64 x 44, gap 2, radius 7 | OK (5 x 64 + 4 x 2, centred, 52 high; content centre -0.5 because of the 1 px border). | ok | n/a |
| TB2 | Selected icon colour | `--acc`, >= 3:1 | uses `--acc-switch #3d8be0` on `--hover-plane` over `--plane`: **2.47:1 light** (dark 3.74). | should | `.tab[aria-selected="true"] svg{color:var(--acc)}` in light (`#2876d2` about 3.6:1). |
| TB3 | Glyph stroke | same optical weight as other icons | Glyphs are a 24-unit grid at 20 px with `stroke-width:1.5` => **1.25 px** effective; `Icon` s16 gives 1.5 px, tslot 1.4 at 14/16 => 1.2 px. Three weights. | nit | `.tab svg{stroke-width:1.8}` (=1.5 px) and `.tslot svg{stroke-width:1.7}`. |
| TB4 | Hover / focus / pressed | hover `--hover`, ring | hover ok, focus ring ok (selected and unselected), no `:active`. | nit | `.tab:active{background:var(--press)}` |
| TB5 | Update dot | 7 px accent | 7 px, top 6 right 14: sits on the About tab top-right; the title bar also has a 7 px dot (two dots for one state). | nit | intended (SPEC 8.5), keep. |
| TB6 | Tab labels VI/EN | no clip | EN (General/Shortcuts/Macros/Advanced/About) and VI measured: widest label 56 px in 64: no overflow. | ok | n/a |

## 11. Title bar `TitleBar.svelte`, `LangToggle.svelte`

| # | State | Expected | Actual | Sev | Fix |
|---|---|---|---|---|---|
| H1 | Update-dot button (Windows) | hit >= 24 | 14 x 14 (`.tslot.upd`). | should | wrap in 24 x 24 hit area (`::before inset:-5px`). |
| H2 | Theme button | 24 hit | 22 x 22 (macOS bar is 28 and Windows 36 high: room for 28). | should | `.tslot{width:28px;height:28px}` on Windows (keep the 14 glyph). |
| H3 | VI / EN segments | equal width, 24 hit | VI 22.3 x 18, EN 26.1 x 18 (different widths; 18 high in a 36 bar); tray 20 high. | should | `.langtg button{min-width:26px;height:22px}`; tray 24. |
| H4 | Focus on selected language | ring | no ring on `.langtg button.on` (own box-shadow, F1). | must (F1) | selector list |
| H5 | Theme glyph | shows current or target? | shows the **target** theme (moon in light, sun in dark); tooltip says "Giao diện: Tối". Ambiguous. | nit | show current, or add `aria-pressed`. |
| H6 | Windows caption buttons | 44 x 36, red close hover | OK (`#c42b1c`, white). No `:active`, no focus style (`.wbtn` shows the default ring clipped at the window edge). | nit | `.wbtn:focus-visible{box-shadow:inset 0 0 0 2px var(--focus)}` |
| H7 | macOS bar | 28 high, title centred, right cluster 8 px | OK (title y 5..23, cluster right 8, tslot 22 in 28). | ok | n/a |
| H8 | Real app (125 %) | same as browser | `real-titlebar-zoom4x.png`: VI/EN thumb and theme icon render as designed; right window corner radius visible. | ok | n/a |

## 12. Modal / sheet `Modal.svelte`, `.sheetdlg` and AppRulesDialog

| # | State | Expected | Actual | Sev | Fix |
|---|---|---|---|---|---|
| M1 | Placement | under the toolbar, 440 wide, bottom radius 12 | OK: x 20, y = chrome + 52 (88 Win, 80 Mac), 440 wide, `0 0 12 12`. | ok | n/a |
| M2 | Close button | hit >= 24 | 20 x 20 ghost icon (and no focus ring, F2). | should | B4 + F2 |
| M3 | Footer | 24 px buttons | Huỷ / Lưu 24 high, gap 8, right inset 16. | ok | n/a |
| M4 | Backdrop | `rgba(0,0,0,.28)` over content | `--shade` .28 light / .45 dark covers the title bar and toolbar too; toolbar tabs stay clickable? (not tested) | nit | `inert` the page behind (`showModal` already does). |
| M5 | Row menu in Gõ tắt (`.rowmenu`, `web-macros-rowmenu.jpg`) | same menu language as the rest | local style: radius **10**, items 30 high, weight **500**, no icons, no keyboard hint; elsewhere radius 6/8, 13/400. Shows 2 items ("Nhân bản", "Xoá"). | should | `.rowmenu{border-radius:8px} .rowmenu button{height:26px;font-weight:400;border-radius:5px}`; the `Select` list will need the same look. |
| M6 | Sample rows (empty state) | kit `.sample-row` 26 high radius 6 | local override 34 high radius 8, 13/20. (Empty state not rendered: see "not checked".) | nit | delete local override. |

## 13. Toast `Toast.svelte`, `.toast` (rendered by injecting the same markup, `web-toasts-injected-macos.jpg`)

| # | State | Expected | Actual | Sev | Fix |
|---|---|---|---|---|---|
| N1 | Close button | 24 hit | 22 x 22, glyph 12. | should | B4 pattern |
| N2 | Two-line error toast | icon top-aligned | `align-items:center`: icon floats at the middle of two lines. | nit | `.toast{align-items:flex-start} .toast .gi{height:16px}` |
| N3 | Action link | 600, `--acc-text` | OK (12 px / 600); hit 56 x 16. | nit | add `padding:4px` hit area. |
| N4 | Size / position | min 34, 8 radius, above footer | OK (34, radius 8, y bottom 50). | ok | n/a |

## 14. Banners (macOS) `AccessibilityBanner`, `SecureInputHint`

| # | State | Expected | Actual | Sev | Fix |
|---|---|---|---|---|---|
| A1 | Banner layout | 72 high, title 13/600, body 11, pri button | OK (72, icon 18, button 24 high, y 92..164, then 8 gap). | ok | n/a |
| A2 | Banner button focus | ring | primary => none (F1). | must (F1) | n/a |
| A3 | `SecureInputHint` | 28 high | Not shown in the browser (needs native status): **not checked**. | - | - |

## 15. Bulk bar and table (Gõ tắt) `.bulk`, `.tr`

| # | State | Expected | Actual | Sev | Fix |
|---|---|---|---|---|---|
| U1 | Bulk bar, 1 selected (`web-macros-bulkbar.jpg`) | 30 high, single line | Left label is two lines ("Đã chọn 1" / "trong 8"): the `.n` block is **33 px tall in a 30 px bar** (y 154.5..187.5 vs bar 156..186), `small` is **10 px**. | should | `.bulk .n{flex-direction:row;align-items:baseline;gap:4px} .bulk .n small{font-size:11px}` => "Đã chọn 1 / 8"; keep bar 30. |
| U2 | Table header | 11/600 `--t2` (kit) | local `.tr.head` 12/600 **`--t3`** (2.7:1 dark, 2.2:1 light), bg `--seg-bg`. | must (T1) | remove the local override. |
| U3 | Row action icons | visible to keyboard and pointer | `opacity:0` until hover/focus-within; icons 16 in 20 px buttons, hit 20. | should | B4 |
| U4 | Selected row | `--acc-tint` | OK. | ok | n/a |
| U5 | Header / body scrollbar | tokens | custom 10 px thumb `--line-3`; fine. | ok | n/a |
| U6 | Local overrides of kit values in `Macros.svelte` (`.lnk` 500/12, `.hint` 12, `.field` 13/500, `.sample-row` 34/8, `.rowmenu`, textarea 72) | single source in kit.css | seven local overrides contradict kit; one control, two looks. | should | move into kit or delete. |

## 16. Contrast / tokens (computed from resolved values)

| # | Token pair | Result | Sev | Fix |
|---|---|---|---|---|
| T1 | `--t3` text on `--raised` / `--surface` | light **2.24 / 2.19:1**, dark **2.70 / 2.82:1** (target 4.5:1 for text, per SPEC "tương phản hợp lý"; tokens.css comment claims "still >= 4.5:1"). Used for: placeholder, `Chưa đặt`, `.hint`, table header, About changelog date, "+" in caps, version hints. | must | light `--t3: rgba(60,60,67,.70)`, dark `--t3: rgba(235,235,245,.55)` (about 4.5:1), or switch the informational users to `--t2` and keep `--t3` for placeholders only. `--t4` (1.73 / 2.09) is decoration only. |
| T2 | white on `--acc` | 4.55:1 (both themes), on hover 4.82 | ok but marginal; keep. | ok | n/a |
| T3 | `--acc-text` on `--plane` (light) | 4.29:1 | nit | darken light `--acc-text` to `#1a63b5` |
| T4 | `--focus` on `--surface` light | 4.18:1; dark 6.2:1 | ok | n/a |
| T5 | Control edges vs `--raised` (light) | `--ctl-bg` is `#fff` on `#fff` (1.0:1); edge `--ctl-edge` gives 1.41:1; select/button boundary relies on a .5 px ring + 1 px shadow, and at 125 % DPI a .5 px shadow rounds to 1 device px (slightly heavier than at 100/200 %). | nit | none, or `--ctl-edge` .22 in light. |

## 17. Cross-control consistency

| # | Rule | Result | Sev |
|---|---|---|---|
| X-1 | Control height | button / select / input / cap = 24; **segmented = 22**; langtg 20; tslot 22; keycap 18/20; toggle 20; checkbox 16; icon button 20 | should (G1) |
| X-2 | Radius | controls 6; tab 7; keycap/checkbox 4; seg tray 8; group 8; toast 8; row menu **10**; sample-row **8**; sheet bottom 12 | nit (M5, M6) |
| X-3 | Font size in controls | 13 (btn, sel, input, cap value), 12 (seg, `.btn.sm`, cap.empty, banner), 11 (tab, langtg, keycap) | nit |
| X-4 | Icon size / stroke | 14 / 16 / 20 px; effective stroke 1.25 (tabs) / 1.5 (Icon s16) / 1.2 (tslot) | nit (TB3, B5) |
| X-5 | Hit area >= 24 (Windows) | fails: update dot 14, theme 22, VI/EN 18 high, seg 18, icon btn 20, bulk `.btn.sm` 20, checkbox 16, toast close 22, About footer links and fold 16 high, toggle 36 x 20 | should (B4) |
| X-6 | Hover/press feedback | `.btn` hover inverted in dark (B1); toggle none; seg/tab/langtg no `:active`; primary step imperceptible | should |
| X-7 | Focus ring | ring missing on 5 families (F1/F2/G3/H4); present on toggle, tab, tslot, cb, sel, input, secondary btn | must |

## Not checked / limits

- Native `<select>` open list (OS-drawn): option colours in dark and the selected-item highlight not verified; needs a look in the real app.
- Real app only captured once (Chung, dark, Windows 125 %; PID 70108 no longer exists when I finished); other tabs, light mode and hover/focus in the real app not captured. I did not touch it.
- Gõ tắt empty state, "Gõ tắt đang tắt" warn strip, import/export dialog, `SecureInputHint`, update/error states of About, `tone` variant of Segmented: not reachable with the mock data; not rendered.
- Toast and banner were rendered from the same CSS (toast by injecting markup with the real classes), not from app state.
- Press (`:active`) states inferred from CSS, not from a held pointer.
- Hover on VI/EN, tslot, tabs inferred from CSS (token values), not screenshot-compared.
- Screenshots from the browser pane are 436 px wide (scaled); measurements come from `getBoundingClientRect`, not pixels.

## Scratch files left (not deleted)

`design/macos-style/_audit-shots/` (all `web-*.jpg`, `real-*.png`, `_capture.ps1`, `_crop.ps1`), `design/macos-style/_audit-vite.log`, `design/macos-style/_audit-vite.pid`.

## Fix round (2026-11-09, coder)

Re-measured in the browser at 480 px, light and dark, `?platform=windows`. The macOS path shares the kit rules; it is checked by CSS, not captured. "Measured" = read with getBoundingClientRect / getComputedStyle / a real Tab key. "CSS" = fixed by a rule in ui/src/styles/kit.css (fix block at the end) or v4.css. The pointer hover could not be captured: the browser pane scales the page, so real coordinates fell outside the frame. Hover rows therefore say CSS.

Row height decision (B8, R2): **28 px rows hold the 24 px control** (row padding 2px 12px). Hint rows stay 38. SPEC numbers unchanged.

Counts: must 6 of 6 FIXED (T1 by token, measured). Should 21: 17 FIXED, 4 NOT FIXED (native select list, K5 Tab-to-record, About footer links 16 high, X-5 remainder). Nits: 11 FIXED, 9 NOT FIXED or kept on purpose (listed below).

| # | Status | Evidence / reason |
|---|---|---|
| S1 | FIXED | Measured: cap 16 x 24 at the select's top-right (select 96 x 24), glyph 12 x 12, radius 0 6 6 0. |
| S2 | FIXED | Hover overlay: gradient of --hover-plane over --ctl-bg; :active uses --press (CSS). |
| S3 | NOT FIXED (nit) | Cap still sits over the 1 px inset ring on the right edge. Left to avoid a focus regression. |
| S4 | NOT FIXED | Native select list is drawn by the OS and cannot be styled. color-scheme now inherits the root scheme (dark in dark mode), so Windows should draw a dark list (CSS). Selected-item colour stays OS. Verify in the real app; a custom listbox is a later round. |
| S5 | FIXED | .sel:disabled dims with --dis (CSS). |
| B1 | FIXED | .btn:hover is an overlay on --ctl-bg (CSS). Rest colour measured rgb(74,74,77) in dark. Pointer hover not captured in the pane. |
| F1 | FIXED | Measured Tab-focused "Đóng" (.btn.pri): ring 0 0 0 1px --raised, 0 0 0 3px --focus. |
| F2 | FIXED | Measured Tab-focused ghost button: same ring. |
| B2 | FIXED | Light hover #2169bd, press #1b5dab; dark hover #3a85dc, press #4c97e6 (tokens.css). |
| B3 | ok | 24 high, unchanged. |
| B4 | FIXED | Hit extension ::before on .btn.sm, .btn.icon.sm, .cb, .tslot, .toast .x, .seg > button, .langtg button, .tg (CSS). Not measured with elementFromPoint. |
| B5 | NOT FIXED (nit) | Icon sizes 14/16/20 unchanged: changing them moves every glyph. |
| B6 | ok | - |
| B7 | NOT FIXED (nit) | pointer-events:none still hides the title tooltip on disabled buttons. Kept. |
| B8 | FIXED | Measured Chung groups: 28,28 / 28,28,28,28 / 38,38 (was 30). |
| B9 | FIXED | .v4grp overflow visible (CSS): the ring is no longer cut at the last row. |
| T-1 | ok | - |
| T-2 | FIXED (press) / NOT FIXED (hover) | Press squashes the knob (:active::after, CSS). No hover colour added. |
| T-3 | FIXED | Light --tg-off rgba(120,120,128,.45) (tokens.css); contrast not re-measured. |
| X1 | FIXED | Measured Advanced disabled row: row opacity 0.42, switch opacity 1 (dims once). |
| T-4 | FIXED | Measured: clicking the label "Kiểm tra chính tả" flips aria-checked false to true. Delegated handler in App.svelte. |
| T-5 | NOT FIXED (nit) | Duplicate ring entry in an old kit rule is harmless; left. |
| G1 | FIXED | Measured segmented tray 191 x 24, buttons 20 high. |
| G2 | FIXED | Hit ::before inset -2 / -3 (CSS): 20 + 4 = 24 high. |
| G3 | FIXED | .seg > button.on:focus-visible ring (CSS; not measured). |
| G4 | NOT FIXED (nit) | Kept 12 px on purpose. |
| G5 | ok | - |
| G6 | FIXED | .seg > button:active:not(.on) press (CSS). |
| G7 | NOT FIXED (nit) | Tray radius 8 kept. |
| C1 | FIXED | Measured Gõ tắt row: row centre 194, checkbox centre 194, keycap centre 194. |
| C2 | FIXED | .cb::before inset -4 = 24 hit around the 16 box (CSS). |
| C3 | NOT FIXED | Checkbox.svelte is unused but tracked; deleting it needs the owner's go-ahead. Left in place. |
| C4 | NOT FIXED (nit) | Unchecked edge alpha unchanged. |
| C5 | ok | - |
| K0 | FIXED | .k.sm 10.5 px (CSS). |
| K0b | NOT FIXED (nit) | 5 px header/keycap text offset left as is. |
| K1 | FIXED | Measured: .capa 236 x 24 inside the 248 x 24 well, keycaps at x 210 and 240 (3 px gap). |
| K2 | FIXED (caret) / NOT FIXED (text) | .caret is an inline-block 1 x 14 (CSS). "Esc huỷ" still shares the well with 2+ held keys; shortening it needs a CaptureField change, deferred. |
| K3 | ok | - |
| K4 | FIXED | Reset/cancel button at the well's right end (flex; .capa takes the free space). |
| K5 | NOT FIXED | Tab into a capture field still starts recording (onfocus). A behaviour change in CaptureField, outside this round; for the owner. |
| K6 | FIXED | "Chưa đặt" uses --t3, now 5.05 to 5.31 light, 5.52 to 6.19 dark (T1). |
| R1 | FIXED | Row note is a small element, 11 px --t2; row 38 when a note exists (markup + CSS). |
| R2 | FIXED | See B8; AppRulesDialog rows 28. |
| R3 | FIXED | Table row padding 12 (CSS). |
| R4 | FIXED | .tr:hover background (CSS). |
| R5 | ok | - |
| R6 | FIXED | aria-disabled on the Advanced disabled rows (measured true). |
| I1 | FIXED | Local textarea height removed; kit 64. |
| I2 | ok | - |
| I3 | FIXED | .input.err:focus-within keeps the focus halo (CSS). |
| I4 | FIXED | Local .field .input padding removed. |
| I5 | FIXED | Placeholder and hint on --t3 (T1). Placeholder inside the dark control fill measures 4.00:1: accepted as the 3:1 floor for placeholders (justification: placeholders are exempt from the text rule). |
| I6 | FIXED | Local .field (13/500) removed; kit 12/400. |
| I7 | ok | - |
| TB1 | ok | - |
| TB2 | FIXED | Tab glyph uses --acc-text: light 4.29 on the plane, dark 5.95 (computed); measured dark rgb(106,176,242). |
| TB3 | FIXED | Tab glyph stroke 1.8 (1.5 px); title-bar glyph 1.7 (v4.css). |
| TB4 | FIXED | .tab:active press (CSS). |
| TB5 | kept | Intentional (SPEC 8.5). |
| TB6 | ok | - |
| H1 | FIXED | Update dot hit ::before inset -5 = 24 (CSS); the dot moved to ::after. |
| H2 | FIXED | Theme slot hit ::before inset -1 = 24 (CSS). |
| H3 | FIXED | VI/EN min-width 26, height 22, tray 24 (CSS; not measured). |
| H4 | FIXED | Ring on the selected language (CSS). |
| H5 | NOT FIXED (nit) | Theme glyph still shows the target state. Kept. |
| H6 | FIXED | .wbtn:focus-visible inset ring (CSS). |
| H7 | ok | - |
| H8 | ok | - |
| M1 | ok | - |
| M2 | FIXED | Close icon hit 24 (::before) and ring (F2). |
| M3 | ok | - |
| M4 | NOT FIXED (nit) | Backdrop still tints the title bar and toolbar; inert not added. |
| M5 | FIXED | Shared .rowmenu in kit (radius 8, rows 26, weight 400); Macros copy removed. |
| M6 | FIXED | Local .sample-row removed; kit 26 high. |
| N1 | FIXED | Toast close hit 26 (::before inset -2, CSS). |
| N2 | FIXED | Toast align-items flex-start, icon padded 2 px (CSS). |
| N3 | FIXED | Toast action link padding 4 px (CSS). |
| N4 | ok | - |
| A1 | ok | - |
| A2 | FIXED | Banner primary button uses the F1 ring (CSS). |
| A3 | not checked | SecureInputHint needs native status. |
| U1 | FIXED | Measured bulk bar 30 high, one line "Đã chọn 1 trong 8", 12 px. |
| U2 | FIXED | Header is the kit style: 24 high, 11/600, --t2 (measured). Local override removed. |
| U3 | FIXED | Row action icons hit 24 (::before); still shown on hover and focus-within (unchanged). |
| U4 | ok | - |
| U5 | ok | - |
| U6 | FIXED | Local overrides removed from Macros.svelte (.lnk, .field, .field .input, .hint, .tr.head, .sample-row, .rowmenu, .samples, textarea, .form-error, .empty .inline, .empty p). Kit is the one source; layout rules with no kit twin stay (.empty, .tbl, .tblbody, .form, .illustration). |
| T1 | FIXED | Light --t3 .76 (5.05 on surface, 5.31 on raised); dark .62 (5.52 on raised, 6.19 on surface). Computed from the token values. |
| T2 | ok | - |
| T3 | NOT FIXED (nit) | Light --acc-text on the plane is 4.29; link colour kept. |
| T4 | ok | - |
| T5 | NOT FIXED (nit) | Control edge alpha unchanged. |
| X-1 | FIXED | Segmented 24 (tray); controls 24. Keycap, checkbox and switch keep their own sizes by design. |
| X-2 | NOT FIXED (nit) | Radii unchanged. |
| X-3 | NOT FIXED (nit) | Font sizes unchanged. |
| X-4 | FIXED (stroke) | Tab and title-bar glyph strokes aligned (TB3). |
| X-5 | FIXED except About | Switch, segmented, language, theme, update dot, icon buttons, checkbox, toast close have 24 hit areas (CSS). NOT FIXED: About footer links and the "Vì sao" fold are 16 high. |
| X-6 | FIXED except switch hover | Button hover and press are overlays (B1, S2); seg, tab and language got press. Switch hover still none. |
| X-7 | FIXED | Focus ring on pri, ghost, icon, selected segment and language (F1, F2, G3, H4). |

Not checked this round: the real Windows app (the QA exe is built, not captured); macOS control rendering (CSS shared, not captured); native select popup colours.
