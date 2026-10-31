//! Wiring between the Tauri layer and the macOS `CGEventTap` backend.
//!
//! Same shape as `windows_backend.rs`: the tap runs on its own run-loop thread
//! and must never block on file I/O or tray work, so both callbacks only push an
//! [`Event`] onto a channel. A single worker thread applies each event in order,
//! through the config path the Tauri commands use (`update_config`), which
//! persists the file, refreshes the tray, emits `vkb:status` and pushes the new
//! config into the tap (`vkb_macos::set_config`) without a restart.
//!
//! Without the Accessibility grant `install()` fails; the app still opens and the
//! settings window shows the banner. [`ensure_installed`] (called from the
//! `accessibility_status` poll) installs the tap as soon as the grant appears,
//! and the worker also retries on its own every [`TIMER_GAP`] while the tap is
//! missing, so granting with no window open needs no restart (S-02).
//!
//! Compiled on macOS only (see `main.rs`).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use vkb_config::AppConfig;
use vkb_platform::BackendCommand;

use crate::commands;

/// Minimum gap between install attempts triggered by the banner poll, so a grant
/// that still cannot produce a tap does not log an error every 2 seconds.
const RETRY_GAP: Duration = Duration::from_secs(5);

/// Backend-side retry period while the tap is not installed. Cheap: one
/// `is_hook_ready` check per tick once the tap runs.
const TIMER_GAP: Duration = Duration::from_secs(3);

enum Event {
    Enabled(bool),
    Command(BackendCommand),
    /// Accessibility is granted and the tap is not running: try to install it.
    Install,
}

static EVENTS: OnceLock<Sender<Event>> = OnceLock::new();
/// An `Install` event is queued or running; further requests are dropped.
static INSTALL_PENDING: AtomicBool = AtomicBool::new(false);
static LAST_ATTEMPT: Mutex<Option<Instant>> = Mutex::new(None);

/// Connect the tap to the app's config and install it.
///
/// Call once from `setup`, after `set_app_handle` and the config load.
pub fn start() {
    let config = commands::config_cell().read().clone();

    let (tx, rx) = mpsc::channel();
    if EVENTS.set(tx).is_err() {
        tracing::warn!("macOS backend started twice; keeping the first worker");
    } else if let Err(error) = std::thread::Builder::new()
        .name("vkb-backend-events".to_string())
        .spawn(move || worker(rx))
    {
        tracing::error!(%error, "cannot spawn the backend event worker");
    }

    // Publish the live config first: the tap reads it from its first event on.
    vkb_macos::set_config(commands::shared_config());
    vkb_macos::set_enabled(config.input.enabled);

    // Callbacks run on the tap thread: send and return.
    vkb_macos::set_enabled_callback(|enabled| {
        send(Event::Enabled(enabled));
    });
    vkb_macos::set_command_callback(|command| {
        send(Event::Command(command));
    });

    install_now();

    // Autostart is reconciled by `commands::adopt_autostart_registry` (the
    // LaunchAgent plist is the truth), exactly as on Windows.
    tracing::info!(
        hook_ready = vkb_macos::is_hook_ready(),
        accessibility = vkb_macos::has_accessibility_permission(),
        enabled = config.input.enabled,
        "macOS backend start requested"
    );
}

fn install_now() {
    if crate::QUITTING.load(Ordering::SeqCst) {
        return;
    }
    match vkb_macos::install() {
        Ok(()) => tracing::info!("event tap installed"),
        // The app must still open: the banner tells the user what is missing.
        Err(error) => tracing::warn!(error = %error, "event tap install failed"),
    }
}

/// Whether the timer should try an install now. Without the grant there is
/// nothing to try (and nothing to log); once the tap runs the timer stops.
fn timer_should_install(quitting: bool, hook_ready: bool, trusted: bool) -> bool {
    !quitting && !hook_ready && trusted
}

