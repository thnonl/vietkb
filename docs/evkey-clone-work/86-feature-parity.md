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
| `ưo → ươ` tự động (spec A1) | `typing.auto_horn_uo` | **covered** | engine + tests `auto_horn_uo_*` |
| Vùng gõ thô URL/code (spec A9) | `typing.raw_zone` | **covered** | engine: opener chờ phím kế tiếp, bỏ qua ellipsis/digit-dot, Backspace xoá opener; tests `raw_zone_*` |
| Macro theo hoa/thường của trigger (spec A8) | `typing.macro_case_follow` | **covered** | engine + E2E A12/A13 |
| Gõ tắt khi đã tắt tiếng Việt | `typing.macro_when_disabled` | **covered** | engine macro-only mode + backend route; tests `macro_only_*` |
| Sửa lỗi dấu gợi ý 1/2 (Browsers, Excel) | toggle removed; hành vi sẽ nằm trong bảng app | **removed** | owner: bật/tắt trên Chrome không đổi gì; EVKey áp dụng cho Excel/Sheets |
| Hỗ trợ ứng dụng Metro | toggle removed | **removed** | không kiểm chứng được; bảng app sẽ xử lý nếu cần |
| Delay giữa các lần gửi | — | removed by owner | implementation detail, không phải khái niệm người dùng |
| Gửi bằng scancode (Standard) | `typing.send_standard` | covered | `vkb-windows/src/state.rs:164` |
| Clipboard fallback | `typing.use_clipboard` | covered | `state.rs:169,441`, test `send_strategy_prefers_clipboard_then_standard_then_unicode` |
| Bàn phím ảo Windows (`VK_PACKET`) | packet được decode và đưa qua engine | **covered** | unit tests `packet_w_transforms_with_empty_word`, `plain_packet_with_empty_word_is_noop`, `control_packet_is_not_text` |
| Vẫn gõ khi dùng layout khác (IME Nhật) | toggle removed; gõ được ở mọi layout | **covered** (theo lựa chọn owner) | `disable_on_non_us_layout` đã gỡ |
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
| Toggle theo press/release | `hotkeys.toggle_mode` chọn lúc nhấn hay lúc nhả | **covered** | `FireMode` trong `vkb-platform`, test `release_mode_*`, `default_config_fires_on_release` |
| Chín action: enable, disable, toggle, restore word, exclude app, macro panel, macro toggle, settings, cycle method | all nine dispatched | covered, **untested per action** |

## 3. Macros

| EVKey | VietKB | Status |
|---|---|---|
| `evkmacro.txt` file | TOML `[macros]` table + UI editor, txt/tsv import-export | covered (different storage, on purpose) |
| Trigger dài, expansion nhiều dòng | trigger ≤ `MAX_WORD` (32); expansion unbounded; `\n`/`\r\n` được gõ thành `Return` | **covered** | test `macro_newline_is_typed_as_return`; UI Enter xuống dòng được |
| Commit bằng Enter | `VK_RETURN` commits then appends `\r` | covered |
| Hoa/thường theo trigger | follows the trigger when `macro_case_follow` is on (default), verbatim when off | covered, E2E A12/A13 |
| Panel F8, bật/tắt F9 | both dispatched | covered, untested |

## 4. Per-app and exclusions

| EVKey | VietKB | Status |
|---|---|---|
| Loại trừ ứng dụng | `apps.rules` by exe basename, overrides enabled/method/charset/macro | covered, **no UI test** |
| Nhớ trạng thái theo app | in-memory `HashMap`, lost on restart | **partial** |
| Thêm loại trừ tự động | app từ chối batch → Standard, rồi loại trừ | **covered** | `on_short_send`, test `short_send_switches_to_standard_then_excludes` |
| Charset/macro override trong hộp thoại app | rule model has them, dialog does not expose them | partial |
| App có quyền Admin | — | **removed by owner** | không có luồng elevation; toggle đã gỡ |

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

## 7. Plan (as of 2026-10-29 evening)

**Done in this wave.** `auto_horn_uo` implemented with tests. `run_as_admin`,
`suggestion_fix_1/2`, `metro_fix`, `disable_on_non_us_layout` and `send_delay_us` removed from
config, IPC and UI; old `config.toml` files still load (migration test
`removed_settings_are_ignored_and_round_trip_without_clobbering_defaults`). `raw_zone` pulled
back to a reserved, unimplemented field with no UI. About window: missing Tauri capability fixed
(`about` added, tray item added, test `every_window_has_a_capability`). `macro_case_follow` wired
and proven. Two distinct toggle sounds (880 Hz on / 520 Hz off). **Macro table now actually
reaches the engine** — it never did, so gõ tắt was dead at runtime; found by E2E, fixed in
`vkb-windows/src/state.rs`. Host E2E: 12/12 typing cases + 20/20 config-matrix cases
(`88-e2e-report.md`).

