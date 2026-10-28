//! Wiring between the Tauri layer and the Windows keyboard hook.
//!
//! The hook owns a message-loop thread and must never be blocked by file I/O
//! or tray work, so both backend callbacks only push an [`Event`] onto a
//! channel. A single worker thread applies each event in order, through the
//! same config path the Tauri commands use (`save_and_publish`), which
//! persists the file, refreshes the tray, emits `vkb:status` and pushes the
//! new config into the hook (`vkb_windows::set_config`) without a restart.
//!
//! Everything here is compiled on Windows only (see `main.rs`).

use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::OnceLock;
use std::time::Duration;

use vkb_config::AppConfig;
use vkb_platform::BackendCommand;

use crate::commands;

/// How often a long `VKB_PERF=1` session rewrites its report. Only armed when the
/// probe is on; otherwise the worker blocks on `recv` exactly as before.
const PERF_FLUSH_INTERVAL: Duration = Duration::from_secs(60);

enum Event {
    Enabled(bool),
    Command(BackendCommand),
    /// Write the P2 report now (window hidden to the tray). No-op unless
    /// `VKB_PERF=1`.
    PerfFlush,
}

static EVENTS: OnceLock<Sender<Event>> = OnceLock::new();

/// Install the hook and connect it to the app's config.
///
/// Call once from `setup`, after `set_app_handle` and the config load, so the
/// hook never reads a config the rest of the app does not have.
pub fn start() {
    let config = commands::config_cell().read().clone();

    let (tx, rx) = mpsc::channel();
    if EVENTS.set(tx).is_err() {
        tracing::warn!("windows backend started twice; keeping the first worker");
    } else if let Err(error) = std::thread::Builder::new()
        .name("vkb-backend-events".to_string())
        .spawn(move || worker(rx))
    {
        tracing::error!(%error, "cannot spawn the backend event worker");
    }

    // Publish the live config first: the hook thread copies it when it starts,
    // and every later `set_config` is a single `Arc` swap plus a wake-up.
    vkb_windows::set_config(commands::shared_config());

    // Register both callbacks before installing, so a hotkey pressed during
    // startup is already delivered instead of dropped.
    vkb_windows::set_enabled_callback(|enabled| send(Event::Enabled(enabled)));
    vkb_windows::set_command_callback(|command| send(Event::Command(command)));

    match vkb_windows::install() {
        Ok(()) => tracing::info!("keyboard hook installed"),
        // The app must still open: the window is where the user can see why
        // typing does nothing. The full Win32 error (including its code) is in
        // this line and in the hook thread's own "refused" line.
        Err(error) => tracing::error!(error = %error, "keyboard hook install failed"),
    }

    vkb_windows::set_enabled(config.input.enabled);
    sync_autostart(config.system.start_with_windows);

    tracing::info!(
        hook_ready = vkb_windows::is_hook_ready(),
        enabled = config.input.enabled,
        "Windows backend start requested"
    );
}

fn send(event: Event) {
    match EVENTS.get() {
        Some(tx) => {
            // Never block the hook thread; a full channel would mean the
            // worker is stuck, and losing one event is better than a stall.
            if let Err(error) = tx.send(event) {
                tracing::warn!(%error, "backend event dropped");
            }
        }
        None => tracing::warn!("backend event before start; ignored"),
    }
}

/// Reconcile the `Run` key with the config. The config wins; a mismatch that
/// survives the write is a real failure and is logged instead of ignored.
fn sync_autostart(enabled: bool) {
    if let Err(error) = vkb_windows::set_autostart(enabled) {
        tracing::warn!(%error, enabled, "could not update the Run key");
        return;
    }
    if vkb_windows::is_autostart_enabled() != enabled {
        tracing::warn!(
            wanted = enabled,
            found = vkb_windows::is_autostart_enabled(),
            "autostart still disagrees with the config after writing"
        );
    }
}

/// Push the current config into the hook; called after every save.
pub fn push_config() {
    vkb_windows::set_config(commands::shared_config());
}

