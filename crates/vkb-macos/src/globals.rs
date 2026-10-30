//! Process-wide state shared between the UI thread and the tap thread.
//!
//! The keystroke path only reads atomics; callbacks are `Arc`s cloned out of
//! their slot before being invoked, so a callback that calls back into this
//! crate cannot deadlock.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use parking_lot::Mutex;
use vkb_platform::{BackendCommand, EngineSink, SharedConfig};

/// Live Vietnamese on/off. The single authority for the base state.
pub static ENABLED: AtomicBool = AtomicBool::new(true);
/// The tap exists, is enabled and the run loop is pumping.
pub static READY: AtomicBool = AtomicBool::new(false);
/// Asks the tap thread to leave its run loop.
pub static STOP: AtomicBool = AtomicBool::new(false);
/// Bumped whenever the UI publishes a new config; the tap thread compares it
/// with the version it last applied (one relaxed load per event).
pub static CONFIG_VERSION: AtomicU64 = AtomicU64::new(0);

type EnabledSlot = Mutex<Option<Arc<dyn Fn(bool) + Send + Sync>>>;
type CommandSlot = Mutex<Option<Arc<dyn Fn(BackendCommand) + Send + Sync>>>;

pub static ENABLED_CB: EnabledSlot = Mutex::new(None);
pub static COMMAND_CB: CommandSlot = Mutex::new(None);
pub static CONFIG: Mutex<Option<SharedConfig>> = Mutex::new(None);
/// Set when the app hands the backend an `EngineSink` (it then owns injection).
pub static SINK: Mutex<Option<Box<dyn EngineSink>>> = Mutex::new(None);
pub static DATA_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Both callbacks below run on the **tap thread**, inside the event callback:
/// they must not block (macOS disables a tap whose callback is slow). The app
/// should only push onto a channel, as `windows_backend.rs` does.
pub fn set_enabled_callback(cb: impl Fn(bool) + Send + 'static) {
    // A Mutex makes the closure `Sync` without requiring it of the caller.
    let slot = Mutex::new(cb);
    *ENABLED_CB.lock() = Some(Arc::new(move |enabled| (slot.lock())(enabled)));
}

pub fn set_command_callback(cb: impl Fn(BackendCommand) + Send + 'static) {
    let slot = Mutex::new(cb);
    *COMMAND_CB.lock() = Some(Arc::new(move |command| (slot.lock())(command)));
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

pub fn store_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::SeqCst);
}

/// Copy of the app config for the tap thread. Called only on install/reload.
pub fn snapshot() -> Option<Arc<vkb_config::AppConfig>> {
    CONFIG
        .lock()
        .as_ref()
        .map(|shared| Arc::new(shared.read().clone()))
}

pub fn publish_config(config: SharedConfig) {
    *CONFIG.lock() = Some(config);
    CONFIG_VERSION.fetch_add(1, Ordering::SeqCst);
}

pub fn config_version() -> u64 {
    CONFIG_VERSION.load(Ordering::Relaxed)
}

/// Tests that touch the process-wide state below must hold this lock.
#[cfg(test)]
pub static TEST_LOCK: Mutex<()> = Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callbacks_are_invoked_and_can_be_replaced() {
        let _guard = TEST_LOCK.lock();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let sink = calls.clone();
        set_command_callback(move |cmd| sink.lock().push(cmd));
        fire_command(BackendCommand::OpenSettings);
        fire_command(BackendCommand::RestoreWord);
        assert_eq!(
            *calls.lock(),
            vec![BackendCommand::OpenSettings, BackendCommand::RestoreWord]
        );

        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = seen.clone();
        set_enabled_callback(move |on| sink.lock().push(on));
        fire_enabled(false);
        assert_eq!(*seen.lock(), vec![false]);
    }

    #[test]
    fn config_publication_bumps_the_version() {
        let _guard = TEST_LOCK.lock();
        let before = config_version();
        publish_config(Arc::new(parking_lot::RwLock::new(vkb_config::AppConfig::default())));
        assert!(config_version() > before);
        assert!(snapshot().is_some());
    }
}