**Still to do, in this order.**
1. Built-in application table (invisible): per-exe send strategy and the Excel/Google-Sheets
   suggestion fix that `CSuggestion`/`CSuggestion2` actually targeted.
2. Persist per-app remembered on/off to config (EVKey's `blacklist`).
3. `VK_PACKET` — Windows virtual/touch keyboard events must reach the engine.
4. Auto-exclusion — done (short send → Standard → exclude, session-scoped).
5. Multiline macro typing — done (newline sends Return, UI edits multiline).
6. `raw_zone` — done (tests-first: `crates/vkb-core/tests/raw_zone.rs`).
7. `macro_when_disabled` — done (engine macro-only mode, tests-first).
8. `hotkeys.toggle_mode` — done (press/release wired, default release).
9. Verification debt left: each hotkey action, per-app rules, macro F8/F9.

## 8. What the resource strings add (found after the first matrix)

EVKey's own config vocabulary, read from its resources: `setting`, `blacklist`,
`user_method_input`, `\setting.ini`, `SGui`, `SInput`, `RAdmin`, `DisFOK`, `CSuggestion`,
`CSuggestion2`, `ICCharset`, `CUnicode`, `CTIcon`.

| EVKey key | meaning | VietKB |
|---|---|---|
| `blacklist` | per-app exclusion list, persisted in `setting.ini` | `apps.rules` exists; remembered on/off is session-only |
| `user_method_input` | **custom input method**: the user maps keys to transforms, stored as rules | **missing** |
| `RAdmin` | run as administrator | removed by owner (no elevation flow) |
| `DisFOK` | disable Vietnamese when another keyboard layout is active | owner chose EVKey's behaviour: keep typing |
| `CSuggestion`, `CSuggestion2` | the two suggestion fixes | toggles removed; the *behaviour* should be built in per app instead |
| `ICCharset`, `CUnicode` | charset choice | dropped by design |
| `CTIcon` | tray icon style | we ship V/E only, on purpose |

Built-in application table (exe names found side by side in the resource area): `misa sme.net`,
`epic.exe`, `sfive.exe`, `slimjet.exe`, `- microsoft excel`, `safari.exe`, `- google trang tính`,
`iexplore.exe`, `microsoftedge.exe`, `firefox.exe`, `vivaldi.exe`, `kinza.exe`, `terminal.exe`,
`viber.exe`, `emeditor.exe`, `mindmanager.exe`, `audition.exe`, `crossfire.dat`,
`winuapentry.exe`. EVKey ships **invisible per-app fixes** for these — the user never sees a
toggle. That is why `CSuggestion` exists: it targets Excel and Google Sheets, not Chrome.

Transform vocabulary in the same area (the custom-method editor): `Xoá dấu`,
`Dấu Sắc/Huyền/Hỏi/Ngã/Nặng`, `Dấu mũ`, `Dấu móc`, `Dấu gạch d`, `Char ă/â/đ/ê/ô/ơ/ư` in both
cases, `Horn for uo becomes ươ`, `Horn for a, u, o … or Char ư except start of word`.
So EVKey's engine is not "Telex + VNI": it is a **rule table** the user can edit.

## 9. Owner decisions, second round (2026-10-29)

1. **Custom input method (`user_method_input`): dropped.** Telex and VNI cover the real use;
   a rule-table editor is a large surface (schema, engine, editor UI, migration) for a feature
   the owner does not want to maintain. Recorded here so it is not rediscovered as a "gap".
2. **Built-in application table: to be implemented, invisible.** Per-exe behaviour for the apps
   in EVKey's list (Excel, Google Sheets, browsers, Viber, EmEditor, MindManager, Audition,
   terminal, the games, `winuapentry.exe`), with no user-facing toggle. This is where the two
   "suggestion fixes" belong, which is why removing those toggles loses nothing.
3. **Per-app remembered state: persist to config**, like EVKey's `blacklist`, instead of dying
   with the session.

## 10. Open questions for the owner

1. EVKey's `suggestion_fix` mechanism is `UNVERIFIED` in our spec; do we run a real Chrome/Excel
   experiment (real keyboard, since EVKey ignores injected keys) before implementing?
2. `run_as_admin`: ship an elevation flow, or drop the toggle?
3. Two distinct toggle sounds: keep system beep, or bundle two short sounds like EVKey?
4. `disable_on_non_us_layout` — EVKey keeps typing under a Japanese IME; we disable. Which way?
