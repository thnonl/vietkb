# VietKB Settings Parity Plan — Unikey-Based

Date: 2026-07-04

Scope: implement missing runtime behavior for settings already present in VietKB config/UI, using Unikey source behavior as reference where applicable. Do not add large new feature families unless needed to make current settings truthful.

Base: Unikey - https://github.com/hochanh/unikey-source

References inspected:

- Unikey option struct: `x-unikey/src/ukengine/keycons.h` (`freeMarking`, `modernStyle`, `macroEnabled`, `useUnicodeClipboard`, `alwaysMacro`, `strictSpellCheck`, `useIME`, `spellCheckEnabled`, `autoNonVnRestore`).
- Unikey engine tone/spell/restore paths: `x-unikey/src/ukengine/ukengine.cpp`.
- Unikey Win hook/shared options: `src-win/keyhook/keyhook.cpp`, `src-win/keyhook/vietkey.cpp`.
- VietKB config/UI/runtime: `crates/vietkb-config/src/lib.rs`, `crates/vietkb-gui/dist/index.html`, `crates/vietkb-gui/src/main.rs`, `crates/vietkb-windows/src/hook.rs`, `crates/vietkb-core/src/engine.rs`.

## Current settings inventory

### Input settings

| Setting | Current status | Gap |
|---|---:|---|
| `input.method` Telex/VNI/VIQR | Runtime wired | Missing Simple Telex/MS Vietnamese/user keymap, but not currently exposed. |
| `input.charset` Unicode/VNI Windows/TCVN3 | UI/config/tray wired | Runtime engine only emits Unicode; VNI Windows/TCVN3 are not applied. |
| `input.enabled` | Runtime wired | Per-app effective enabled not applied. |

### Typing settings

| Setting | Current status | Gap |
|---|---:|---|
| `old_oa_uy` | Partially wired to Telex tone placement | Need verify full Unikey cases: `oa`, `oe`, `uy`, `qu`, `gi`, trailing consonants, uppercase. |
| `smart_mark` | Partially wired | Semantics currently inverted/too simple vs Unikey `freeMarking`; needs exact scanning window behavior. |
| `use_j_f_w_as_consonants` | Config/UI only | Engine still treats Telex tone keys `f/j` as tone keys universally. |
| `auto_capitalize_after_punctuation` | Config/UI only | Engine/hook does not auto capitalize after Enter/`.!?:`. |
| `use_clipboard_to_send_keys` | Config/UI only | Windows hook always uses Unicode `SendInput`; no clipboard send path. |
| `spell_check` | Partially wired | Current dictionary bypass is simplistic; no strict spell validation like Unikey. |
| `restore_wrong_word` | Config/UI only | No restore keystrokes / auto restore non-VN behavior. |
| `us_keyboard_layout_when_off` | Config/UI only | Hook does not force/switch US layout when Vietnamese off. |
| `suggestion_fix_1` | Config/UI only | No browser/suggestion workaround. |
| `suggestion_fix_2` | Config/UI only | No browser/suggestion workaround. |
| `skip_english_words` | Runtime wired | Needs integrate with `spell_check`/restore policy; current word list only. |

### Hotkey settings

| Setting | Current status | Gap |
|---|---:|---|
| `hotkeys.toggle` | Runtime wired | OK for modifier combo + optional key. |
| `hotkeys.restore_word` | UI/config only | Hook never handles restore hotkey. |
| `hotkeys.function_keys.ctrl/shift/alt/win` | Config only | Hook never handles function-key combos. |
| `f1_toggle_vietnamese_on` | Config only | Missing. |
| `f2_toggle_vietnamese_off` | Config only | Missing. |
| `f3_unicode` | Config only | Missing. |
| `f4_charset` + `f4_charset_value` | Config only | Missing. |
| `f5_settings` | Config only | Missing. |
| `f6_convert_tool` | Config only | Missing. |
| `f7_convert_clipboard` | Config only | Missing. |
| `f11_toggle_excluded_apps` | Config only | Missing. |
| `f12_reset` | Config only | Missing. |

### System settings

