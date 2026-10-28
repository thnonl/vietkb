#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod charset_convert;
mod commands;
mod running_apps;
mod tray;
#[cfg(windows)]
mod windows_backend;

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{Manager, PhysicalPosition, WindowEvent};

/// Set only by the tray's quit item. Everything else is "hide to tray".
static QUITTING: AtomicBool = AtomicBool::new(false);

/// Bring the settings window forward from anywhere (tray, hotkey, second launch).
pub fn show_settings(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Centre `win_len` inside the monitor span `[mon_start, mon_start + mon_len)`,
/// all in physical pixels.
///
/// `Monitor::position/size` and `Window::outer_size` already have the scale
/// factor applied, so nothing here converts or divides by it: under mixed DPI
/// that would double-apply the scaling and push the window off the monitor.
fn centred_axis(mon_start: i32, mon_len: u32, win_len: u32) -> i32 {
    let slack = i64::from(mon_len) - i64::from(win_len);
    if slack <= 0 {
        // Not smaller than the monitor: pin to its origin instead of centring
        // half the window off-screen.
        mon_start
    } else {
        mon_start + (slack / 2) as i32
    }
}

/// Place the fixed-size window centred on the monitor it opens on: the one
/// under the cursor at launch, then the primary, then the current monitor.
/// Runs before the first `show()`, so the window never flashes elsewhere.
fn centre_on_launch_monitor(window: &tauri::WebviewWindow) {
    let monitor = window
        .cursor_position()
        .ok()
        .and_then(|cursor| window.monitor_from_point(cursor.x, cursor.y).ok().flatten())
        .or_else(|| window.primary_monitor().ok().flatten())
        .or_else(|| window.current_monitor().ok().flatten());
    let Some(monitor) = monitor else { return };
    let Ok(win) = window.outer_size() else { return };

    let mon_pos = monitor.position();
    let mon_size = monitor.size();
    let x = centred_axis(mon_pos.x, mon_size.width, win.width);
    let y = centred_axis(mon_pos.y, mon_size.height, win.height);

    tracing::info!(
        monitor = ?monitor.name(),
        mon_x = mon_pos.x,
        mon_y = mon_pos.y,
        mon_w = mon_size.width,
        mon_h = mon_size.height,
        work_x = monitor.work_area().position.x,
        work_y = monitor.work_area().position.y,
        work_w = monitor.work_area().size.width,
        work_h = monitor.work_area().size.height,
        scale = monitor.scale_factor(),
        win_w = win.width,
        win_h = win.height,
        x,
        y,
        "window placement (physical px)"
    );

    if let Err(error) = window.set_position(PhysicalPosition::new(x, y)) {
        tracing::warn!(%error, "could not position window");
    }
}

/// The real exit. Closing the window never does this.
pub fn quit(app: &tauri::AppHandle) {
    QUITTING.store(true, Ordering::SeqCst);
    // No hook may outlive the app: unhook on the way out, then let the probe
    // write its file. The same pair runs after the event loop returns.
    #[cfg(windows)]
    {
        if let Err(error) = vkb_windows::uninstall_hook() {
            tracing::warn!(%error, "could not remove the keyboard hook");
        }
        // P2 probe: write the TSV and the verdict before the process goes away.
        // No-op unless the app was started with `VKB_PERF=1`.
        vkb_windows::perf::finish();
    }
    app.exit(0);
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "vietkb=info,vkb_windows=info".into()),
        )
        .init();

    // P2 latency probe: reads `VKB_PERF` once, here. Off unless the variable is
    // `1`, in which case the keystroke path records samples and the exit path
    // writes `%APPDATA%\VietKB\perf-<timestamp>.tsv`.
    #[cfg(windows)]
    vkb_windows::perf::init_from_env();

    let result = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_settings(app);
        }))
        // The `--autostart` argument is what lets the app tell "Windows started
        // me" apart from "the user double-clicked the exe".
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .arg("--autostart")
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        // No updater plugin: update checking is Sprint 4. Registering it without a
        // `plugins.updater` block in tauri.conf.json fails plugin init at startup.
        .setup(|app| {
            let handle = app.handle().clone();
            commands::set_app_handle(handle.clone());

            let config = commands::config_cell().read().clone();

            // Make the Run key match the saved config on every start, so a
            // config copied between machines does not lie about autostart.
            commands::sync_autostart_now(config.system.start_with_windows);

            tray::install(&handle)?;

            // Hook, config hand-off and both callbacks. Must run after the tray
            // exists (a hotkey can flip it) and after the config load.
            #[cfg(windows)]
            windows_backend::start();

            // Fixed-size window: centre it on the monitor it opens on, before
            // anything can show it (tauri.conf.json no longer says "center").
            if let Some(window) = app.get_webview_window("main") {
                centre_on_launch_monitor(&window);
            }

            tracing::info!(
                enabled = config.input.enabled,
                method = config.input.method.as_str(),
                charset = config.input.charset.as_str(),
                hook_ready = commands::hook_ready_now(),
                "VietKB starting"
            );

            // A Windows-launched instance always starts hidden — the tray is the
            // whole window. A manual launch honours `start_minimized`.
            let launched_by_autostart = std::env::args().any(|arg| arg == "--autostart");
            let should_show = !launched_by_autostart && !config.system.start_minimized;
            if should_show {
                show_settings(&handle);
            }

            Ok(())
        })
        // Closing the window only hides it: the tray keeps the app (and the
        // hook) alive. A real quit is the tray menu's "Thoát VietKB".
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if crate::QUITTING.load(std::sync::atomic::Ordering::SeqCst) {
                    return;
                }
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::set_settings,
            commands::patch_settings,
            commands::reset_settings,
            commands::get_status,
            commands::toggle_enabled,
            commands::set_method,
            commands::set_charset,
            commands::list_macros,
            commands::upsert_macro,
            commands::delete_macros,
            commands::import_macros,
            commands::export_macros,
            commands::list_app_rules,
            commands::set_app_rule,
            commands::set_app_rule_overrides,
            commands::delete_app_rule,
            commands::list_running_apps,
            commands::parse_hotkey,
            commands::check_hotkey_conflict,
            commands::convert_text,
            commands::open_config_folder,
            commands::restart_app,
            commands::app_info,
        ])
        .run(tauri::generate_context!());

    // Exit hook: same unhook and flush as `quit()`, for any path that leaves
    // the event loop without the tray item. Both are idempotent.
    #[cfg(windows)]
    {
        if let Err(error) = vkb_windows::uninstall_hook() {
            tracing::warn!(%error, "could not remove the keyboard hook");
        }
        vkb_windows::perf::finish();
    }

    result.expect("error while running VietKB");
}
