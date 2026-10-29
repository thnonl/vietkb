# 86 — Feature parity: EVKey → VietKB

Sources, in order of authority: decompiled `EVKey64.exe` (sha256 `69274b44…`, Ghidra 12.1.4,
`reference/evkey-decomp/`), EVKey's own changelog strings embedded in the binary, the VietKB
repo at `2795555`, and `00-charter.md`. Nothing here comes from memory of using EVKey.

Status vocabulary:

- **covered** — implemented, and a test or the binary contradicts nothing.
- **partial** — implemented but a documented part is missing.
- **inert** — the config key and the UI control exist, the engine/backend never reads it:
  the toggle lies to the user.
- **missing** — EVKey has it, we have neither config nor behaviour.
- **dropped** — intentionally removed by owner decision (`log.md`, 2026-10-29).

## 1. Engine and typing

| EVKey | VietKB | Status | Evidence |
|---|---|---|---|
| Telex, VNI | both | covered | `vkb-core/src/lib.rs:221-235` |
| VIQR | — | dropped | charter C-list, `log.md` |
| Bảng mã tự động (auto charset) | — | dropped | needs legacy charsets |
| TCVN3/ABC fonts, VISCII, VNI-Windows | — | dropped | same |
| Kiểu cũ / kiểu mới (tone placement) | `typing.modern_tone` | covered, **untested** | `vkb-core/src/lib.rs:250`, `tone.rs` |
| Bỏ dấu tự do (free tone) | `typing.free_tone` | covered, **untested** | `vkb-core/src/lib.rs:260` |
| Kiểm tra chính tả | `typing.spell_check` | covered | `engine.rs:424-426` |
| Tự khôi phục từ gõ sai | `typing.restore_wrong_word` | covered | `engine.rs` restore path |
| Viết hoa đầu câu | `typing.auto_capitalize` | covered (default `false`) | `engine.rs:347`, test `auto_capitalize_after_a_sentence_end` |
| j/f/w/z là phụ âm | `typing.jfwz_as_consonant` | covered | test `jfwz_off_lets_the_tone_keys_win` |
| `ưo → ươ` tự động (spec A1) | `typing.auto_horn_uo` | **inert** | only `schema.rs:111`, `commands.rs:222` |
| Vùng gõ thô URL/code (spec A9) | `typing.raw_zone` | **inert** | only `schema.rs:113`, `commands.rs:223` |
| Macro theo hoa/thường của trigger (spec A8) | `typing.macro_case_follow` | **inert**, behaviour is hard-coded | `engine.rs:408` ignores the flag |
| Gõ tắt khi đã tắt tiếng Việt | `typing.macro_when_disabled` | **inert** | documented as not implementable today, `70-impl-notes-windows.md:176` |
| Sửa lỗi dấu gợi ý 1/2 (Browsers, Excel) | `typing.suggestion_fix_1/2` | **inert** | spec E3/E4; no consumer outside config |
| Hỗ trợ ứng dụng Metro | `typing.metro_fix` | **inert** | `schema.rs:101` |
| Delay giữa các lần gửi | `typing.send_delay_us` | **inert** | no `SendInput` delay wired |
| Gửi bằng scancode (Standard) | `typing.send_standard` | covered | `vkb-windows/src/state.rs:164` |
| Clipboard fallback | `typing.use_clipboard` | covered | `state.rs:169,441`, test `send_strategy_prefers_clipboard_then_standard_then_unicode` |
| Bàn phím ảo Windows (`VK_PACKET`) | — | **missing** | `vkb-windows/src/state.rs:227-236` passes `VK_PACKET` through |
| Vẫn gõ khi dùng layout khác (IME Nhật) | `typing.disable_on_non_us_layout` does the opposite | **different** | `foreground.rs:113-125` |
| Backspace xoá 1 ký tự hiển thị | same | covered | binary B1, `28-engine-rulings.md` |
| Mở lại từ sau separator | same | covered | binary B4/D8, test `reopening_covers_separators_but_not_line_breaks` |

## 2. Hotkeys

| EVKey | VietKB | Status |
|---|---|---|
| F1..F24, phím số, ký tự đặc biệt | grammar accepts all of them | covered |
| Ctrl / Shift / Alt / Win | all four, exact-match | covered |
| Chỉ modifier (`Ctrl+Shift`) | supported, fires on chord completion | covered |
| F1/F2 cố định bật/tắt | defaults F1/F2, rebindable | covered |
| Hai binding cho một action | one binding per action | **unverified whether EVKey really has 2** — no code evidence found |
| Toggle theo press/release | `hotkeys.toggle_mode` stored; matcher latches on key-down | **inert** |
| Chín action: enable, disable, toggle, restore word, exclude app, macro panel, macro toggle, settings, cycle method | all nine dispatched | covered, **untested per action** |