| Setting | Current status | Gap |
|---|---:|---|
| `run_as_admin` | Config only | No relaunch/elevation path. |
| `start_with_windows` | Runtime wired | OK. |
| `show_dialog_on_startup` | Runtime wired | OK. |
| `notification_sound` | Runtime wired | OK. |
| `standard_key_sending` | Config/UI only | No alternate key sending mode. |
| `metro_apps_support` | Config/UI only | No UWP/Metro special path. |
| `custom_tray_icon` | Config only | UI does not expose; tray icon fixed. |
| `excluded_apps` | Config/UI only | No active-app detection/policy. |
| `auto_exclude_apps` | Config only | No auto detection/persistence. |
| `check_updates_on_startup` | Config/UI only | No updater/check endpoint. |

### Per-app settings

| Setting | Current status | Gap |
|---|---:|---|
| `apps[exe].enabled` | TOML/config struct only | Not parsed from INI; not applied at runtime. |
| `apps[exe].browser_fix` | TOML/config struct only | Not parsed from INI; not applied at runtime. |

## Implementation principles

1. Make every visible setting either functional or hidden/disabled with explanation.
2. Keep Unikey parity only for behavior already represented by VietKB settings.
3. Do not introduce macro/conversion UI unless implementing existing `F6/F7` hotkeys requires minimal plumbing.
4. Core remains OS-independent. Windows-only app detection/clipboard/key sending stays in `vietkb-windows`.
5. Add tests before/with each behavioral setting.
6. Prefer small phases; each phase should pass `cargo test` and `scripts/build-windows.ps1`.

## Phase 1 — exact engine option semantics

Goal: make existing typing toggles truthful in `vietkb-core`.

### 1.1 Rename/internal-map Unikey concepts

Current config names stay stable. Internally map:

- `typing.smart_mark` -> Unikey-like `freeMarking` behavior.
- `typing.old_oa_uy` -> Unikey-like `modernStyle` special placement for `oa/oe/uy` as UI label says: place tone on second vowel (`oà`, `uý`).
- `typing.spell_check` -> `spellCheckEnabled`.
- `typing.restore_wrong_word` -> `autoNonVnRestore` + restore keystrokes policy.

Files:

- `crates/vietkb-core/src/engine.rs`
- `crates/vietkb-core/src/transform.rs`
- `crates/vietkb-core/src/telex.rs`
- `crates/vietkb-core/src/vni.rs`
- `crates/vietkb-core/src/viqr.rs`

### 1.2 Tone placement parity tests

Add tests covering Unikey cases:

- `hoaf` -> default expected per UI; with `old_oa_uy=true` -> `hoà`.
- `hoef`, `thuyf` with option on/off.
- `quas`, `giaf`: tone target remains after `qu`/`gi` logic.
- trailing consonants: `toans`, `thuyeenf`, `nguyeenx`.
- uppercase: `HOAF`, `Thuyf`.

Files:

- `crates/vietkb-core/tests/telex.rs`
- `crates/vietkb-core/tests/methods.rs`

### 1.3 Free marking / smart mark

Unikey behavior:

- `freeMarking=0`: only modify recent/current valid sequence near the end.
- `freeMarking=1`: allow tone/mark scan farther left within max modify length.

Plan:

- Add a bounded scan window constant like Unikey `MAX_MODIFY_LENGTH`.
- `smart_mark=true`: use Vietnamese syllable-aware target selection.
- `smart_mark=false`: allow freer placement on nearest available vowel/mark target.
- Apply to Telex, VNI, VIQR, not just Telex.

Acceptance:

- Existing tests unchanged.
- New tests verify late tone key still modifies intended vowel when setting allows.

### 1.4 `use_j_f_w_as_consonants`

Expected behavior:

- When `true`: `f/j/w` can remain raw consonants in invalid Vietnamese positions, avoiding unwanted tone/mark conversion.
- When `false`: preserve legacy Telex behavior.

Plan:

- Extend `EngineOptions` with `use_j_f_w_as_consonants`.
- Pass from `TypingConfig` in `vietkb-windows::set_keyboard_config`.
- In Telex handler, before treating `f/j/w` as tone/mark key, ask spelling policy whether current raw word can still be Vietnamese.
- If impossible and option enabled -> commit raw key, no transform.

Tests:

- English-ish words with `f/j/w` do not get accidental marks when option enabled.
- Vietnamese valid transformations still work.

### 1.5 Spell check + skip English integration

Current behavior uses a static English word list. Unikey has richer `spellCheckEnabled` / `strictSpellCheck` / non-VN restore.

