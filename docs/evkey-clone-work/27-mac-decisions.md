# 27 — macOS decisions

**Status:** authoritative for the macOS build. Lead records every Mac-specific decision here.
Owner mandate (2026-10-31): "Làm hết những gì cần làm để release app cho Mac" — the Lead picks
sensible Mac-native defaults and records them; the owner reviews this file before release.

Shared engine (`vkb-core`), config schema (`vkb-config`) and injection planning
(`vkb-platform`) stay identical across platforms. Only defaults and UI visibility differ.

## M1 — Backend surface (`vkb-macos`)
`vkb-macos` mirrors the `vkb-windows` functions that `vkb-app` calls, so the app wiring is a
copy of `windows_backend.rs` with the crate name swapped:

| Function | Notes |
|---|---|
| `install() -> Result<()>` | Starts the `CGEventTap` thread. Fails (no panic) without Accessibility. |
| `uninstall_hook() -> Result<()>` | Idempotent. |
| `is_hook_ready() -> bool` | Tap created and enabled. |
| `set_config(SharedConfig)` | Arc swap, read by the tap thread. |
| `set_enabled(bool)`, `is_enabled() -> bool` | |
| `set_enabled_callback(Fn(bool))`, `set_command_callback(Fn(BackendCommand))` | Same semantics as Windows (hotkeys). |
| `set_autostart(bool) -> Result<()>`, `is_autostart_enabled() -> bool` | LaunchAgent plist `~/Library/LaunchAgents/vn.vietkb.app.plist`. |
| `set_data_dir(PathBuf)` | |
| `sound::play_toggle(bool)` | |
| `has_accessibility_permission() -> bool`, `request_accessibility_permission() -> bool` | `AXIsProcessTrusted(WithOptions)`. |
| `active_app_bundle() -> Option<String>` | Frontmost bundle id, for per-app rules. |

Injected events carry a marker in `kCGEventSourceUserData`; the tap passes them through.
Secure input (`IsSecureEventInputEnabled`) → the tap passes every key through untouched.

## M2 — Default hotkeys on macOS
Function keys on Mac laptops are brightness/media keys by default, and `F11` is Show Desktop.
Defaults differ by platform (`cfg(target_os = "macos")` in the `hk_*` default fns; schema unchanged):

| Action | Windows | macOS |
|---|---|---|
| toggle | `Ctrl+Shift` | `Ctrl+Shift` (⌃⇧, same as OpenKey/EVKey Mac) |
| enable / disable | `F1` / `F2` | none |
| restore_word | `Ctrl+Shift+Z` | `Ctrl+Shift+Z` (⌃⇧Z; ⌘⇧Z is Redo, untouched) |
| app_exclude | `F11` | none |
| macro_panel / macro_toggle | `F8` / `F9` | none |
| open_settings | `Ctrl+Alt+S` | `Ctrl+Alt+S` (⌃⌥S) |

Modifier naming on macOS: `Ctrl`=⌃ Control, `Alt`=⌥ Option, `Win`=⌘ Command, `Shift`=⇧.
The UI shows the Mac symbols.

## M3 — App shell
- Menu-bar extra only (Tauri tray icon, template image), no Dock icon: `ActivationPolicy::Accessory`.
- Menu-bar icon shows `V` / `E` state like the Windows tray.
- Settings window opens from the menu; standard macOS traffic-light title bar (no custom Windows title bar).
- First run without Accessibility: settings window opens with a clear banner and a button that triggers the system prompt and opens System Settings → Privacy & Security → Accessibility. The app never scripts System Settings.

## M4 — Settings hidden on macOS
Hidden (kept in config, ignored by the Mac backend): legacy charsets other than Unicode,
Windows-only injection options (send mode / clipboard paste variants), "Start with Windows"
label → "Mở cùng macOS" (same `system.start_with_windows` key).

## M5 — Packaging and updates
- Universal `.app` + `.dmg` (`universal-apple-darwin`), ad-hoc signed (`signingIdentity: "-"`)
  so Accessibility permission sticks to the bundle. Not notarized in the first release; release
  notes explain right-click → Open / `xattr -dr com.apple.quarantine`.
- Local QA builds on the owner's Mac (owner, 2026-10-31): signed with the existing
  "Apple Development" identity in the owner's keychain, so the Accessibility grant survives
  rebuilds. Local only — release/CI builds stay ad-hoc. Never export or copy the identity.
- In-app updater on macOS: notify only — shows the new version and opens the release page.
  Silent self-replace waits until the app is signed and notarized.
- CI: `macos-latest` job (test + clippy). Release: macOS job builds the `.dmg` and uploads it
  next to the Windows installer, same tag.
