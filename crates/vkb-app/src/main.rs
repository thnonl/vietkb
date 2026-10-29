#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod running_apps;
mod tray;
mod update;
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

/// Raise the About modal: from the tray's update item and from the title-bar
/// badge's window-level entry point. The page may not exist yet, so the request
/// is also left in the update state for the next startup of the main window.
pub fn show_about(app: &tauri::AppHandle) {
    if let Err(error) = show_aux(app, "about") {
        tracing::warn!(%error, "could not open the About window");
        open_settings(app);
    }
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

/// Open one of the three secondary windows (or the macro table) as a modal of
/// the main window. They share the frontend bundle and select their screen from
/// `?win=`.
///
/// Tauri 2 has no `modal` window flag, so modality is built from two parts:
/// the window is *owned* by the main window (`owner`), which keeps it above its
/// parent and destroys it with it, and the main window is disabled
/// (`set_enabled(false)`) for as long as the modal lives. The modal is centred
/// on the main window rather than on the monitor.
pub fn show_aux(app: &tauri::AppHandle, kind: &str) -> Result<(), String> {
    // Height is the content height rounded up to the 4 px grid, never a
    // guessed constant — the same rule the main window uses. Each value is
    // measured from the screen it opens:
    //   hotkeys   36 caption + 1 + 44 intro + 5×44 rows + 56 footer = 357 → 360
    //   advanced  36 caption + 1 + 4 sections (33 head, 28/row grid, 36 field
    //             row) + 56 footer = 488 → 488
    //   macros    36 caption + 1 + 16 + 36 toolbar + 12 + 406 table
    //             (36 header + 8×46 rows + 2 border) + 9 + 28 footer
    //             + 16 sheet pad = 560 → 560
    //   about     36 caption + 1 + 12 pad + 493 measured content (story +
    //             update block + links) + 14 pad = 556, inside the 560 px hard
    //             cap the owner set; the sheet scrolls only if a longer
    //             changelog is clipped.
    let (label, title, width, height) = match kind {
        "hotkeys" => ("hotkeys", "Phím tắt — VietKB", 520.0, 360.0),
        "advanced" => ("advanced", "Nâng cao — VietKB", 560.0, 488.0),
        "macros" => ("macros", "Bảng gõ tắt — VietKB", 720.0, 560.0),
        "about" => ("about", "Giới thiệu — VietKB", 560.0, 556.0),
        other => return Err(format!("cửa sổ không hợp lệ: {other}")),
    };

    let main = app.get_webview_window(MAIN_LABEL);

    if let Some(window) = app.get_webview_window(label) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        if let Some(main) = &main {
            let _ = main.set_enabled(false);
        }
        return Ok(());
    }

    let url = tauri::WebviewUrl::App(format!("index.html?win={kind}").into());
    let mut builder = tauri::WebviewWindowBuilder::new(app, label, url)
        .title(title)
        .inner_size(width, height)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .decorations(false)
        // Frameless windows get no system border, so the native shadow is the
        // only thing separating this window from whatever is behind it. The 1 px
        // ring the page draws on top is the edge itself (ui/src/styles/v4.css).
        .shadow(true)
        .transparent(false)
        .visible(false);

    // Owned by the main window: no orphan in the taskbar, and Windows tears it
    // down first when the owner goes away.
    if let Some(main) = &main {
        builder = builder.owner(main).map_err(|error| error.to_string())?;
    }

    let window = builder.build().map_err(|error| error.to_string())?;

    window.show().map_err(|error| error.to_string())?;
    centre_on_parent(&window, main.as_ref());
    // Modal: the parent stops accepting input until this window is gone.
    if let Some(main) = &main {
        if let Err(error) = main.set_enabled(false) {
            tracing::warn!(%error, "could not disable the main window for a modal");
        }
    }
    window.set_focus().map_err(|error| error.to_string())?;
    Ok(())
}

/// Centre a modal on the window that owns it, in physical pixels.
fn centre_on_parent(
    window: &tauri::WebviewWindow,
    parent: Option<&tauri::WebviewWindow>,
) {
    let Some(parent) = parent else { return };
    let (Ok(pos), Ok(parent_size), Ok(win)) = (
        parent.outer_position(),
        parent.outer_size(),
        window.outer_size(),
    ) else {
        return;
    };

    // Exact centre of the parent in both axes: a modal wider or taller than the
    // 480 px main window overhangs both sides by the same amount instead of
    // hugging its top-left corner.
    let mut x = pos.x + (parent_size.width as i32 - win.width as i32) / 2;
    let mut y = pos.y + (parent_size.height as i32 - win.height as i32) / 2;

    // …but never off the monitor the parent sits on.
    if let Ok(Some(monitor)) = parent.current_monitor() {
        let area = monitor.work_area();
        let left = area.position.x;
        let top = area.position.y;
        let right = (left + area.size.width as i32 - win.width as i32).max(left);
        let bottom = (top + area.size.height as i32 - win.height as i32).max(top);
        x = x.clamp(left, right);
        y = y.clamp(top, bottom);
    }

    if let Err(error) = window.set_position(PhysicalPosition::new(x, y)) {
        tracing::warn!(%error, "could not centre the modal on the main window");
    }
}

