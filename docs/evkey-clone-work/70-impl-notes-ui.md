# Impl notes — wt-ui (Sprint 1)

Scope: `crates/vkb-app/**`, `ui/**`, plus the config fields this panel needs in
`crates/vkb-config/src/schema.rs` (marked below — contract proposals for the Lead).

## What the panel is

- **720 × 560, frameless, not resizable, no scrolling** at 100 % DPI. `tauri.conf.json`:
  width 720, height 560, minWidth 680, minHeight 520, `resizable: false`,
  `maximizable: false`, `decorations: false`, `shadow: true`, title `Cài đặt VietKB`.
  The 40 px strip is the caption bar (logo · 4 tabs · state chip on tabs 2–4 ·
  minimise/close; no maximise).
- Four tabs, no sidebar, no dashboard: `Gõ` (landing) · `Gõ tắt` · `Phím tắt` ·
  `Hệ thống`. `Ctrl+1…4` switches.
- Live apply unchanged: optimistic `patch_settings`, 150 ms debounce, reload on
  failure, toast. No Save button.
- C10 windows are **not** built: “Mở bảng gõ tắt”, the tray’s “Bảng gõ tắt…” and
  “Chuyển mã…” open this panel for now; the comment in the UI says the full table
  is a separate window later. No charset-conversion UI in the panel.

## Deleted (not dormant)

`Overview`, `Typing`, `Charset`, `Apps`, `About` screens; `ScreenHead`, `Card`,
`EmptyState`, `ResetButton`, `SettingRow`, `HotkeyField`, `Toasts`,
`ui/src/styles/controls.css` (superseded by `kit.css`).

## Design system

`ui/src/styles/tokens.css` = DESIGN-SPEC v3 colour ramps (light paper / dark ink,
mực tím accent, semantic ok/warn/bad, keycap/elevation/knob, focus, motion, type
families and the 4 px rhythm). `ui/src/styles/kit.css` = the mock’s anatomy.
Components, one file per control: `Row`, `Toggle`, `Keycap`, `CaptureField`,
`TabStrip`, `Segmented`, `Select`, `Toast`, `Icon`, `Modal`, `AppRulesDialog`.

## Copy: COPY.md + 27-copy-decisions (binding overrides win)

All visible strings on the four tabs are the deck’s verbatim strings, with these
overrides from `27-copy-decisions.md`:

- Off state is `Đã tắt tiếng Việt` (status chip, tray tooltip), never `English`.
- Reset actions say `Đặt lại…` / `Đặt lại tất cả`; `Khôi phục` appears only for
  “khôi phục từ gõ sai” and “khôi phục bộ gõ”.
- `Bảo vệ đường dẫn và mã` → `Giữ nguyên email, địa chỉ web`.
- Restart copy is `Cần mở lại VietKB, Windows sẽ hỏi xác nhận · Mở lại ngay`
  (the link calls the new `restart_app` command).
- `Tự động khôi phục từ gõ sai` keeps its dependency: when `Kiểm tra chính tả`
  is off the row is disabled, the note reads `Cần bật Kiểm tra chính tả` with an
  inline `Bật` link. Defaults ship spell check ON, restore OFF.
- `Nâng cao` holds **9** options including `Kiểu chuyển bật/tắt`
  (`Khi nhả phím` default / `Khi nhấn phím`), `Độ trễ giữa các phím` (0 µs,
  0–5000, step 500) and `Viết hoa theo từ tắt`.
- Charset pickers show the six verified charsets only; BK HCM1, VPS and the UTF-8
  literal are hidden but remain in the enum for config compatibility.
- Tray: tooltip `VietKB · Tiếng Việt · Telex · Unicode` / `VietKB · Đã tắt tiếng Việt`;
  menu adds `Bật gõ tắt`, `Kiểm tra chính tả`, `Bảng gõ tắt…`, `Mở cài đặt…`,
  `Chuyển mã…`, `Loại trừ ứng dụng này` (disabled until the active-app probe
  exists), `Thoát VietKB`.
- Macro import/export is TSV UTF-8 (`#vkb1`, `\n`/`\t` escapes, 5 000 rows,
  2 MB), all-or-nothing with the first bad line named.

## Schema additions in this worktree (flag for the Lead)

`crates/vkb-config/src/schema.rs` gained, with defaults so old configs still load:

| Field | Type | Default |
|---|---|---|
| `typing.auto_horn_uo` | bool | true |
| `typing.raw_zone` | bool | true |
| `typing.macro_case_follow` | bool | true |
| `typing.send_delay_us` | u32 | 0 |
| `hotkeys.toggle_mode` | `"release" \| "press"` | release |
| `hotkeys.cycle_charset` | hotkey | unbound |
| `hotkeys.cycle_method` | hotkey | unbound |

Also: hotkey defaults now match the design — `toggle = Ctrl+Shift`,
`restore_word = Ctrl+Shift+Z`, `open_settings = Ctrl+Alt+S`;
`UiConfig::default` is now 720 × 560 and `last_tab = "go"` (legacy values are
mapped to the four tabs). New command `restart_app` (additive) backs `Mở lại ngay`.
The engine/hook streams still have to read these flags; the UI persists them but
this build does not change typing behaviour from them yet.

## Layout deviation forced by C9