## 3. Macros

| EVKey | VietKB | Status |
|---|---|---|
| `evkmacro.txt` file | TOML `[macros]` table + UI editor, txt/tsv import-export | covered (different storage, on purpose) |
| Trigger dài, expansion nhiều dòng | trigger ≤ `MAX_WORD` (32); expansion unbounded; core can emit `\n` | **partial** — UI editor Enter does not commit a newline, and typing `\n` into a target app is unverified |
| Commit bằng Enter | `VK_RETURN` commits then appends `\r` | covered |
| Hoa/thường theo trigger | always follows, flag ignored | partial (see `macro_case_follow`) |
| Panel F8, bật/tắt F9 | both dispatched | covered, untested |

## 4. Per-app and exclusions

| EVKey | VietKB | Status |
|---|---|---|
| Loại trừ ứng dụng | `apps.rules` by exe basename, overrides enabled/method/charset/macro | covered, **no UI test** |
| Nhớ trạng thái theo app | in-memory `HashMap`, lost on restart | **partial** |
| Thêm loại trừ tự động | — | **missing** |
| Charset/macro override trong hộp thoại app | rule model has them, dialog does not expose them | partial |
| App có quyền Admin | `system.run_as_admin` | **inert** (no elevation path) |

## 5. Shell, tray, update

| EVKey | VietKB | Status |
|---|---|---|
| Tray: bật/tắt, kiểu gõ, cài đặt, phím tắt, thoát | same plus update item | covered |
| Bảng mã ▸ trong tray | — | dropped |
| Công cụ chuyển mã văn bản | — | dropped (charter C10 still mentions the dialog; superseded) |
| Âm thanh khi chuyển E/V (`evkson.mp3`/`evksoff.mp3`) | `system.notification_sound` → system beep on toggle | **partial** (one beep, not two distinct sounds) |
| Khởi động cùng Windows | `system.start_with_windows` + HKCU Run | covered |
| Chạy ẩn | `system.start_minimized` | covered (consumer in `vkb-app`) |
| Kiểm tra cập nhật | `system.check_updates` + GitHub release + changelog + install | covered |
| Tự động cập nhật qua `evkau.exe` + task | in-app check only, no scheduled task | **different** (deliberate: no separate updater binary) |

## 6. Dropped by design (not gaps)

Legacy charsets (TCVN3, VNI-Windows, VISCII, BK HCM1, VPS, UTF-8), VIQR, clipboard text
converter, tray charset submenu, Linux/cloud/telemetry. Recorded in `log.md` 2026-10-29.

## 7. Plan

**Wave 1 — stop lying: wire or remove every inert toggle.** Order by risk:
1. `auto_horn_uo` (spec A1) — engine, tests.
2. `raw_zone` (spec A9) — engine, tests.
3. `macro_case_follow` (spec A8) — engine, tests, and make the hard-coded behaviour the default.
4. `send_delay_us` — backend, respects 0 = off.
5. `metro_fix` — backend compatibility path (needs a definition before code).
6. `suggestion_fix_1/2` — browser autocomplete dismissal; spec marks the mechanism `UNVERIFIED`, so it needs a real Chrome experiment before code.
7. `run_as_admin` — elevation relaunch + manifest, or remove the toggle.
8. `macro_when_disabled` — needs the backend to call the engine while Vietnamese is off.
9. `hotkeys.toggle_mode` — either implement press/release or drop the control.

**Wave 2 — EVKey features we simply do not have.**
1. `VK_PACKET` (Windows virtual keyboard / touch keyboard) — feed those events through the engine.
2. Auto-exclusion: detect an app where injection fails and fall back (clipboard or standard) per app, without the user adding a rule.
3. Persist remembered per-app state across restarts.
4. Multiline macro editing that actually types newlines into the target app.
5. Two distinct toggle sounds (on/off), opt-in.

**Wave 3 — close the verification gaps.** `modern_tone`, `free_tone`, each hotkey action,
per-app rules, macro F8/F9. Add asserting tests; the audit found none.

## 8. Open questions for the owner

1. EVKey's `suggestion_fix` mechanism is `UNVERIFIED` in our spec; do we run a real Chrome/Excel
   experiment (real keyboard, since EVKey ignores injected keys) before implementing?
2. `run_as_admin`: ship an elevation flow, or drop the toggle?
3. Two distinct toggle sounds: keep system beep, or bundle two short sounds like EVKey?
4. `disable_on_non_us_layout` — EVKey keeps typing under a Japanese IME; we disable. Which way?