/// Labels of the modal windows that can be open at once.
const AUX_LABELS: [&str; 4] = ["hotkeys", "advanced", "macros", "about"];

/// Close every open modal. Used before the main window goes away so no window
/// outlives its parent.
fn close_aux(app: &tauri::AppHandle) {
    for label in AUX_LABELS {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.close();
        }
    }
}

/// Build the settings window from scratch: same label, same fixed 480x460
/// frameless shell, centred on the launch monitor as it appears.
fn build_settings_window(app: &tauri::AppHandle) -> tauri::Result<tauri::WebviewWindow> {
    let started = std::time::Instant::now();

    let window = tauri::WebviewWindowBuilder::new(
        app,
        MAIN_LABEL,
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title("Cài đặt VietKB")
    .inner_size(480.0, 460.0)
    .resizable(false)
    .maximizable(false)
    .decorations(false)
    // Frameless: without the native shadow the window has no edge against a
    // neighbouring app. `transparent` stays false, which is what Windows needs
    // for DWM to paint the frame shadow at all.
    .shadow(true)
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
        // Second launch: same rule as a first one, so `Chạy ẩn` also keeps a
        // re-launch in the tray instead of flashing a window.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if !commands::config_cell().read().system.start_minimized {
                show_settings(app);
            }
        }))
        // The `--autostart` argument identifies a Windows launch in the `Run`
        // value. It no longer decides what appears on screen: `Chạy ẩn` does
        // that for every launch path alike.
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .arg("--autostart")
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        // No updater plugin: the GitHub Releases check lives in `update.rs`
        // (plain `ureq`), so `plugins.updater` is never needed in the config.
        .setup(|app| {
            let handle = app.handle().clone();
            commands::set_app_handle(handle.clone());

            let config = commands::config_cell().read().clone();

            // The `Run` key is the truth for autostart, not the config file: a
            // user who removed the entry in Windows' own startup settings must
            // see it off, not get it silently re-created. Adopt it before the
            // first window is built, so the switch never lies.
            commands::adopt_autostart_registry(config.system.start_with_windows);

            tray::install(&handle)?;

            // Five seconds after launch, off the UI thread, honouring
            // `system.check_updates`. A failure stays silent; the About window
            // shows the reason when the owner opens it.
            update::schedule_startup_check(handle.clone());

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

            // Every launch path obeys the same switch: `Chạy ẩn`
            // (`system.start_minimized`) hides the window for a Start-menu pick,
            // a desktop shortcut, an autostart run and a second instance alike.
            // `--autostart` is no longer a special case.
            if !config.system.start_minimized {
                show_settings(&handle);
            }

            Ok(())
        })
        // Closing the main window destroys it: the tray keeps the app (and the
        // hook) alive, and WebView2 tears its processes down with the window
        // instead of sitting resident behind a hidden one (charter P3). Nothing
        // is prevented here on purpose — `destroy()` from inside the close
        // handler can fail on a wedged webview channel and leave a window that
        // can never be closed again; the default close always destroys. The run
        // loop's `ExitRequested` guard is what keeps the process alive after the
        // last window is gone. A real quit is the tray menu's "Thoát VietKB".
        //
        // Modals are owned by the main window and the main window is disabled
        // while one is open, so this is also where they are taken down: closing
        // the parent closes its modals first (no orphan window), and closing a
        // modal hands the main window its input back.
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { .. } => {
                if window.label() == MAIN_LABEL {
                    close_aux(window.app_handle());
                }
                // Closed to the tray, not quit: flush the P2 report now so a
                // force-kill afterwards still leaves the file. No-op without
                // `VKB_PERF=1`, and it runs on the backend worker.
                #[cfg(windows)]
                crate::windows_backend::flush_perf();
            }
            WindowEvent::Destroyed => match window.label() {
                MAIN_LABEL => close_aux(window.app_handle()),
                label if AUX_LABELS.contains(&label) => {
                    if let Some(main) = window.app_handle().get_webview_window(MAIN_LABEL) {
                        let _ = main.set_enabled(true);
                    }
                }
                _ => {}
            },
            _ => {}
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
            commands::autostart_state,
            commands::set_autostart,
            update::check_update,
            update::update_status,
            update::download_update,
            update::install_update,
            update::open_url,
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
