//! Cached Input Monitoring status (macOS).
//!
//! `CGPreflightListenEventAccess()` asks the TCC daemon and costs 15-23 ms. It
//! used to run on the main thread right before every settings-window build (the
//! window height depends on the banner) and again for the page's
//! `input_monitoring_status` query. Both now read this cache instead.
//!
//! The cache is filled off the main thread at startup and refreshed, also off
//! the main thread, whenever it may have changed: a window is about to open, a
//! VietKB window gains focus (the user is back from System Settings), the page
//! polls the status, and after the permission request. A refresh that finds a
//! new value emits `vkb:input-monitoring` so the banner updates at once instead
//! of on its next poll.
//!
//! Compiled on macOS only (see `main.rs`).

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use tauri::Emitter;

const UNKNOWN: u8 = 0;
const GRANTED: u8 = 1;
const DENIED: u8 = 2;

static STATUS: AtomicU8 = AtomicU8::new(UNKNOWN);
/// A refresh thread is running; further requests are folded into it.
static REFRESHING: AtomicBool = AtomicBool::new(false);
/// A request arrived while a refresh was running: read once more after it.
static AGAIN: AtomicBool = AtomicBool::new(false);

fn encode(granted: bool) -> u8 {
    if granted {
        GRANTED
    } else {
        DENIED
    }
}

/// Store a fresh reading; `true` when it differs from a known earlier one.
fn store(granted: bool) -> bool {
    let previous = STATUS.swap(encode(granted), Ordering::AcqRel);
    previous != UNKNOWN && previous != encode(granted)
}

/// The cached status. Before the first reading has landed (only possible in
/// the first milliseconds after launch) it reads synchronously once.
pub fn granted() -> bool {
    match STATUS.load(Ordering::Acquire) {
        GRANTED => true,
        DENIED => false,
        _ => {
            let granted = vkb_macos::has_input_monitoring_permission();
            store(granted);
            granted
        }
    }
}

/// Re-read the status on a background thread. Never blocks the caller.
pub fn refresh() {
    if REFRESHING.swap(true, Ordering::AcqRel) {
        AGAIN.store(true, Ordering::Release);
        return;
    }
    let spawned = std::thread::Builder::new()
        .name("vkb-input-monitoring".into())
        .spawn(|| loop {
            AGAIN.store(false, Ordering::Release);
            let granted = vkb_macos::has_input_monitoring_permission();
            if store(granted) {
                tracing::info!(granted, "Input Monitoring status changed");
                if let Some(app) = crate::commands::app_handle() {
                    let _ = app.emit("vkb:input-monitoring", granted);
                }
            }
            REFRESHING.store(false, Ordering::Release);
            // A request that came in while reading must not be lost.
            if !AGAIN.swap(false, Ordering::AcqRel) || REFRESHING.swap(true, Ordering::AcqRel) {
                break;
            }
        });
    if let Err(error) = spawned {
        REFRESHING.store(false, Ordering::Release);
        tracing::warn!(%error, "cannot spawn the Input Monitoring refresh");
    }
}

/// Record the answer the permission request itself returned, then confirm it.
pub fn note(granted: bool) {
    store(granted);
    refresh();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_change_is_reported_only_against_a_known_value() {
        STATUS.store(UNKNOWN, Ordering::SeqCst);
        assert!(!store(false), "first reading is not a change");
        assert!(!store(false));
        assert!(store(true), "denied -> granted is a change");
        assert!(granted());
        assert!(store(false));
        assert!(!granted());
    }
}
