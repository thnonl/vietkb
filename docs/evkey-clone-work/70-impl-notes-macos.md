# 70 — Implementation notes: macOS backend (`vkb-macos`)

Scope: the real macOS keyboard backend behind decision M1 (`27-mac-decisions.md`), plus the
M2 hotkey defaults in `vkb-config`. App wiring (`vkb-app`, `ui/`, Tauri config, CI) is another
workstream.

## Layout

| File | Role | Platform |
|---|---|---|
| `keys.rs` | macOS keycode → Windows VK (the engine's key space), `CGEventFlags` modifier state, `typed_char` | all |
| `hotkeys.rs` | `HotkeySet` → `vkb_platform::hotkeys::Matcher` | all |
| `state.rs` | `Runtime`: the event state machine (mirror of `vkb-windows/state.rs`), `Host` trait, marker | all |
| `inject.rs` | pure injection plan: Backspace count + UTF-16 chunks | all |
| `autostart.rs` | LaunchAgent plist generation and file handling | all |
| `globals.rs`, `sound.rs` | atomics/callbacks, embedded `go_moc` cues | all |
| `sys.rs` | FFI: CGEventTap thread, `CGEventPost`, AX, secure input, NSWorkspace, NSSound | macOS only |
| `lib.rs` | public API (`imp` module), `MacBackend`, `Injector` | API macOS only |

All decisions live in `Runtime`; `sys.rs` only moves data in and out. `Runtime` talks to the OS
through the `Host` trait, so the whole flow (Telex tone, restore word, hotkeys, secure input,
per-app rules, app switch, config reload) is unit-tested with a mock host — no tap, no
Accessibility, no synthetic input.

## Key design points

- **Key space.** `KeyInput.vk` is the Windows VK space (documented in `vkb-core`). `keys::vk_for_keycode`
  maps `kVK_*` to it, so the engine, `Matcher` and hotkey config are shared unchanged. Command
  is the `win` modifier, Option is `alt`, Control is `ctrl` (M2).
- **Characters** come from the event itself (`CGEventKeyboardGetUnicodeString`), so any layout
  works (AZERTY, Dvorak, …). Windows uses a US table for ASCII then `ToUnicode`; the Mac event
  already carries the layout result, shift and caps lock. A key with no string (dead key) is
  non-text and ends the word; only a *letter* with no string falls back to the US character.
  Keypad and function keys never type (same as Windows).
- **Modifiers** are rebuilt from the flags of every event, so a lost event can never latch
  one. For `flagsChanged`, the device-dependent side bits give left/right; without them (synthetic
  events) the generic mask decides. Caps Lock is `kCGEventFlagMaskAlphaShift`; its `flagsChanged`
  resets the word like the Windows non-text key path.
- **Tap.** `CGEventTapCreate(kCGSessionEventTap, head-insert, default (filtering))` on a dedicated
  thread with its own `CFRunLoop`. Mask: key down/up, flagsChanged, left/right/other mouse down
  (mouse down ends the word, replacing the Windows mouse hook). Re-enabled on
  `kCGEventTapDisabledByTimeout` / `ByUserInput` (the runtime is also reset, events were lost).
  Callback body is wrapped in `catch_unwind`. Returning null swallows the key.
- **Injection.** Backspace = real keycode 51 down/up. Text = key events (keycode 0) with
  `CGEventKeyboardSetUnicodeString`, at most 16 UTF-16 units per event, never splitting a surrogate
  pair or a base+combining-mark cluster. Every event is created from a *private*
  `CGEventSource`, has empty flags (the held Shift/Caps never colours the text, so no Shift
  juggling is needed on macOS) and carries `MARKER` in `kCGEventSourceUserData`; the tap passes
  tagged events through.
- **Secure input.** `IsSecureEventInputEnabled()` is checked on each key-down; true → reset the
  word, pass the key. (macOS normally stops delivering key events to taps in secure fields anyway.)
- **Hotkeys** are matched in the tap (never a global-hotkey API): trigger swallowed on down and
  up, auto-repeat of a swallowed trigger swallowed, `toggle_mode = release` semantics from
  `Matcher`, `restore_word` only after the chord is up and only swallowed when there is a word to
  restore.
- **Per-app rules.** Frontmost pid is read on every key-down (cheap); the lowercased bundle id
  only when the pid changes or after 1.5 s. The bundle id is the rule's `process`. A pid or
  enablement change ends the word.
- **Config.** `set_config` publishes the `SharedConfig` and bumps an atomic version; the tap
  thread reloads when it differs (no lock on the keystroke path).
- **Autostart.** `~/Library/LaunchAgents/vn.vietkb.app.plist`, `RunAtLoad`, `ProgramArguments =
  [current_exe, --autostart]`. Directory overridable with `VKB_LAUNCH_AGENTS_DIR` or the
  `*_in(dir, …)` functions; tests use a temp dir only. Note: nothing runs `launchctl`; the agent
  is loaded at the next login.
- **Sound.** Same `assets/sounds/go_moc/{on,off}.wav` as Windows, `NSSound` from memory on a
  spawned thread (cached per state); `Beep` → `NSBeep`.
- **Hotkey defaults (M2)** in `vkb-config`: `enable/disable/app_exclude/macro_panel/macro_toggle`
  default to unbound on macOS; `toggle` (`Ctrl+Shift`), `restore_word` (`Ctrl+Shift+Z`),
  `open_settings` (`Ctrl+Alt+S`) unchanged. Both the serde default fns and `Default for
  HotkeySet` go through the same `platform_hotkey` helper.

## Deviations from the brief / M1

1. **Post path.** Injected events are posted with `CGEventTapPostEvent(proxy, …)` from inside the
   tap callback (inserted right after the tap, ahead of keys the user types meanwhile, which
   keeps ordering) rather than `CGEventPost`. `CGEventPost(kCGHIDEventTap)` is the fallback
   (no proxy) and can be forced with `VKB_MAC_POST=hid` for A/B on a live run.
2. `vkb-macos` now depends on `vkb-core/config/platform/anyhow/parking_lot/tracing` on every
   platform (was macOS-only) so the pure logic is tested on Windows CI too; only `objc2` is
   macOS-only.
3. Added beyond M1: `active_app()`, `secure_input_enabled()`, `Injector` (`EngineSink`),
   `MacBackend` (`PlatformBackend`), `is_injected`, `MARKER`.
4. `ReplaceThenPass` (macro committed by Enter/Tab) injects then passes the key; the Windows
   hook only injects when `decision.swallow`, which looks like a latent gap there (not touched).
5. `crates/vkb-windows/tests/hook_integration.rs` got `#![cfg(windows)]` so the workspace test
   build works on macOS (outside the stated scope, one line).

## Not verified (needs a live run with Accessibility granted)

- Everything in `sys.rs`: tap creation and enable, re-enable after timeout, `CGEventTapPostEvent`
  ordering vs. the swallowed key, the private-source events being accepted by Chrome/Electron,
  Terminal, VS Code, Safari, Notes, TextEdit.
- Unicode-string events with keycode 0: some apps read the keycode instead of the string
  (terminals?). Fallbacks if seen: real keycodes for ASCII, per-app send delay, clipboard
  paste (Windows has these; the Mac port deliberately does not yet — decision M4).
- Timing: no inter-event delay is inserted; apps that drop fast Backspace bursts (Chrome omnibox,
  Electron autocompletion) may need a small delay.
- `kAXTrustedCheckOptionPrompt` dictionary construction (prompt appearance).
- `NSSound` playback from a background thread; the very first toggle may lag while the
  `NSData`/`NSSound` pair is created.
- `NSWorkspace.frontmostApplication` freshness from the tap thread (it is updated via the main run
  loop; fine in the Tauri app, where the main thread runs it).
- Event tap `kCGSessionEventTap` vs HID: session was chosen (no extra privilege); if some
  secure contexts need HID, it is a one-constant change.
- IME composition (Windows `ime.rs`) has no Mac equivalent here: the macOS Vietnamese input
  source must be switched off (ABC) while VietKB is active.
- Non-macOS compile of the crate (no Windows/Linux target installed locally): the pure modules
  use only portable crates and `sys` is cfg-gated, but it was not cross-checked.
- `cargo test/clippy --workspace` is green on macOS except `vkb-app`, which does not compile
  on macOS yet (`WebviewWindowBuilder::owner`, other workstream).