fn worker(events: Receiver<Event>) {
    // Read `VKB_PERF` once, here: with the probe off this is the same blocking
    // receive as before — no timer, no wake-ups, no file.
    let perf_timer = vkb_windows::perf::enabled();
    loop {
        let received = if perf_timer {
            events.recv_timeout(PERF_FLUSH_INTERVAL)
        } else {
            events.recv().map_err(|_| RecvTimeoutError::Disconnected)
        };
        match received {
            Ok(Event::Enabled(enabled)) => apply_enabled(enabled),
            Ok(Event::Command(command)) => apply_command(command),
            Ok(Event::PerfFlush) => {
                vkb_windows::perf::flush();
            }
            // A session that never quits must still leave a fresh report on disk.
            Err(RecvTimeoutError::Timeout) => {
                vkb_windows::perf::flush();
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
}

/// Write the P2 report now, on the backend worker (never on the hook thread).
/// Called when the window hides to the tray so closing cannot strand the
/// samples. No-op unless `VKB_PERF=1`.
pub fn flush_perf() {
    if vkb_windows::perf::enabled() {
        send(Event::PerfFlush);
    }
}

/// Hotkey toggled Vietnamese on/off: make the file and the tray say the same.
fn apply_enabled(enabled: bool) {
    let mut draft = commands::config_cell().read().clone();
    if draft.input.enabled != enabled {
        draft.input.enabled = enabled;
        if let Err(error) = commands::save_and_publish(draft) {
            tracing::warn!(%error, enabled, "could not persist the enabled state");
        }
    } else if let Some(app) = commands::app_handle() {
        // Same value but the tray may still be stale (e.g. a repeated press).
        crate::tray::refresh(app, &commands::status_now());
    }
    tracing::info!(
        enabled,
        hook_ready = vkb_windows::is_hook_ready(),
        "enabled state from the hook"
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
        BackendCommand::SetMethod(method) => update(|cfg| cfg.input.method = method),
        BackendCommand::SetCharset(charset) => update(|cfg| cfg.input.charset = charset),
        BackendCommand::RestoreWord => {
            // The word buffer lives on the hook thread and no config applies;
            // nothing to persist here.
            tracing::info!("restore-word hotkey fired");
        }
        BackendCommand::ToggleAppExclude => toggle_app_exclude(),
        BackendCommand::ToggleMacro => {
            update(|cfg| cfg.typing.macro_enabled = !cfg.typing.macro_enabled)
        }
        BackendCommand::OpenConvertTool => navigate("go"),
        BackendCommand::OpenMacroPanel => navigate("macros"),
        BackendCommand::OpenSettings => {
            if let Some(app) = commands::app_handle() {
                crate::show_settings(app);
            }
        }
    }
}

/// Read-modify-write the config through the commands' own save path.
fn update(mutate: impl FnOnce(&mut AppConfig)) {
    let mut draft = commands::config_cell().read().clone();
    mutate(&mut draft);
    if let Err(error) = commands::save_and_publish(draft) {
        tracing::warn!(%error, "could not persist a hook-driven change");
    }
}

/// "Loại trừ ứng dụng này" for whatever owns the caret right now.
fn toggle_app_exclude() {
    let Some(process) = vkb_windows::active_app_name() else {
        tracing::warn!("no foreground app to exclude");
        return;
    };
    let mut draft = commands::config_cell().read().clone();
    let global = draft.input.enabled;
    match draft
        .apps
        .rules
        .iter_mut()
        .find(|rule| rule.process.eq_ignore_ascii_case(&process))
    {
        Some(rule) => rule.enabled = Some(!rule.enabled.unwrap_or(global)),
        None => {
            let mut rule = vkb_config::AppRule::new(process.clone());
            rule.enabled = Some(!global);
            draft.apps.rules.push(rule);
        }
    }
    if let Err(error) = commands::save_and_publish(draft) {
        tracing::warn!(%error, process, "could not persist the app rule");
    }
}

fn navigate(tab: &str) {
    if let Some(app) = commands::app_handle() {
        // A closed window is rebuilt with `?tab=` so the deep link wins over
        // the restored last tab; an open one takes the event.
        crate::show_settings_tab(app, tab);
    }
}
