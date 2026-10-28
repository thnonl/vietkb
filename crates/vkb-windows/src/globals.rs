//! Process-wide state shared between the UI thread and the hook thread.
//!
//! Rules that keep the keystroke path fast:
//! * everything the hook reads per keystroke is an `Atomic*` — no lock;
//! * the config `SharedConfig` is only *read* on the hook thread when the UI
//!   posts `WM_APP_CONFIG`, never per keystroke;
//! * callbacks are `Arc`s cloned out of the slot before being invoked, so a
//!   callback that calls back into this crate cannot deadlock.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

use parking_lot::Mutex;
use vkb_config::HotkeySet;
use vkb_platform::{BackendCommand, EngineSink, Hotkey, SharedConfig};

/// Live Vietnamese on/off. The single authority for the base state.
pub static ENABLED: AtomicBool = AtomicBool::new(true);

/// `true` once the hook is installed and the message loop is pumping.
pub static READY: AtomicBool = AtomicBool::new(false);

/// `true` while the hook thread is alive (set/cleared inside `worker_main`).
pub static ALIVE: AtomicBool = AtomicBool::new(false);

/// Bumped by the hook thread every loop turn; the watchdog treats a stale value
/// as a hung hook.
pub static HEARTBEAT: AtomicU64 = AtomicU64::new(0);

/// Asks the hook thread to exit its loop.
pub static STOP: AtomicBool = AtomicBool::new(false);

/// Thread id of the hook thread, for `PostThreadMessageW`. 0 = not running.
pub static THREAD_ID: AtomicU32 = AtomicU32::new(0);

pub static HOTKEY_CB: Mutex<Option<Arc<dyn Fn(Hotkey) + Send + Sync>>> = Mutex::new(None);
pub static ENABLED_CB: Mutex<Option<Arc<dyn Fn(bool) + Send + Sync>>> = Mutex::new(None);
pub static COMMAND_CB: Mutex<Option<Arc<dyn Fn(BackendCommand) + Send + Sync>>> = Mutex::new(None);

/// Config the hook re-reads on demand.
pub static CONFIG: Mutex<Option<SharedConfig>> = Mutex::new(None);

/// Hotkeys used when the app has not supplied a full config yet.
pub static PENDING_HOTKEYS: Mutex<Option<HotkeySet>> = Mutex::new(None);

/// Set when the app hands the backend an `EngineSink` (it then owns injection).
pub static SINK: Mutex<Option<Box<dyn EngineSink>>> = Mutex::new(None);

pub fn set_hotkey_callback(cb: impl Fn(Hotkey) + Send + 'static) {
    // A Mutex makes the closure `Sync` without requiring it of the caller.
    let slot = parking_lot::Mutex::new(cb);
    *HOTKEY_CB.lock() = Some(Arc::new(move |hotkey| (slot.lock())(hotkey)));
}

pub fn set_enabled_callback(cb: impl Fn(bool) + Send + 'static) {
    let slot = parking_lot::Mutex::new(cb);
    *ENABLED_CB.lock() = Some(Arc::new(move |enabled| (slot.lock())(enabled)));
}

pub fn set_command_callback(cb: impl Fn(BackendCommand) + Send + 'static) {
    let slot = parking_lot::Mutex::new(cb);
    *COMMAND_CB.lock() = Some(Arc::new(move |command| (slot.lock())(command)));
}

pub fn fire_hotkey(hotkey: Hotkey) {
    let cb = HOTKEY_CB.lock().clone();
    if let Some(cb) = cb {
        cb(hotkey);
    }
}

pub fn fire_enabled(enabled: bool) {
    let cb = ENABLED_CB.lock().clone();
    if let Some(cb) = cb {
        cb(enabled);
    }
}

pub fn fire_command(command: BackendCommand) {
    let cb = COMMAND_CB.lock().clone();
    if let Some(cb) = cb {
        cb(command);
    }
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Copy of the app config for the hook thread. Called only on install/reload.
pub fn snapshot() -> Option<Arc<vkb_config::AppConfig>> {
    CONFIG
        .lock()
        .as_ref()
        .map(|shared| Arc::new(shared.read().clone()))
}

pub fn store_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callbacks_are_invoked_and_can_be_replaced() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let sink = calls.clone();
        set_hotkey_callback(move |hk| sink.lock().push(hk));
        fire_hotkey(Hotkey::Toggle);
        fire_hotkey(Hotkey::Enable);
        assert_eq!(*calls.lock(), vec![Hotkey::Toggle, Hotkey::Enable]);
    }

    #[test]
    fn enabled_flag_round_trips() {
        store_enabled(false);
        assert!(!enabled());
        store_enabled(true);
        assert!(enabled());
    }

    #[test]
    fn command_callback_maps_backend_commands() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = seen.clone();
        set_command_callback(move |cmd| sink.lock().push(cmd));
        fire_command(BackendCommand::OpenSettings);
        assert_eq!(*seen.lock(), vec![BackendCommand::OpenSettings]);
    }
}
