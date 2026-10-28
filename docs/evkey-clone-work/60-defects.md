# 60 — UI defects, round 3 (rebuilt shell)

**Date:** 2026-10-29 · **Owner:** QA · **Build:** `target\release\vietkb.exe`
(2026-10-29 03:41:02, SHA-256 `15BCDA2A3DB7576FEB459B7F54F60E5AAF73F1D67BA22F2301430B33C55F29F3`;
re-verified 2026-10-29 05:25:51 on SHA-256 `B12F9FCF2916BCCB10E464C613E0196F16D61A450AEAD409AE0A47DC39655757`).
**Scope:** failures from `50-test-cases.md` §3 (`UI-1…UI-46`), 2560 × 1440 @ 125 %.
No product code was changed by this pass. Size shown as P1 (feature unreachable) →
P3 (cosmetic); the Lead sets the final priority.

| id | severity | area | repro | expected | actual | evidence | status |
|---|---|---|---|---|---|---|---|
| D-UI-01 | P1 | Bảng mã (charset) removed from every UI surface | 1) open the main window and look for `Bảng mã`; 2) open the tray menu and look for `Bảng mã ▸` | the main window offers the charset picker (as the brief lists) **and** the tray offers `Bảng mã ▸` with the 6 verified charsets, both live-applied | neither exists: the main window has no `Bảng mã` label/select (`UI-11`), and the tray menu is `Bật/tắt tiếng Việt · Kiểu gõ · Mở cài đặt… · Phím tắt… · Thoát VietKB` with no charset submenu (`UI-23`, `UI-26`). `input.charset` can only be changed by editing `config.toml` | `qa/evidence/UI-11.txt`, `UI-23.txt`, `UI-26.txt`; `ui/src/screens/Quick.svelte` has no `PICKER_CHARSETS` import; `crates/vkb-app/src/tray.rs` builds only the method submenu | OPEN |
| D-UI-02 | P2 | language switch is a no-op (**FIXED + VERIFIED 2026-10-29** — the control was removed; see §Re-verification) | `Nâng cao` → `Ngôn ngữ hiển thị` → `English`; wait 1.5 s; read the main window | the visible strings switch to English (product terms such as `Kiểu gõ`, `Bảng mã`, `Gõ tắt` stay Vietnamese) with no missing-key placeholders | `system.language` becomes `"en"` and persists, but every label on every window stays Vietnamese (`Bật tiếng Việt`, `Kiểm tra chính tả`, `Đóng`, headings) — the switch has no visible effect | `qa/evidence/UI-17.txt`; `ui/src/lib/i18n.ts` dictionaries are never read by `Quick/Advanced/Hotkeys/Macros` | OPEN → **FIXED (control removed) / VERIFIED** |
| D-UI-03 | P2 | modifier-only hotkeys cannot be captured and bypass the conflict check (**FIXED + VERIFIED 2026-10-29**; see §Re-verification) | `Phím tắt` → focus `Đổi kiểu gõ` → press `Ctrl+Shift` (the toggle's chord) → Enter | the field records `Ctrl+Shift`, shows a `Trùng` warning naming `Bật/tắt tiếng Việt`, and does not commit | the field stays unchanged: `held` is filled on key-down but `onkeyup` resets `draft` to the stored value when the last modifier is released (`CaptureField.onkeyup`), so nothing is captured; no conflict check runs and nothing is persisted | `qa/evidence/UI-32.txt` (`fieldCaughtModifierOnly=False`, `config=""`, no warning) | OPEN → **FIXED / VERIFIED** |
| D-UI-04 | P2 | a hotkey conflict already in `config.toml` is invisible (**FIXED + VERIFIED 2026-10-29**; see §Re-verification) | stop the app; set `cycle_method = "Ctrl+Shift+Z"` (same as `restore_word`); start; open `Phím tắt` | both fields show a conflict state on load (the file is a supported way to edit settings) | both fields render `Ctrl+Shift+Z` with no `Trùng` marker; `check_hotkey_conflict` is only called while a field is being edited | `qa/evidence/UI-33.txt` (`cycleField=True`, `restoreField=True`, `warnings=`) | OPEN → **FIXED / VERIFIED** |
| D-UI-05 | P2 | `Kiểm tra chính tả` dependency is not live across windows (**FIXED + VERIFIED 2026-10-29**; see §Re-verification) | open `Nâng cao`, then turn `Kiểm tra chính tả` off in the main window and wait 1.6 s | `Tự động khôi phục từ gõ sai` disables within 1 s in the already-open window (and stays disabled when reopened) | the open window keeps the row enabled; only a reopened `Nâng cao` shows it disabled (each window holds its own settings snapshot; only `vkb:status` input fields are broadcast) | `qa/evidence/UI-34.txt` (`disabledLive=False`, `disabledAfterReopen=True`) | OPEN → **FIXED / VERIFIED** |
| D-UI-06 | P2 | macro table hides everything after the 8th row (**FIXED + VERIFIED 2026-10-29**; see §Re-verification) | write 12 macros to `config.toml`, start, open `Bảng gõ tắt` | all 12 rows are reachable (render them, or give the table a scroll area) | only 8 row buttons render (`rows.slice(0, 8)` in `Macros.svelte`), there is no scrollbar, and the footer admits `Đang hiện 8 trong 12`; entries 9–12 are only reachable through search | `qa/evidence/UI-40.txt` | OPEN → **FIXED / VERIFIED** |
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

## Re-verification of D-UI-02…06 (2026-10-29)

**Independent re-run by QA-subagent — the fix report was not trusted.** Build:
`target\release\vietkb.exe`, 2026-10-29 05:25:51 (+07:00), SHA-256
`B12F9FCF2916BCCB10E464C613E0196F16D61A450AEAD409AE0A47DC39655757`
(`npm --prefix ui run build` + `cargo build -p vkb-app --release --features custom-protocol`).
Method: same UIA/MSAA harness (`qa/lib.ps1`), 2560 × 1440 @ 125 %, no screenshots, no window moves;
the app was launched hidden and a second launch was used to show the window. Baseline
`config.toml` was restored byte-identical at the end (`Compare-Object` empty;
`system.language="vi"`, `restore_word="Ctrl+Shift+Z"`, `spell_check=true`, macros `m01`…`m12`).

| id | measurement (this run) | verdict |
|---|---|---|
| D-UI-02 | all windows enumerated for a text/combobox matching `Ngôn ngữ`: **0 controls**; `system.language` still `"vi"` | **PASS** — the dead i18n switch is gone, no language surface remains |
| D-UI-03 | focus `Đổi kiểu gõ`, hold then release `Ctrl+Shift` **with no Enter**: field label became `… phím tắt hiện tại Ctrl+Shift`, warnings `Trùng với Bật/tắt tiếng Việt — chọn tổ hợp khác.` (+ the mirror warning on the toggle row), `hotkeys.cycle_method` unchanged (still unset). Same chord on `Khôi phục từ gõ sai`: label `Ctrl+Shift`, same `Trùng … Bật/tắt tiếng Việt` warning, `restore_word` unchanged `Ctrl+Shift+Z` | **PASS** — modifier-only chords are captured and blocked by the conflict check |
| D-UI-04 | `cycle_method = "Ctrl+Shift+Z"` (= `restore_word`) written then app restarted: both fields present, **2 warnings on load** (`Trùng "Ctrl + Shift + Z" với Khôi phục từ gõ sai…` and `… với Đổi kiểu gõ…`); clearing the cycle field (Backspace+Enter) left **0 warnings** and `cycle_method` empty | **PASS** |
| D-UI-05 | `Nâng cao` left open the whole time: restore row `IsEnabled=True` with spell check on; toggled `Kiểm tra chính tả` off in the main window → row `IsEnabled=False` in the **same** window after 1.6 s (`spell_check=false` persisted); toggled back on → `True` again | **PASS** — dependency is live across windows |
| D-UI-06 | 12 macros in `config.toml`: **12 row buttons** render (`Sửa m*`), **9 visible** at 900 × 700 physical / 720 × 560 logical, 3 offscreen (`m7`, `m8`, `m9` at string order). `ScrollItemPattern.ScrollIntoView()` on `m7` → `IsOffscreen=False`, rows scroll (`m1/m10/m11` go offscreen); `Sửa m12` present and visible, and invoking `Sửa m7` opened its editor | **PASS** — every row reachable, footer no longer truncates at 8 |

No defect was reopened. Open items stay as before: D-UI-01 (charset surface) and D-UI-07 (VIQR),
neither in this scope.


## Closure — 2026-10-29
- **D-UI-01 closed as intentional**: charsets were removed by owner decision; UI-11/23/26 are N/A, not defects.
- **D-UI-02 … D-UI-06 fixed and independently re-verified** by QA on build B12F9FCF…: 43 pass / 3 N/A / 0 fail.
- **D-UI-07 closed as documentation**: the contract now lists Telex and VNI only.
