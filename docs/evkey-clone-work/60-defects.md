# 60 — UI defects, round 3 (rebuilt shell)

**Date:** 2026-10-29 · **Owner:** QA · **Build:** `target\release\vietkb.exe`
(2026-10-29 03:41:02, SHA-256 `15BCDA2A3DB7576FEB459B7F54F60E5AAF73F1D67BA22F2301430B33C55F29F3`).
**Scope:** failures from `50-test-cases.md` §3 (`UI-1…UI-46`), 2560 × 1440 @ 125 %.
No product code was changed by this pass. Size shown as P1 (feature unreachable) →
P3 (cosmetic); the Lead sets the final priority.

| id | severity | area | repro | expected | actual | evidence | status |
|---|---|---|---|---|---|---|---|
| D-UI-01 | P1 | Bảng mã (charset) removed from every UI surface | 1) open the main window and look for `Bảng mã`; 2) open the tray menu and look for `Bảng mã ▸` | the main window offers the charset picker (as the brief lists) **and** the tray offers `Bảng mã ▸` with the 6 verified charsets, both live-applied | neither exists: the main window has no `Bảng mã` label/select (`UI-11`), and the tray menu is `Bật/tắt tiếng Việt · Kiểu gõ · Mở cài đặt… · Phím tắt… · Thoát VietKB` with no charset submenu (`UI-23`, `UI-26`). `input.charset` can only be changed by editing `config.toml` | `qa/evidence/UI-11.txt`, `UI-23.txt`, `UI-26.txt`; `ui/src/screens/Quick.svelte` has no `PICKER_CHARSETS` import; `crates/vkb-app/src/tray.rs` builds only the method submenu | OPEN |
| D-UI-02 | P2 | language switch is a no-op | `Nâng cao` → `Ngôn ngữ hiển thị` → `English`; wait 1.5 s; read the main window | the visible strings switch to English (product terms such as `Kiểu gõ`, `Bảng mã`, `Gõ tắt` stay Vietnamese) with no missing-key placeholders | `system.language` becomes `"en"` and persists, but every label on every window stays Vietnamese (`Bật tiếng Việt`, `Kiểm tra chính tả`, `Đóng`, headings) — the switch has no visible effect | `qa/evidence/UI-17.txt`; `ui/src/lib/i18n.ts` dictionaries are never read by `Quick/Advanced/Hotkeys/Macros` | OPEN |
| D-UI-03 | P2 | modifier-only hotkeys cannot be captured and bypass the conflict check | `Phím tắt` → focus `Đổi kiểu gõ` → press `Ctrl+Shift` (the toggle's chord) → Enter | the field records `Ctrl+Shift`, shows a `Trùng` warning naming `Bật/tắt tiếng Việt`, and does not commit | the field stays unchanged: `held` is filled on key-down but `onkeyup` resets `draft` to the stored value when the last modifier is released (`CaptureField.onkeyup`), so nothing is captured; no conflict check runs and nothing is persisted | `qa/evidence/UI-32.txt` (`fieldCaughtModifierOnly=False`, `config=""`, no warning) | OPEN |
| D-UI-04 | P2 | a hotkey conflict already in `config.toml` is invisible | stop the app; set `cycle_method = "Ctrl+Shift+Z"` (same as `restore_word`); start; open `Phím tắt` | both fields show a conflict state on load (the file is a supported way to edit settings) | both fields render `Ctrl+Shift+Z` with no `Trùng` marker; `check_hotkey_conflict` is only called while a field is being edited | `qa/evidence/UI-33.txt` (`cycleField=True`, `restoreField=True`, `warnings=`) | OPEN |
| D-UI-05 | P2 | `Kiểm tra chính tả` dependency is not live across windows | open `Nâng cao`, then turn `Kiểm tra chính tả` off in the main window and wait 1.6 s | `Tự động khôi phục từ gõ sai` disables within 1 s in the already-open window (and stays disabled when reopened) | the open window keeps the row enabled; only a reopened `Nâng cao` shows it disabled (each window holds its own settings snapshot; only `vkb:status` input fields are broadcast) | `qa/evidence/UI-34.txt` (`disabledLive=False`, `disabledAfterReopen=True`) | OPEN |
| D-UI-06 | P2 | macro table hides everything after the 8th row | write 12 macros to `config.toml`, start, open `Bảng gõ tắt` | all 12 rows are reachable (render them, or give the table a scroll area) | only 8 row buttons render (`rows.slice(0, 8)` in `Macros.svelte`), there is no scrollbar, and the footer admits `Đang hiện 8 trong 12`; entries 9–12 are only reachable through search | `qa/evidence/UI-40.txt` | OPEN |
| D-UI-07 | P2 | VIQR is no longer selectable (contract deviation, observation) | read the `Kiểu gõ` options and the tray submenu | the contract lists three methods (Telex, VNI, VIQR); both surfaces should offer them | both offer Telex and VNI only (`UI-10` passes against the 2-item catalogue; `UI-25` shows 2 tray items); `vkb-core` still implements VIQR | `qa/evidence/UI-10.txt`, `UI-25.txt`; `ui/src/lib/catalog.ts` `METHODS` has 2 entries | OPEN |

## Notes on the three non-defect observations

* `UI-24` tooltip evidence shows two strings concatenated
  (`VietKB · Đã tắt tiếng Việt VietKB · Tiếng Việt · Telex`). This is a **shell ghost icon** left by
  force-killing the app between harness groups (Windows keeps the dead icon until the flyout is
  refreshed); it also makes the automation see two buttons. A normal `Thoát VietKB` leaves one icon
  (verified by `UI-28`). Not a product defect.
* `UI-39` failed intermittently only when the native “Open” dialog lost foreground to the macro
  window; after focusing the dialog itself the round-trip is stable
  (export 4 lines → delete `ko` → import → `Đã nhập 3 mục`). Harness issue, not a UI defect.
* `UI-8`/`UI-44` are measured at 125 % only: the display scaling was not changed, and the focus-ring
  contrast was read from `tokens.css`, not measured on screen.
