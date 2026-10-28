//! Windows backend: low-level keyboard hook, text injection, hotkeys, autostart.
//!
//! Sprint 1 fills the bodies. The public surface is frozen — see
//! `docs/evkey-clone-work/25-contract.md` §4.

#![cfg(windows)]

use anyhow::Result;
use vkb_config::HotkeySet;
use vkb_platform::{BackendCommand, Hotkey};

/// Install the global keyboard hook and start pumping events on the hook thread.
pub fn install_hotkeys(_hotkeys: &HotkeySet, _on_hotkey: impl Fn(Hotkey) + Send + 'static) -> Result<()> {
    todo!("sprint 1: install hook + hotkey matcher")
}

/// Remove every hook. Safe to call when nothing is installed.
pub fn uninstall_hook() -> Result<()> {
    todo!("sprint 1: unhook")
}

pub fn set_enabled(_enabled: bool) {
    todo!("sprint 1: flip the atomic the hook reads")
}

pub fn is_enabled() -> bool {
    false
}

pub fn set_enabled_callback(_cb: impl Fn(bool) + Send + 'static) {}

pub fn set_command_callback(_cb: impl Fn(BackendCommand) + Send + 'static) {}

/// `HKCU\...\Run` entry. Per-user, no elevation.
pub fn set_autostart(_enabled: bool) -> Result<()> {
    todo!("sprint 1: HKCU Run key")
}

pub fn play_notification_sound() {}

/// Foreground process name, lowercased. Used for per-app rules.
pub fn active_app_name() -> Option<String> {
    None
}