Plan:

- Introduce `spelling_policy.rs` with:
  - `is_probably_vietnamese_syllable(raw, transformed)`.
  - `is_known_english_word(raw)` using existing `english.rs`.
  - `should_restore_raw(options, raw, transformed)`.
- `spell_check=false`: allow transformations freely.
- `skip_english_words=true`: bypass known English words.
- `restore_wrong_word=true`: restore raw when word becomes invalid at boundary.

Tests:

- `windows`, `google` remain raw with skip on.
- Vietnamese words still transform.
- Invalid transformed word restores at space/enter when `restore_wrong_word=true`.

## Phase 2 — restore/backspace behavior

Goal: implement Unikey-like `UnikeyRestoreKeyStrokes`, `UnikeyBackspacePress`, `autoNonVnRestore` for current settings.

### 2.1 Keep raw keystroke history

Add to `SimpleVietnameseEngine`:

- `raw: String` already exists; extend with `committed: String` or per-word history.
- Track last emitted replacement length.
- Track last word raw/transformed until boundary.

Add `EngineAction` variants only if necessary:

- Prefer reuse `Replace { delete_graphemes, insert }` for restore.
- Add `Restore { delete_graphemes, insert }` only if platform needs distinct handling.

### 2.2 Backspace-aware state

Current backspace just pops one char from `word/raw`. Need match output state.

Plan:

- On backspace, remove from raw history and recompute transformed word from scratch.
- Emit replacement if recompute output differs.
- If no internal word state -> pass through.

Tests:

- Type `tieengs`, backspace tone key -> output becomes `tieeng`/expected intermediate.
- Backspace after mark key restores correct prior string.

### 2.3 Restore hotkey

Current `hotkeys.restore_word` UI/config exists, hook ignores it.

Plan:

- Add second `Hotkey` in `HookState`: `restore_hotkey`.
- Extend `set_keyboard_config(enabled, method, hotkeys, typing, system?)` or add `set_hotkeys_config` to pass both toggle + restore.
- On restore hotkey: ask engine for last raw word restore action.
- Emit backspaces + raw text.

Files:

- `crates/vietkb-core/src/engine.rs`
- `crates/vietkb-windows/src/hook.rs`
- `crates/vietkb-gui/src/main.rs`

Tests:

- Core restore action unit tests.
- Hook hotkey parser tests if extracted into testable module.

## Phase 3 — runtime charsets

Goal: make `input.charset` functional for currently exposed Unicode/VNI Windows/TCVN3.

### 3.1 Add charset conversion module

Implement minimal output mapping:

- Unicode -> Unicode string.
- VNI Windows -> byte/codepage-like Unicode representation for VNI font encoding.
- TCVN3 -> ABC font encoding.

Reference:

- Unikey `x-unikey/src/vnconv/data.cpp`, `charset.h`.

Plan:

- Add `crates/vietkb-core/src/charset.rs`.
- Extend core `Charset` enum: `Unicode`, `VniWindows`, `Tcvn3`.
- Convert only final emitted text in engine action boundary: `Commit`, `Replace.insert`, `Preedit`.
- Keep internal `word/raw` in Unicode.

### 3.2 Wire config to runtime

Current `vietkb_windows::set_keyboard_config` receives method but not charset.

Plan:

- Pass `config.input.charset` from GUI to Windows hook.
- Map `CharsetConfig` -> `vietkb_core::Charset`.
- `set_charset` actually affects output conversion.

Files:

- `crates/vietkb-core/src/engine.rs`
- `crates/vietkb-core/src/lib.rs`
- `crates/vietkb-windows/src/hook.rs`
- `crates/vietkb-gui/src/main.rs`

Tests:

- `tiếng` converted to expected VNI Windows sequence.
- `tiếng` converted to expected TCVN3 sequence.
- Tray charset changes affect next output.

## Phase 4 — Windows key sending modes

Goal: make `standard_key_sending`, `use_clipboard_to_send_keys`, `metro_apps_support` functional.

### 4.1 Output strategy enum

Add Windows-only strategy:

```rust
enum SendStrategy {
    UnicodeSendInput,
    VirtualKeyStandard,
    ClipboardPaste,
}
```

Mapping:

- `typing.use_clipboard_to_send_keys=true` -> `ClipboardPaste`.
- Else `system.standard_key_sending=true` -> `VirtualKeyStandard` where possible.
- Else -> current `UnicodeSendInput`.
- `system.metro_apps_support=true` -> prefer `ClipboardPaste` or future UIA/TSF path for apps where `SendInput` fails.

### 4.2 Clipboard send path

Reference:

- Unikey `useUnicodeClipboard`, `ClipboardIsEmpty`, `ClearClipboard`.

Plan:

- Save existing clipboard content when possible.
- Set Unicode text.
- Send Ctrl+V.
- Restore clipboard asynchronously/best-effort after small delay, but avoid blocking hook.
- For replacement: send backspaces first, then paste.

Files:

- `crates/vietkb-windows/src/hook.rs`
- maybe new `crates/vietkb-windows/src/clipboard.rs`

Tests:

- Unit-test strategy selection.
- Manual Windows validation: Notepad, browser address bar, UWP text box.

### 4.3 Standard key sending

Plan:

- For ASCII output, send VK events.
- For Vietnamese Unicode output, fallback to Unicode SendInput unless clipboard path enabled.
- Document fallback.

Acceptance:

- Setting no longer inert.
- No regression for current Unicode path.

## Phase 5 — per-app exclude + active app detection

Goal: make `system.excluded_apps`, `system.auto_exclude_apps`, `apps[exe].enabled`, `apps[exe].browser_fix` functional.

Unikey is not required here; implement directly.

### 5.1 Active app detection on Windows

Add functions in `vietkb-windows/src/system.rs` or new `active_app.rs`:

- `GetForegroundWindow`.
- `GetWindowThreadProcessId`.
- `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)`.
- `QueryFullProcessImageNameW`.
- Return:
  - `exe_name`: lowercase file name (`code.exe`).
  - `path`: full exe path.
  - `window_title`: optional via `GetWindowTextW`.

Update `WindowsBackend::active_app` or expose hook helper.

### 5.2 Apply effective enabled policy

Policy:

1. Global `input.enabled=false` -> off.
2. If `system.excluded_apps=false` -> ignore app rules.
3. Else if `apps[exe].enabled=Some(false)` -> off for active app.
4. Else if `apps[exe].enabled=Some(true)` -> on for active app.
5. Else global enabled.

Hook behavior:

- Check active app on each keydown or cache for ~250ms.
- If effective disabled: reset engine, pass key through.
- Tray icon can still show global state; optional later show per-app effective status.

### 5.3 Auto exclude apps

Simple plan:

- If `system.auto_exclude_apps=true`, detect apps where injection likely failed/reentered repeatedly.
- Safer first version: only provide command/API to add current foreground app to `apps[exe].enabled=false` when F11 is pressed.
- Do not auto-write arbitrary app rules based on fragile heuristics initially.

### 5.4 INI support for `[apps.*]`

Current INI parser ignores `apps` entries. Add support:

```ini
[apps.code.exe]
enabled=false
browser_fix=true
```

Plan:

- In `from_ini`, parse section prefix `apps.`.
- In `to_ini`, write app sections sorted by `BTreeMap`.
- Keep TOML support unchanged.

Tests:

- Roundtrip INI with two apps.
- Unknown app options ignored or preserved? Prefer ignore unknown.

## Phase 6 — function key hotkeys

Goal: make existing `hotkeys.function_keys` settings functional.

### 6.1 Hotkey model

Use configured modifiers `ctrl/shift/alt/win` plus F-key.

In hook:

- Detect `VK_F1`..`VK_F12` with exact modifier match.
- Respect per-action booleans.
- Consume event if action handled.

### 6.2 Actions

| Setting | Action |
|---|---|
| `f1_toggle_vietnamese_on` | Set global enabled true. |
| `f2_toggle_vietnamese_off` | Set global enabled false. |
| `f3_unicode` | Set charset Unicode. Persist config. |
| `f4_charset` | Set charset to `f4_charset_value`; validate known values. Persist config. |
| `f5_settings` | Emit callback/event to GUI to show settings. |
| `f6_convert_tool` | If no tool yet: show settings/conversion placeholder; avoid inert setting. |
| `f7_convert_clipboard` | Implement after Phase 7 conversion; initially disabled/hidden. |
| `f11_toggle_excluded_apps` | Add/remove current app exclusion. Persist config. |
| `f12_reset` | Reset engine state. |

