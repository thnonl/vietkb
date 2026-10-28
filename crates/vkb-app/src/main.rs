#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod running_apps;
mod tray;
#[cfg(windows)]
mod windows_backend;

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{Manager, PhysicalPosition, RunEvent, WindowEvent};

/// Set only by the tray's quit item. Everything else is "hide to tray".
static QUITTING: AtomicBool = AtomicBool::new(false);

/// The one label the settings window ever uses. Closing it destroys it; the
/// next open builds a new window under the same label.
const MAIN_LABEL: &str = "main";

/// Bring the settings window forward from anywhere (tray, hotkey, second
/// launch), building it if it is not there yet.
///
/// The window does not exist while the app sits in the tray: closing it tears
/// the WebView2 stack down (charter P3) instead of hiding it, so this is also
/// where it comes back.
pub fn show_settings(app: &tauri::AppHandle) {
    open_settings(app);
}

/// Keep old callers (hotkeys, tray) working after the tab strip was removed:
/// the three screens that used to be tabs are now real windows of their own.
pub fn show_settings_tab(app: &tauri::AppHandle, tab: &str) {
    match tab {
        "hotkeys" => {
            let _ = show_aux(app, "hotkeys");
        }
        "advanced" | "system" | "go" => {
            if tab == "go" {
                show_settings(app);
            } else {
                let _ = show_aux(app, "advanced");
            }
        }
        "macros" => {
            let _ = show_aux(app, "macros");
        }
        _ => show_settings(app),
    }
}

