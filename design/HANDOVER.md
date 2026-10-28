# Design handover — 720 × 560 quick-settings panel

**Source of truth:** `design/DESIGN-SPEC.md` + `design/light/*.html` + `design/dark/*.html`
(4 screens, exactly 720 × 560, rendered and checked at that size).
Screenshots for comparison: `design/_shots/{light,dark}/*.png`.

Direction: "mực tím" — accent `#5844C8` light / `#6B57D8` dark, warm paper neutrals (light),
cool violet-grey neutrals (dark), frameless window whose status strip is the title bar,
keycaps recessed in a well as the one recurring signature, Sitka for produced Vietnamese text
and Segoe UI Variable for UI text. No uppercase anywhere.

## 1. Tabs → contract commands

| Tab | Content | Commands used |
|---|---|---|
| `Gõ` (default) | toggle + kiểu gõ + bảng mã + 8 daily options + `Nâng cao` + one-line thử gõ | `get_settings`, `set_settings`/`patch_settings`, `toggle_enabled`, `set_method`, `set_charset`, `reset_settings` |
| `Gõ tắt` | toolbar (search / Thêm / Nhập / Xuất), 8 rows, bulk bar replacing the header, footer link to the full window | `list_macros`, `upsert_macro`, `delete_macros`, `import_macros`, `export_macros` |
| `Phím tắt` | 6 rows, presets row, capture field, conflict line, reset-all footer | `parse_hotkey`, `check_hotkey_conflict`, `patch_settings` |
| `Hệ thống` | autostart, admin, sound, language, theme, updates, 3-row exclusion list | `patch_settings`, `list_app_rules`, `list_running_apps`, `set_app_rule`, `delete_app_rule`, `open_config_folder`, `app_info` |

Nothing outside this table may be added to the panel. Anything bigger (full macro table,
charset conversion tool) is a **separate window** — charter C10.

## 2. Corrections to apply while implementing (the mocks are not the spec here)

1. **Ship real defaults on the `Gõ` tab.** Config default is `spell_check = true`,
   `restore_wrong_word = false`. The mocks turn spell check **off** only to demonstrate the
   disabled dependency (`Tự động khôi phục` greyed + "Cần bật Kiểm tra chính tả · Bật").
   Keep the disabled state as a live behaviour, never as the first screen the user sees.
2. **`Nhả để chuyển` is cryptic.** That select is the toggle mode. Render it as a labelled
   control (`Chế độ` + options `Nhả phím để chuyển` / `Nhấn để chuyển`) or move it into
   `Hệ thống → Nâng cao`. A bare two-word select next to keycaps is not understandable.
3. **Dark "chạy với quyền Admin" toggle** has ~2:1 knob-to-danger contrast (designer's own
   note). Use `--danger` for the track and a light knob, verify ≥ 3:1 for the track against
   the row surface, and never rely on colour alone: the row carries a warning icon + text.
4. **`Phím tắt` has ~20 px dead space above the footer.** Distribute it into the section gaps;
   do not grow the footer.
5. **Tab strip state:** the live state chip appears once. On `Gõ` the state lives in the status
   row; on the other three tabs it is a small chip in the tab strip. Do not show both.

## 3. Accessibility floor (verify in the build, not in the mock)

- Focus ring 2 px accent + 2 px offset on every interactive element; login order = visual order.
- Toggles are `role="switch"` + `aria-checked`; segmented controls use `aria-pressed`.
- Body text ≥ 4.5:1, UI boundaries ≥ 3:1, both themes. `DESIGN-SPEC.md` §contrast is the table.
- Vietnamese text: line-height ≥ 1.5 for body, never uppercase labels (`GÕ TẮT` clips marks).
- `prefers-reduced-motion: reduce` disables all animation and the capturing caret pulse.
- No scroll at 100 % DPI on any tab; at 125/150 % the panel must still fit — test it.

## 4. Do not build

Sidebar · dashboard/Tổng quan · separate Bảng mã screen · separate Loại trừ screen ·
first-run wizard · a 148-row table inside the panel · a maximise button (window is fixed size).
