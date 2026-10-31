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
- Menu-bar extra (Tauri tray icon, template image). **Revised (owner, 2026-10-31):** the app
  shows a Dock icon while any VietKB window is open (`ActivationPolicy::Regular`, app activated
  so windows come to the front), and returns to menu-bar-only (`Accessory`) when the last window
  closes/hides. Owner report: no Dock icon when the GUI opened, and settings modals opened hidden
  behind other windows.
- Every window and modal VietKB opens must come to the front and be visible (activate the app);
  modals must sit above their parent and never end up behind it or behind other apps.
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

## M6 — English UI and docs (owner, 2026-10-31; Windows and macOS)
- The app UI is fully bilingual, Vietnamese (default) and English, on both platforms: every
  screen, dialog, toast, banner, tray/menu-bar menu, tooltip and Rust-side user-facing message.
  This supersedes the contract C7 note that product terms and long copy stay Vietnamese-only.
- Uses the existing `system.language` config key (`vi` | `en`); no schema change.
- Quick toggle `VI | EN` sits next to the theme toggle in the title bar on Windows. On macOS it
  sits at the same top-right spot of the window content, **next to a theme toggle** (owner,
  2026-10-31: "Ở Mac cũng phải có toggle dark mode") with the same three states as Windows
  (Theo hệ thống / Sáng / Tối, default Theo hệ thống on Mac), stored in `system.theme`. This
  replaces the earlier rule that the Mac app always follows the system appearance. Switching applies instantly to every open window and the menu, no restart.
- The GitHub Pages site gets an English version with a language switch in the header; Vietnamese
  stays the default at `/`, English at `/en/`. Both languages cover Windows and macOS.
- English copy is written/polished by the copywriter; tone follows 90-copy-voice.md adapted to English.

## M7 — Theme default follows the system (owner, 2026-10-31; Windows and macOS)
Owner, verbatim (given in the Windows session): "Default theme của app sẽ theo system, khi user bật
dark/light mode sẽ được lưu lại."
- Default `system.theme = system` on both platforms; migration no longer rewrites `system` to `dark`.
- With `system`, every window follows the OS appearance live.
- The theme toggle shows the effective light/dark state; clicking stores an explicit `light`/`dark`
  in config.toml (persists across restarts, applies to every window).
- "Đặt lại tất cả" / Reset all returns the theme to `system`.
- Existing 0.2.6 files with `theme = "dark"` stay dark (indistinguishable from a chosen dark).
- Supersedes the Mac-only three-state toggle of M6.

## M8 — Singleton and update badge (owner, 2026-10-31; Windows and macOS)
Owner, verbatim: "Hãy nhớ rằng app này là singleton. Và khi có update cần hiện icon update giống như Windows."
- VietKB is a singleton per channel: one running instance per bundle identifier (`vn.vietkb.app`;
  the dev channel `vn.vietkb.app.dev` is a separate identity for side-by-side testing only). A second
  launch — from Finder, Spotlight, Dock, `open -n`, a second copy of the .app at another path, or
  login-item start while already running — must not start a second process/menu-bar icon/Dock tile;
  it hands over to the running one (which shows its settings window) and exits.
- Agents and the QA harness keep at most one dev-channel instance alive and quit it gracefully.
- Update available on macOS shows the same signals as Windows: the menu-bar icon carries an update
  badge (template image keeps the glyph; the badge is a coloured dot drawn as a non-template overlay,
  or a dedicated badged image — whatever renders legibly in light and dark menu bars), the title-bar
  area/top-right row of the settings window shows the same update badge/button as Windows, and the
  menu has the update item. Supersedes the M5/Dev-B choice "no update badge on mac".
- macOS menu-bar badge is monochrome in the template alpha (menu-bar tint follows the wallpaper); see 70-impl-notes-macos.md.
- Owner, 2026-10-31, on 2-state vs adding a "system" state: "Làm theo Windows, khi mở lần đầu thì theo hệ thống, khi user toggle thì lưu user setting." → keep the 2-state toggle as implemented (no third state); the way back to following the system is "Đặt lại tất cả".
