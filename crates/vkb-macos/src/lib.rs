//! macOS backend (Sprint 3). Compiles everywhere; only the Apple build has a body.
//!
//! Design notes: CGEventTap on a dedicated run-loop thread, Accessibility
//! permission requested lazily on first enable, menu-bar-only app,
//! secure-input detection must disable the tap (never type into password fields).

#![cfg_attr(not(target_os = "macos"), allow(dead_code))]

#[cfg(target_os = "macos")]
mod imp {
    use anyhow::Result;

    pub fn install_event_tap() -> Result<()> {
        todo!("sprint 3: CGEventTap")
    }

    pub fn remove_event_tap() -> Result<()> {
        todo!("sprint 3: teardown")
    }

    pub fn has_accessibility_permission() -> bool {
        false
    }

    pub fn request_accessibility_permission() -> bool {
        false
    }

    pub fn set_autostart(_enabled: bool) -> Result<()> {
        todo!("sprint 3: launch agent plist")
    }

    pub fn active_app_bundle() -> Option<String> {
        None
    }
}

#[cfg(target_os = "macos")]
pub use imp::*;

/// True when the current OS can host a real backend.
pub const IMPLEMENTED: bool = cfg!(target_os = "macos");