fn open_settings(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_LABEL) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        return;
    }

    if let Err(error) = build_settings_window(app) {
        tracing::warn!(%error, "could not create the settings window");
        // A window whose destruction is still in flight refuses the label. It
        // is the same window the user asked for, so surface it instead of
        // leaving nothing on screen.
        if let Some(window) = app.get_webview_window(MAIN_LABEL) {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

/// Open one of the two secondary windows (or the macro table). They share the
/// frontend bundle and select their screen from `?win=`.
pub fn show_aux(app: &tauri::AppHandle, kind: &str) -> Result<(), String> {
    let (label, title, width, height) = match kind {
        "hotkeys" => ("hotkeys", "Phím tắt — VietKB", 520.0, 460.0),
        "advanced" => ("advanced", "Nâng cao — VietKB", 560.0, 496.0),
        "macros" => ("macros", "Bảng gõ tắt — VietKB", 720.0, 560.0),
        other => return Err(format!("cửa sổ không hợp lệ: {other}")),
    };

    if let Some(window) = app.get_webview_window(label) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        return Ok(());
    }

    let url = tauri::WebviewUrl::App(format!("index.html?win={kind}").into());
    let window = tauri::WebviewWindowBuilder::new(app, label, url)
        .title(title)
        .inner_size(width, height)
        .resizable(false)
        .maximizable(false)
        .decorations(false)
        .shadow(false)
        .transparent(false)
        .visible(false)
        .build()
        .map_err(|error| error.to_string())?;

    window.show().map_err(|error| error.to_string())?;
    centre_on_anchor_monitor(&window, app);
    centre_on_anchor_monitor(&window, app);
    window.set_focus().map_err(|error| error.to_string())?;
    Ok(())
}

/// Build the settings window from scratch: same label, same fixed 480x384
/// frameless shell, centred on the launch monitor as it appears.
fn build_settings_window(app: &tauri::AppHandle) -> tauri::Result<tauri::WebviewWindow> {
    let started = std::time::Instant::now();

    let window = tauri::WebviewWindowBuilder::new(
        app,
        MAIN_LABEL,
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title("Cài đặt VietKB")
    .inner_size(480.0, 384.0)
    .resizable(false)
    .maximizable(false)
    .decorations(false)
    .shadow(false)
    .transparent(false)
    // Hidden at build time; placed and shown below in one go.
    .visible(false)
    .build()?;

    // Show first: a window that has never been shown has not had its DWM frame
    // applied yet, so `outer_size()` lies (measured 918x710 for a 720x560
    // request) and centring would land ~90 px off. Both calls are messages on
    // the same loop, so the first paint happens after the move.
    window.show()?;
    // Two passes: a window that has never been shown sits on the monitor it was
    // created on, and `outer_size()` answers in that monitor's DPI. Moving it
    // to the cursor's monitor first, then centring again, measures the size it
    // actually has there (94 px off on a 125% -> 100% pair otherwise). Both are
    // window messages on the same loop, so the first paint is after the move.
    centre_on_launch_monitor(&window);
    centre_on_launch_monitor(&window);
    window.set_focus()?;

    tracing::info!(
        elapsed_ms = started.elapsed().as_millis() as u64,
        "settings window built"
    );
    Ok(window)
}

/// Place a secondary window on the monitor that currently hosts the main
/// window, so a 125 % main display does not hand its dialogs to a 100 % one.
/// Falls back to the cursor's monitor, then primary/current.
fn centre_on_anchor_monitor(window: &tauri::WebviewWindow, app: &tauri::AppHandle) {
    let monitor = app
        .get_webview_window(MAIN_LABEL)
        .and_then(|main| main.current_monitor().ok().flatten())
        .or_else(|| {
            window
                .cursor_position()
                .ok()
                .and_then(|cursor| window.monitor_from_point(cursor.x, cursor.y).ok().flatten())
        })
        .or_else(|| window.primary_monitor().ok().flatten())
        .or_else(|| window.current_monitor().ok().flatten());
    let Some(monitor) = monitor else { return };
    let Ok(win) = window.outer_size() else { return };

    let mon_pos = monitor.position();
    let mon_size = monitor.size();
    let x = centred_axis(mon_pos.x, mon_size.width, win.width);
    let y = centred_axis(mon_pos.y, mon_size.height, win.height);

    if let Err(error) = window.set_position(PhysicalPosition::new(x, y)) {
        tracing::warn!(%error, "could not position secondary window");
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

/// Hand the pages the window and its WebView2 stack faulted in back to the OS.
/// The code they belong to stays mapped and faults back in on the next open,
/// which is what keeps a tray-only idle inside the P3 budget.
fn release_working_set() {
    #[cfg(windows)]
    {
        use windows::Win32::System::Threading::{GetCurrentProcess, SetProcessWorkingSetSize};
        // SAFETY: `GetCurrentProcess` is a pseudo-handle to this process and is
        // always valid; the two `usize::MAX` arguments mean "trim as much as
        // the OS likes", which is the documented way to empty a working set.
        unsafe {
            if let Err(error) = SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX)
            {
                tracing::warn!(%error, "could not trim the working set");
            }
        }
    }
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

    let app = tauri::Builder::default()
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

            // No window exists before the first open: `tauri.conf.json` no
            // longer lists one, so a tray-only start never pays for WebView2.
            // Every window is centred by `build_settings_window`.

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
        // Closing the window destroys it: the tray keeps the app (and the hook)
        // alive, and WebView2 tears its processes down with the window instead
        // of sitting resident behind a hidden one (charter P3). Nothing is
        // prevented here on purpose — `destroy()` from inside the close
        // handler can fail on a wedged webview channel and leave a window that
        // can never be closed again; the default close always destroys. The
        // run loop's `ExitRequested` guard is what keeps the process alive
        // after the last window is gone. A real quit is the tray menu's
        // "Thoát VietKB". A modal (the UI's `<dialog>`, or a native dialog's
        // own loop) swallows the close, so nothing is torn down under one.
        .on_window_event(|_window, event| {
            if let WindowEvent::CloseRequested { .. } = event {
                // Closed to the tray, not quit: flush the P2 report now so a
                // force-kill afterwards still leaves the file. No-op without
                // `VKB_PERF=1`, and it runs on the backend worker.
                #[cfg(windows)]
                crate::windows_backend::flush_perf();
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
            commands::open_config_folder,
            commands::restart_app,
            commands::open_window,
            commands::quit_app,
            commands::app_info,
        ])
        .build(tauri::generate_context!())
        .expect("error while building VietKB");

    app.run(|_handle, event| match event {
        // Destroying the last window asks for an exit. In tray mode the answer
        // is no: the tray icon and the keyboard hook are the app, and the
        // settings window comes back on demand. A requested exit (the tray's
        // quit item) has a code and is always honoured.
        RunEvent::ExitRequested { api, code, .. } => {
            if code.is_none() && !QUITTING.load(Ordering::SeqCst) {
                api.prevent_exit();
                // The last window is gone, so the WebView2 stack is too: hand
                // back the pages hosting it faulted into this process. Without
                // this the tray idles at ~31 MB instead of ~15 MB (P3).
                release_working_set();
            }
        }
        // Exit hook: same unhook and flush as `quit()`, for any path that
        // leaves the event loop without the tray item. Both are idempotent.
        RunEvent::Exit => {
            #[cfg(windows)]
            {
                if let Err(error) = vkb_windows::uninstall_hook() {
                    tracing::warn!(%error, "could not remove the keyboard hook");
                }
                vkb_windows::perf::finish();
            }
        }
        _ => {}
    });
}