Implementation detail:

- Hook currently cannot persist config directly. Add callback from `vietkb-windows` to GUI app layer:
  - `on_command(KeyboardCommand)`.
  - GUI mutates `AppConfig`, saves, updates tray.

Files:

- `crates/vietkb-windows/src/hook.rs`
- `crates/vietkb-gui/src/main.rs`
- `crates/vietkb-config/src/lib.rs`

Tests:

- Hotkey parser/detector unit tests.
- Command callback tests at integration boundary if practical.

## Phase 7 — conversion/clipboard hotkeys

Goal: support currently exposed `F6/F7` enough to avoid dead settings.

### 7.1 Minimal converter API

Reference:

- Unikey `vnconv` conversion pipeline.

Plan:

- Reuse Phase 3 charset mapping.
- Add `vietkb-core::convert_text(input, from_charset, to_charset)`.
- Initially support Unicode <-> VNI Windows <-> TCVN3 only.

### 7.2 F7 convert clipboard

Plan:

- Read Unicode clipboard text.
- Convert using current/selected conversion target.
- Write back to clipboard.
- Notification sound/status message optional.

### 7.3 F6 convert tool

Minimal plan:

- Open settings window on a new `convert` tab if UI is added.
- If no UI, disable/hide F6 setting until tool exists.

## Phase 8 — UI/config cleanup

Goal: visible UI matches implemented runtime.

### 8.1 Add missing visible controls if runtime exists

- Excluded apps list/editor.
- Current active app + add/remove exclusion.
- Function key modifiers/actions.
- Charset conversion tool only after implemented.

### 8.2 Hide/disable nonfunctional settings until implemented

If a setting remains unimplemented after chosen phases, disable with tooltip:

- `run_as_admin`.
- `custom_tray_icon`.
- `check_updates_on_startup`.
- `f6_convert_tool` until conversion UI exists.

### 8.3 Default button

Current `Mặc định` button has no handler.

Plan:

- Add `default_settings` Tauri command returning `AppConfig::default()` as `UiSettings`.
- JS click handler replaces state + render.
- Save only when user clicks `Lưu`.

## Phase 9 — system settings not covered by Unikey parity

### 9.1 `run_as_admin`

Plan:

- Detect elevated status.
- If setting true and not elevated, relaunch via `ShellExecuteW(..., "runas", ...)`, then exit current process.
- Handle cancellation gracefully.

### 9.2 `custom_tray_icon`

Plan:

- Either remove from config/UI until custom icon picker exists, or implement simple alternate V/E icons.
- Prefer hide/disable for now.

### 9.3 `check_updates_on_startup`

Plan:

- If no update server exists, hide/disable.
- Later: check GitHub releases/latest with timeout; never block startup.

## Suggested execution order

1. Phase 1.2 tests + fix exact `old_oa_uy`/tone cases.
2. Phase 5.1-5.4 per-app active app + INI app sections.
3. Phase 2 restore/backspace + restore hotkey.
4. Phase 6 function key hotkeys; implement F11 using Phase 5.
5. Phase 3 charset output.
6. Phase 4 clipboard/standard sending.
7. Phase 7 conversion clipboard/tool.
8. Phase 8 UI cleanup/default button.
9. Phase 9 remaining system settings.

## Validation checklist per phase

Run after each phase:

```powershell
cargo fmt
cargo test
.\scripts\build-windows.ps1
```

Manual smoke on Windows:

1. Start `vietkb-gui.exe`.
2. Toggle E/V from tray and hotkey.
3. Type Telex in Notepad: `Tieengs Vieetj` -> `Tiếng Việt`.
4. Type English words with skip enabled: `google windows` unchanged.
5. Change each newly implemented setting; verify immediate/runtime behavior.
6. Restart app; verify `settings.ini` persists.
7. If per-app exclusion implemented: exclude `notepad.exe`; verify pass-through only there.

## Definition of done

- No visible setting in GUI is silently inert.
- Every config field has one of:
  - runtime implementation,
  - disabled/hidden UI with note,
  - explicit TODO in this plan with no visible control.
- Tests cover engine behavior for all core typing settings.
- Windows hook path respects active app, hotkeys, charset, restore, and send strategy.