`Nâng cao` has nine options and must open in place without growing the window.
At 46 px per row it cannot fit (9 × 46 + status + strip + Thử gõ > 560), so the
advanced rows use the kit’s 40 px short row (content 38 px: 14/20 + 12/18). The
main eight rows stay at 46 px. This is the only way to satisfy “9 options, no
scroll, no growth”; recorded for the designer.

## Other technical constraints (and the copy they affect)

- The `Ctrl phải` preset is omitted: `Hotkey` has no side-specific modifier key
  (`RCtrl`), so it cannot be stored or matched. Presets shown: Ctrl+Shift,
  Alt+Z, Ctrl+` — all three are real.
- Import toast says `Đã nhập N mục`, without the deck’s “bỏ qua N mục trùng”:
  `import_macros` returns only the imported count and 27 C#8 makes the import
  all-or-nothing with duplicates overwritten.
- `Kiểm tra ngay` shows the deck’s failure line (`Chưa kiểm tra được, có thể máy
  đang mất mạng`) because there is no updater endpoint; the toggle persists.
- `Windows đã dùng Ctrl + Shift + Esc để mở Task Manager. Hãy nhấn tổ hợp khác.`
  is new wording: 27 C#7 reserves that chord but COPY.md only ships the other
  three sentences.
- The macro `…` menu is the two-item popover required by 27 C#13 (`Nhân bản`,
  `Xoá`), implemented with the `Dễ trùng`/`Trùng` tag split from COPY §3.3.
- The Thử gõ strip still renders through the local Telex approximator
  (`ui/src/lib/preview.ts`), not the engine; the strip is memory-only and clears
  when the window closes (C5).

## Verification

| Command | Result |
|---|---|
| `npm.cmd run check` | 0 errors, 0 warnings |
| `npm.cmd run build` | passes (143 modules) |
| `cargo test -p vkb-config` | 9/9 |
| `cargo test -p vkb-app` | 11/11 (aggregate from the test runner; no failures) |
| `cargo check -p vkb-app` | passes |
| `ui/dist` | 162,482 bytes (8 files incl. 4 review wrappers); JS 134,362 B, CSS 24,605 B |

Fit, measured on the built dist:

- **720 × 560:** all four tabs fit with no scrollbar; the 9-row `Nâng cao` view
  also fits (status row and Thử gõ stay put).
- **125 % / 150 %:** Tauri sizes are logical pixels, so the webview stays
  720 × 560 CSS px; the scale wrappers (iframe at 720 × 560, `scale(1.25|1.5)`)
  show no reflow and nothing clipped.
- **680 × 520 (configured minimum):** the Gõ tab’s Thử gõ strip clips by ~12 px
  because eight 46 px rows + status + disclosure exceed the sheet. The window is
  fixed at 720 × 560 with `resizable:false`, so this size is unreachable in the
  shipped app; if the minimum must be usable, the design needs a compact-row
  variant.

## 2026-10-28 runtime fix after the merge: updater plugin removed (dev)

The merged app panicked at startup: PluginInitialization("updater", "Error
deserializing 'plugins.updater' ... invalid type: null, expected struct Config")
— the plugin was registered in main.rs with no `plugins.updater` block in
`tauri.conf.json`. Update checking is Sprint 4 (charter §1), so the registration
was dropped instead of shipping a fake endpoint/pubkey:

- `crates/vkb-app/src/main.rs`: removed `.plugin(tauri_plugin_updater::Builder::new().build())`.
- `crates/vkb-app/Cargo.toml`: removed the `tauri-plugin-updater = "2"` dependency
  (`Cargo.lock` follows on build).
- `crates/vkb-app/capabilities/default.json`: removed the `updater:default` permission.

No command, schema field or UI change: `system.check_updates` stays in the config and
the System tab keeps its "chưa hỗ trợ" behaviour (`Kiểm tra ngay` shows the offline
message), so the flag is harmless until Sprint 4 wires it up.

Verified: `cargo build -p vkb-app` pass; `target\debug\vietkb.exe` stays alive 8 s
with `MainWindowTitle = VietKB` and an empty stderr. Tray icons checked:
`icons/tray-on.png` and `icons/tray-off.png` both exist, are valid 32×32 PNGs;
`tray-on.png` is byte-identical to `icons/32x32.png` (same hash) — cosmetic only.

## 2026-10-28 release build: assets were never embedded (dev)

crates/vkb-app/Cargo.toml had no Tauri custom-protocol feature, so a release
build kept pointing at `build.devUrl` and the window showed "localhost refused to
connect". Added (deliberately **not** a default feature, so `tauri dev` still uses
`devUrl`):

```toml
[features]
custom-protocol = ["tauri/custom-protocol"]
```

**Working release binary:**
`cargo build -p vkb-app --release --features custom-protocol`
→ `target\release\vietkb.exe`. A plain `--release` without the flag still shows the
devUrl error page, so every release/NSIS build must pass it (the Tauri CLI does this
automatically via its own `--features custom-protocol`; bare `cargo` does not).

Verified: build exit 0; exe contains the hashed `ui/dist` asset names and
`tauri://localhost`; process alive 8 s, `MainWindowTitle = VietKB`, stderr empty;
UI Automation over the window reads the real panel (tabs Gõ / Gõ tắt / Phím tắt /
Hệ thống, «Bật/tắt tiếng Việt (Ctrl + Shift)», «Kiểu mới · hoà, thuý»), i.e. the
embedded UI rendered, not an error page.
