//! Windows backend: low-level keyboard hook, text injection, hotkeys, autostart.
//!
//! Everything here translates between the Win32 key space and
//! `vkb_platform::KeyInput` / `EngineAction`; there is no Vietnamese logic in
//! this crate (charter §4).
//!
//! Public surface: the frozen functions from `25-contract.md` §4 plus additive
//! helpers (`install`, `set_config`, `WindowsBackend`, `Injector`,
//! `is_hook_ready`) that the Tauri layer may use.

#![cfg(windows)]

use anyhow::Result;
use vkb_config::HotkeySet;
use vkb_platform::{BackendCommand, Hotkey, SharedConfig};

mod backend;
pub mod autostart;
pub mod clipboard;
pub mod foreground;
mod globals;
pub mod hook;
pub mod hotkeys;
pub mod ime;
pub mod input;
pub mod keys;
/// Opt-in P2 latency probe (`VKB_PERF=1`); see `docs/evkey-clone-work/81-p2-procedure.md`.
pub mod perf;
pub mod sound;
pub mod state;

pub use backend::{Injector, WindowsBackend};
pub use input::SendMode;

/// Install the global keyboard hook and register the hotkey callback.
///
/// Hotkeys are matched inside the hook (never `RegisterHotKey`), so the trigger
/// key is swallowed before the target application sees it.
pub fn install_hotkeys(
    hotkeys: &HotkeySet,
    on_hotkey: impl Fn(Hotkey) + Send + 'static,
) -> Result<()> {
    globals::set_hotkey_callback(on_hotkey);
    *globals::PENDING_HOTKEYS.lock() = Some(hotkeys.clone());
    hook::install()
}

/// Install the hook without touching the hotkey callback (config already set).
pub fn install() -> Result<()> {
    hook::install()
}

/// Remove every hook. Safe to call when nothing is installed.
pub fn uninstall_hook() -> Result<()> {
    hook::uninstall();
    Ok(())
}

/// Publish the config the hook consults. The hook thread re-reads it on its own
/// thread, so the keystroke path never touches this lock.
pub fn set_config(config: SharedConfig) {
    *globals::CONFIG.lock() = Some(config);
    hook::notify_config_changed();
}

/// Flip Vietnamese on/off. Picked up by the hook on the next keystroke.
pub fn set_enabled(enabled: bool) {
    globals::store_enabled(enabled);
}

pub fn is_enabled() -> bool {
    globals::enabled()
}

/// Called on the hook thread when the hotkey toggles the state.
pub fn set_enabled_callback(cb: impl Fn(bool) + Send + 'static) {
    globals::set_enabled_callback(cb);
}

pub fn set_command_callback(cb: impl Fn(BackendCommand) + Send + 'static) {
    globals::set_command_callback(cb);
}

/// `HKCU\...\Run` entry. Per-user, no elevation.
pub fn set_autostart(enabled: bool) -> Result<()> {
    autostart::set_autostart(enabled)
}

pub fn is_autostart_enabled() -> bool {
    autostart::is_autostart_enabled()
}

pub fn play_notification_sound() {
    sound::play_notification_sound();
}

/// Foreground process name, lowercased. Used for per-app rules.
pub fn active_app_name() -> Option<String> {
    foreground::active_app().map(|app| app.process_name)
}

pub fn active_app() -> Option<vkb_platform::AppInfo> {
    foreground::active_app()
}

/// `true` while the hook is installed and pumping.
pub fn is_hook_ready() -> bool {
    hook::is_ready()
}