/// One timer tick: install when the grant appeared and the tap is missing.
/// A grant that still cannot produce a tap (Input Monitoring denied) warns once,
/// then retries quietly.
fn timer_tick() {
    static WARNED: AtomicBool = AtomicBool::new(false);
    // Cheapest check first: a running tap needs nothing.
    if vkb_macos::is_hook_ready() {
        return;
    }
    if !timer_should_install(
        crate::QUITTING.load(Ordering::SeqCst),
        false,
        vkb_macos::has_accessibility_permission(),
    ) {
        return;
    }
    match vkb_macos::install() {
        Ok(()) => {
            WARNED.store(false, Ordering::Relaxed);
            tracing::info!("event tap installed by the retry timer");
        }
        Err(error) => {
            if !WARNED.swap(true, Ordering::Relaxed) {
                tracing::warn!(error = %error, "event tap install failed; retrying every few seconds");
            }
        }
    }
}

/// Install the tap if it is not running. Cheap and non-blocking: the work is
/// queued to the worker, at most one request at a time and at most one per
/// [`RETRY_GAP`]. Safe to call on every Accessibility poll.
pub fn ensure_installed() {
    if crate::QUITTING.load(Ordering::SeqCst) || vkb_macos::is_hook_ready() || INSTALL_PENDING.load(Ordering::Acquire) {
        return;
    }
    if !retry_allowed(&LAST_ATTEMPT, Instant::now()) {
        return;
    }
    if INSTALL_PENDING.swap(true, Ordering::AcqRel) {
        return;
    }
    if !send(Event::Install) {
        // Nothing will run the install, so nothing would ever clear the flag.
        INSTALL_PENDING.store(false, Ordering::Release);
    }
}

/// Whether enough time has passed since the last attempt; records `now` when so.
fn retry_allowed(last: &Mutex<Option<Instant>>, now: Instant) -> bool {
    let Ok(mut last) = last.lock() else {
        return false;
    };
    if last.is_some_and(|previous| now.duration_since(previous) < RETRY_GAP) {
        return false;
    }
    *last = Some(now);
    true
}

/// Queue an event for the worker. `false` when it was not queued.
fn send(event: Event) -> bool {
    match EVENTS.get() {
        Some(tx) => match tx.send(event) {
            Ok(()) => true,
            Err(error) => {
                tracing::warn!(%error, "backend event dropped");
                false
            }
        },
        None => {
            tracing::warn!("backend event before start; ignored");
            false
        }
    }
}

/// Push the current config into the tap; called after every save. Only the
/// config: the enabled flag is set explicitly by `start()` and `toggle_enabled`,
/// so a save that does not toggle (a settings patch, a hotkey-driven write)
/// cannot overwrite the tap's own live state.
pub fn push_config() {
    vkb_macos::set_config(commands::shared_config());
}

fn worker(events: Receiver<Event>) {
    loop {
        // Always wake: a tap that dies after install must be retried too.
        // `timer_tick` is a no-op while the tap is ready.
        let event = match events.recv_timeout(TIMER_GAP) {
            Ok(event) => event,
            Err(RecvTimeoutError::Timeout) => {
                timer_tick();
                continue;
            }
            Err(RecvTimeoutError::Disconnected) => break,
        };
        match event {
            Event::Enabled(enabled) => apply_enabled(enabled),
            Event::Command(command) => apply_command(command),
            Event::Install => {
                if !vkb_macos::is_hook_ready() {
                    install_now();
                }
                INSTALL_PENDING.store(false, Ordering::Release);
            }
        }
    }
}

/// Hotkey toggled Vietnamese on/off: make the file and the tray say the same.
fn apply_enabled(enabled: bool) {
    // A field patch under the config lock (S-17). The "already that value?"
    // test is inside `update_config`: an unchanged config is not written, and
    // a no-op write does not broadcast, so the tray is redrawn explicitly.
    let unchanged = commands::config_cell().read().input.enabled == enabled;
    if let Err(error) = commands::update_config(|cfg| {
        cfg.input.enabled = enabled;
        Ok(())
    }) {
        tracing::warn!(%error, enabled, "could not persist the enabled state");
    }
    if unchanged {
        // `update_config` skipped its broadcast: redraw the tray ourselves.
        commands::refresh_tray();
    }
    tracing::info!(
        enabled,
        hook_ready = vkb_macos::is_hook_ready(),
        "enabled state from the tap"
    );
}

fn apply_command(command: BackendCommand) {
    match command {
        BackendCommand::Enable(enabled) => {
            // Sent right after `Enabled`; skip the duplicate write.
            if commands::config_cell().read().input.enabled == enabled {
                return;
            }
            update(|cfg| cfg.input.enabled = enabled);
        }
        BackendCommand::RememberApp { process, enabled } => {
            update(|cfg| remember_app(cfg, &process, enabled))
        }
        BackendCommand::SetMethod(method) => update(|cfg| cfg.input.method = method),
        BackendCommand::RestoreWord => {
            // The word buffer lives on the tap thread; nothing to persist.
            tracing::info!("restore-word hotkey fired");
        }
        BackendCommand::ToggleAppExclude => {
            let Some(bundle) = vkb_macos::active_app_bundle() else {
                tracing::warn!("no frontmost app to exclude");
                return;
            };
            update(|cfg| toggle_app_exclude(cfg, &bundle));
        }
        BackendCommand::ToggleMacro => {
            update(|cfg| cfg.typing.macro_enabled = !cfg.typing.macro_enabled)
        }
        BackendCommand::OpenMacroPanel => {
            if let Some(app) = commands::app_handle() {
                crate::show_settings_tab(app, "macros");
            }
        }
        BackendCommand::OpenSettings => {
            if let Some(app) = commands::app_handle() {
                crate::show_settings(app);
            }
        }
    }
}

/// Apply a field change to the latest config under the commands' config lock
/// (the same path `patch_settings` uses), so a concurrent UI patch is never
/// overwritten with a stale copy (S-17).
fn update(mutate: impl FnOnce(&mut AppConfig)) {
    if let Err(error) = commands::update_config(|cfg| {
        mutate(cfg);
        Ok(())
    }) {
        tracing::warn!(%error, "could not persist a tap-driven change");
    }
}

/// Update the remembered state for one frontmost app (its bundle id).
fn remember_app(cfg: &mut AppConfig, process: &str, enabled: bool) {
    match cfg
        .apps
        .rules
        .iter_mut()
        .find(|rule| rule.process.eq_ignore_ascii_case(process))
    {
        Some(rule) => rule.enabled = Some(enabled),
        None => {
            let mut rule = vkb_config::AppRule::new(process.to_string());
            rule.enabled = Some(enabled);
            cfg.apps.rules.push(rule);
        }
    }
}

/// "Loại trừ ứng dụng này" for the frontmost app: flip its effective state.
fn toggle_app_exclude(cfg: &mut AppConfig, process: &str) {
    let global = cfg.input.enabled;
    match cfg
        .apps
        .rules
        .iter_mut()
        .find(|rule| rule.process.eq_ignore_ascii_case(process))
    {
        Some(rule) => rule.enabled = Some(!rule.enabled.unwrap_or(global)),
        None => {
            let mut rule = vkb_config::AppRule::new(process.to_string());
            rule.enabled = Some(!global);
            cfg.apps.rules.push(rule);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remember_app_adds_then_updates_one_rule_case_insensitively() {
        let mut cfg = AppConfig::default();
        remember_app(&mut cfg, "com.apple.TextEdit", false);
        remember_app(&mut cfg, "com.apple.textedit", true);
        assert_eq!(cfg.apps.rules.len(), 1);
        assert_eq!(cfg.apps.rules[0].enabled, Some(true));
    }

    #[test]
    fn exclude_flips_against_the_global_state_then_back() {
        let mut cfg = AppConfig::default();
        cfg.input.enabled = true;
        toggle_app_exclude(&mut cfg, "com.apple.notes");
        assert_eq!(cfg.apps.rules[0].enabled, Some(false));
        toggle_app_exclude(&mut cfg, "com.apple.notes");
        assert_eq!(cfg.apps.rules[0].enabled, Some(true));
        assert_eq!(cfg.apps.rules.len(), 1);
    }

    #[test]
    fn timer_installs_only_when_granted_missing_and_not_quitting() {
        assert!(timer_should_install(false, false, true));
        assert!(!timer_should_install(false, false, false), "no grant: nothing to try");
        assert!(!timer_should_install(false, true, true), "tap already runs: stop");
        assert!(!timer_should_install(true, false, true), "quitting");
    }

    #[test]
    fn install_retries_are_spaced() {
        let last = Mutex::new(None);
        let start = Instant::now();
        assert!(retry_allowed(&last, start));
        assert!(!retry_allowed(&last, start + Duration::from_secs(1)));
        assert!(retry_allowed(&last, start + RETRY_GAP));
    }
}
