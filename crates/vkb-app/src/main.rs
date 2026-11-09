#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod i18n;
mod running_apps;
#[cfg(target_os = "macos")]
mod input_monitoring;
#[cfg(target_os = "macos")]
mod macos_shell;
mod tray;
// Store (MSIX) build: no self-updater. The stub keeps only the link opener the
// About window uses; it has no network code at all.
#[cfg(not(feature = "store"))]
mod update;
#[cfg(feature = "store")]
#[path = "update_store.rs"]
mod update;
mod window_bg;
#[cfg(any(windows, target_os = "macos"))]
mod autostart;
#[cfg(target_os = "macos")]
mod macos_backend;
#[cfg(target_os = "macos")]
mod singleton;
#[cfg(windows)]
mod windows_backend;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use tauri::{Emitter, Manager, PhysicalPosition, RunEvent, WindowEvent};

/// Set only by the tray's quit item. Everything else is "hide to tray".
static QUITTING: AtomicBool = AtomicBool::new(false);

/// The one label the settings window ever uses. Closing it destroys it; the
/// next open builds a new window under the same label.
const MAIN_LABEL: &str = "main";

/// Height of the custom caption the page draws on Windows. macOS windows use the
/// overlay title bar (traffic lights over the page) plus a page-drawn caption row
/// of `MAC_CAPTION_PX` instead.
const CAPTION_PX: f64 = 36.0;

/// Height of the caption row the page draws under the overlay title bar on macOS
/// (`.cap-bar.mac` in ui/src/styles/mac.css). With `TitleBarStyle::Overlay` the
/// webview covers the whole frame, so the window height is the page height. The
/// page still measures itself and grows the window if needed (ui/src/lib/fit.ts).
#[cfg(target_os = "macos")]
const MAC_CAPTION_PX: f64 = 28.0;

/// Window height to request for the current platform: the measured Windows height
/// (caption included) on Windows; on macOS the same page height without the
/// caption, plus the 28 pt overlay caption row.
fn content_height(windows_height: f64) -> f64 {
    #[cfg(target_os = "macos")]
    return windows_height - CAPTION_PX + MAC_CAPTION_PX;
    #[cfg(not(target_os = "macos"))]
    {
        let _ = CAPTION_PX;
        windows_height
    }
}

/// Inner width of the single settings window.
const MAIN_WIDTH: f64 = 560.0;

/// Initial content height; the page fits its own height on load.
fn main_content_height() -> f64 {
    content_height(460.0)
}

/// macOS: the typing engine needs both Accessibility (post the converted text)
/// and Input Monitoring (see the keys). Either one missing means the banner.
#[cfg(target_os = "macos")]
fn permissions_missing() -> bool {
    // Input Monitoring comes from the cache (`input_monitoring.rs`): the TCC
    // query costs ~20 ms on the main thread right before the window build. The
    // refresh lands well before the page asks for the status.
    let im_granted = input_monitoring::granted();
    input_monitoring::refresh();
    !vkb_macos::has_accessibility_permission() || !im_granted
}


/// A window that is built but not shown yet. It stays hidden until its page
/// reports `window_ready` (or the fallback timer fires): a window that appears
/// before the page has painted its content shows a blank WebView first, which is
/// the "empty window, then 1-2 s until the GUI" the owner reported.
struct Pending {
    label: String,
    /// Tells this window's fallback timer apart from a timer left by an earlier
    /// window that had the same label.
    id: u64,
    /// Start of the build: the reference point for every timing log.
    started: Instant,
}

static PENDING: parking_lot::Mutex<Vec<Pending>> = parking_lot::Mutex::new(Vec::new());
static NEXT_PENDING_ID: AtomicU64 = AtomicU64::new(1);

/// Longest a built window stays hidden waiting for its page. A page that never
/// reports (a script error, a lost IPC reply) must not leave an invisible window.
const READY_FALLBACK_MS: u64 = 1500;

/// macOS: how often the shell checks that a staged (alpha 0) window is
/// occlusion-visible, the first check this long after staging. WebKit runs no
/// animation frames and no painting for a window the window server reports as
/// occluded, so its page can never report ready. That happens when the app
/// does not finish activating (seen on fresh processes activated without a user
/// action): the window never turns key and stays occluded until shown. Such a
/// window is shown at once (`cause=early`), blank until it paints, which is
/// never worse than showing it at build. Measured on cold opens: a healthy
/// staged window turns occlusion-visible 70-170 ms after staging (once 400 ms,
/// which this shows early, blank for ~100 ms: the price of the bound).
#[cfg(target_os = "macos")]
const OCCLUSION_CHECK_MS: u64 = 300;

/// Keep the main window hidden this many seconds after the user closes it, so a
/// quick reopen shows it without a rebuild. 0 (shipped) destroys it at once, as
/// charter P3 requires. Cost while kept: the whole WebView2 stack stays resident
/// (the tray idle goes from ~15 MB towards ~31 MB, see `release_working_set`), and
/// on macOS the Dock icon stays because the hidden window still counts as open.
const KEEP_CLOSED_MAIN_SECS: u64 = 0;

/// Bumped on every close that keeps the window, so only the latest timer may
/// destroy it.
static KEPT_GENERATION: AtomicU64 = AtomicU64::new(0);

fn millis_since(started: Instant) -> u64 {
    started.elapsed().as_millis() as u64
}

fn is_pending(label: &str) -> bool {
    PENDING.lock().iter().any(|pending| pending.label == label)
}

/// Record a freshly built window and arm its fallback timer. `id` `None` on the
/// page's own report (matches by label), the timer passes its id.
fn hold_until_ready(app: &tauri::AppHandle, label: &str, started: Instant) {
    // macOS: a hidden WKWebView runs no animation frames (and a cold one may not
    // start its navigation), so the page could never report a painted frame.
    // Order the window in at alpha 0 instead, already in place, and let `reveal`
    // turn it opaque.
    #[cfg(target_os = "macos")]
    if let Some(window) = app.get_webview_window(label) {
        macos_shell::stage_transparent(&window);
        // MAC-REVIEW: only the main window is staged now.
        centre_on_launch_monitor(&window);
        centre_on_launch_monitor(&window);
    }

    let id = NEXT_PENDING_ID.fetch_add(1, Ordering::SeqCst);
    PENDING.lock().push(Pending { label: label.to_string(), id, started });
    tracing::info!(window = label, build_ms = millis_since(started), "window.built");

    // macOS safety net: a staged window that WebKit is throttling is shown now
    // rather than at the fallback.
    #[cfg(target_os = "macos")]
    schedule_occlusion_check(app, label, id);

    let app = app.clone();
    let label = label.to_string();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(READY_FALLBACK_MS));
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || reveal(&handle, &label, Some(id), "fallback"));
    });
}

/// macOS: every `OCCLUSION_CHECK_MS` while the window is still staged, show it
/// at once if the window server reports it occluded.
#[cfg(target_os = "macos")]
fn schedule_occlusion_check(app: &tauri::AppHandle, label: &str, id: u64) {
    let app = app.clone();
    let label = label.to_string();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(OCCLUSION_CHECK_MS));
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || {
            let staged = PENDING.lock().iter().any(|pending| pending.label == label && pending.id == id);
            if !staged {
                return;
            }
            let visible = handle
                .get_webview_window(&label)
                .is_some_and(|window| macos_shell::occlusion_visible(&window));
            if visible {
                // Keep watching: the window can still lose the state.
                schedule_occlusion_check(&handle, &label, id);
            } else {
                reveal(&handle, &label, Some(id), "early");
            }
        });
    });
}

fn take_pending(label: &str, id: Option<u64>) -> Option<Pending> {
    let mut pending = PENDING.lock();
    let index = pending
        .iter()
        .position(|entry| entry.label == label && id.map_or(true, |id| entry.id == id))?;
    Some(pending.remove(index))
}

/// Show a held window: place it and bring it forward. Runs once per build; a later call finds nothing pending.
///
/// Order for the main window: show, then centre, then focus. Showing first matters
/// because a window that has never been shown has not had its DWM frame applied
/// yet, so `outer_size()` lies (measured 918x710 for a 720x560 request) and
/// centring before the show would land ~90 px off. Runs only after the page is
/// ready, so the first frame on screen already has the real content.
fn reveal(app: &tauri::AppHandle, label: &str, id: Option<u64>, cause: &'static str) {
    let Some(pending) = take_pending(label, id) else { return };
    let Some(window) = app.get_webview_window(label) else { return };

    // macOS: the staged window turns opaque in place, already painted. First,
    // before the activation below: macOS may refuse to activate an app whose
    // only window is transparent, and an app that is not active leaves its
    // window behind the frontmost app, occluded and never painted.
    #[cfg(target_os = "macos")]
    macos_shell::reveal_staged(&window);

    enter_foreground(app);
    if let Err(error) = window.show() {
        tracing::warn!(%error, "could not show the settings window");
    }
    // Two passes: a window that has never been shown sits on the monitor it was
    // created on, and `outer_size()` answers in that monitor's DPI. Moving it
    // to the cursor's monitor first, then centring again, measures the size it
    // actually has there (94 px off on a 125% -> 100% pair otherwise).
    // macOS placed it already while staging it (`hold_until_ready`).
    #[cfg(not(target_os = "macos"))]
    {
        centre_on_launch_monitor(&window);
        centre_on_launch_monitor(&window);
    }
    if let Err(error) = window.set_focus() {
        tracing::warn!(%error, "could not focus the settings window");
    }

    tracing::info!(window = label, cause, since_build_ms = millis_since(pending.started), "window.shown");
}

/// The page calls this once its real content is on screen (`App.svelte`). Shows
/// the window it came from; a no-op for a window that is already shown.
#[tauri::command]
fn window_ready(window: tauri::WebviewWindow) {
    let label = window.label().to_string();
    let since_build = PENDING
        .lock()
        .iter()
        .find(|pending| pending.label == label)
        .map(|pending| millis_since(pending.started));
    if let Some(ms) = since_build {
        tracing::info!(window = %label, since_build_ms = ms, "window.page_ready");
    }
    reveal(window.app_handle(), &label, None, "ready");
}

/// Hide the closed main window instead of destroying it (only when
/// `KEEP_CLOSED_MAIN_SECS` > 0), and destroy it once that time has passed unless
/// it was reopened or closed again in the meantime.
fn keep_main_hidden(app: &tauri::AppHandle, api: &tauri::CloseRequestApi) {
    api.prevent_close();
    if let Some(window) = app.get_webview_window(MAIN_LABEL) {
        if let Err(error) = window.hide() {
            tracing::warn!(%error, "could not hide the settings window");
        }
    }
    let generation = KEPT_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(KEEP_CLOSED_MAIN_SECS));
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || {
            let current = KEPT_GENERATION.load(Ordering::SeqCst) == generation;
            let Some(window) = handle.get_webview_window(MAIN_LABEL) else { return };
            if current && !window.is_visible().unwrap_or(true) {
                if let Err(error) = window.destroy() {
                    tracing::warn!(%error, "could not destroy the kept settings window");
                }
            }
        });
    });
}


/// Make the app a regular (Dock) app and bring it to the front. A window of an
/// Accessory app opens behind whatever app is frontmost and cannot take focus,
/// so every window that is about to show calls this first (D-MAC-04/05, M3).
#[cfg(target_os = "macos")]
fn enter_foreground(app: &tauri::AppHandle) {
    if let Err(error) = app.set_activation_policy(tauri::ActivationPolicy::Regular) {
        tracing::warn!(%error, "could not show the Dock icon");
    }
    macos_shell::activate(app);
}
#[cfg(not(target_os = "macos"))]
fn enter_foreground(_app: &tauri::AppHandle) {}

/// Back to a menu-bar-only app once no window is left. The window being
/// destroyed is told apart by identity (its NSWindow), not by label: a new
/// `main` built under the same label before the old one's `Destroyed` arrives
/// still counts.
#[cfg(target_os = "macos")]
fn leave_foreground_if_idle(app: &tauri::AppHandle, destroyed: &tauri::Window) {
    let gone = destroyed.ns_window().ok();
    let any = app.webview_windows().values().any(|w| match (w.ns_window().ok(), gone) {
        (Some(a), Some(b)) => a != b,
        _ => w.label() != destroyed.label(),
    });
    if !any {
        if let Err(error) = app.set_activation_policy(tauri::ActivationPolicy::Accessory) {
            tracing::warn!(%error, "could not hide the Dock icon");
        }
    }
}
#[cfg(not(target_os = "macos"))]
fn leave_foreground_if_idle(_app: &tauri::AppHandle, _destroyed: &tauri::Window) {}

/// Bring the settings window forward from anywhere (tray, hotkey, second
/// launch), building it if it is not there yet.
///
/// The window does not exist while the app sits in the tray: closing it tears
/// the WebView2 stack down (charter P3) instead of hiding it, so this is also
/// where it comes back.
pub fn show_settings(app: &tauri::AppHandle) {
    // macOS: build, staging (alpha 0) and order-in must happen in one main-thread
    // turn. Callers off it (hotkeys on the backend worker, a second launch from
    // the single-instance plugin's task) hop over first.
    #[cfg(target_os = "macos")]
    if !macos_shell::on_main_thread() {
        let handle = app.clone();
        if let Err(error) = app.run_on_main_thread(move || open_settings(&handle)) {
            tracing::warn!(%error, "could not open the settings window on the main thread");
        }
        return;
    }
    open_settings(app);
}

/// Open the settings window on the About tab (tray update / about items).
pub fn show_about(app: &tauri::AppHandle) {
    let _ = show_aux(app, "about");
}

/// Old callers (hotkeys, tray) name a tab; the page switches to it.
pub fn show_settings_tab(app: &tauri::AppHandle, tab: &str) {
    let _ = show_aux(app, tab);
}

/// Tab id of the page for a window kind / legacy tab name.
fn tab_for_kind(kind: &str) -> Option<&'static str> {
    match kind {
        "hotkeys" => Some("hotkeys"),
        "macros" => Some("macros"),
        "advanced" | "system" => Some("advanced"),
        "about" => Some("about"),
        "go" => Some("general"),
        _ => None,
    }
}

/// Tab a caller asked for, until the page takes it (`take_requested_tab`).
static REQUESTED_TAB: parking_lot::Mutex<Option<String>> = parking_lot::Mutex::new(None);

/// Page command: the tab requested while the window was still being built.
#[tauri::command]
fn take_requested_tab() -> Option<String> {
    REQUESTED_TAB.lock().take()
}

/// Bring the settings window forward on the tab for `kind`: store it for a page
/// still loading and tell a page that is already live.
pub fn show_aux(app: &tauri::AppHandle, kind: &str) -> Result<(), String> {
    let Some(tab) = tab_for_kind(kind) else {
        // Plain settings (no tab): keep the current tab.
        show_settings(app);
        return Ok(());
    };
    *REQUESTED_TAB.lock() = Some(tab.to_string());
    show_settings(app);
    // MAC-REVIEW: on macOS show_settings may hop to the main thread, so the window
    // can be built after this emit; the stored tab covers that case.
    if let Err(error) = app.emit_to(MAIN_LABEL, "vkb:tab", tab.to_string()) {
        tracing::debug!(%error, tab, "tab event not delivered");
    }
    Ok(())
}

fn open_settings(app: &tauri::AppHandle) {
    enter_foreground(app);
    if let Some(window) = app.get_webview_window(MAIN_LABEL) {
        // Built but still waiting for its page: `reveal` shows it.
        if is_pending(MAIN_LABEL) {
            return;
        }
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        return;
    }

    let built = build_settings_window(app);
    if built.is_ok() {
        // A stale `Destroyed` of the previous window may have dropped the
        // policy between the call above and the build: assert it again.
        enter_foreground(app);
    } else if let Err(error) = built {
        tracing::warn!(%error, "could not create the settings window");
        // A window whose destruction is still in flight refuses the label. It
        // is the same window the user asked for, so surface it instead of
        // leaving nothing on screen.
        if let Some(window) = app.get_webview_window(MAIN_LABEL) {
            if !is_pending(MAIN_LABEL) {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    }
}


/// Build the settings window from scratch: same label, same fixed 560-wide
/// frameless shell, centred on the launch monitor as it appears.
fn build_settings_window(app: &tauri::AppHandle) -> tauri::Result<tauri::WebviewWindow> {
    let started = std::time::Instant::now();

    let builder = tauri::WebviewWindowBuilder::new(
        app,
        MAIN_LABEL,
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title(i18n::t("rwin.main"))
    .inner_size(MAIN_WIDTH, main_content_height())
    .resizable(false)
    .maximizable(false)
    .decorations(cfg!(target_os = "macos"))
    // Frameless: without the native shadow the window has no edge against a
    // neighbouring app. `transparent` stays false, which is what Windows needs
    // for DWM to paint the frame shadow at all.
    .shadow(true)
    .transparent(false)
    .background_color(window_bg::initial(app))
    .theme(window_bg::configured_window_theme())
    .initialization_script(window_bg::init_script())
    // Hidden at build time; placed and shown below in one go.
    .visible(false);
    // macOS: overlay title bar, the page draws the caption row (TitleBar.svelte).
    #[cfg(target_os = "macos")]
    let builder = builder
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .hidden_title(true);
    let window = builder.build()?;
    // Still hidden: settle the exact colour for the OS theme before it shows.
    window_bg::apply(&window);
    // Not shown here. `reveal` shows and places it once the page reports ready:
    // the placement rules (show first, then centre, see `reveal`) are unchanged,
    // they just run after the first real frame is ready instead of before it.
    hold_until_ready(app, MAIN_LABEL, started);
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

    // The work area (S-20): the menu bar / Dock / taskbar are not part of it, so
    // the window is centred in what the user can actually see. Position and size
    // are physical pixels of that monitor, like `outer_size`.
    let mon_pos = monitor.position();
    let mon_size = monitor.size();
    let area = monitor.work_area();
    let x = centred_axis(area.position.x, area.size.width, win.width);
    let y = centred_axis(area.position.y, area.size.height, win.height);

    tracing::info!(
        monitor = ?monitor.name(),
        mon_x = mon_pos.x,
        mon_y = mon_pos.y,
        mon_w = mon_size.width,
        mon_h = mon_size.height,
        work_x = area.position.x,
        work_y = area.position.y,
        work_w = area.size.width,
        work_h = area.size.height,
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
    #[cfg(target_os = "macos")]
    if let Err(error) = vkb_macos::uninstall_hook() {
        tracing::warn!(%error, "could not remove the event tap");
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

#[cfg(all(feature = "store", feature = "dev-channel"))]
compile_error!("the `store` and `dev-channel` features cannot be combined");

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "vietkb=info,vkb_windows=info,vkb_macos=info".into()),
        )
        .init();

    // P2 latency probe: reads `VKB_PERF` once, here. Off unless the variable is
    // `1`, in which case the keystroke path records samples and the exit path
    // writes `perf-<timestamp>.tsv`.
    #[cfg(windows)]
    {
        // Both opt-in debug outputs (`perf-*.tsv`, `keys.log`) write under the
        // channel-aware data directory, never a hardcoded `%APPDATA%\VietKB`.
        vkb_windows::set_data_dir(vkb_config::paths::config_dir());
        vkb_windows::perf::init_from_env();
    }
    #[cfg(target_os = "macos")]
    {
        vkb_macos::set_data_dir(vkb_config::paths::config_dir());
        // First Input Monitoring reading, off the main thread.
        input_monitoring::refresh();
    }

    // M8: serialise concurrent launches before the single-instance plugin can be
    // raced into creating a second tray icon / event tap (singleton.rs).
    #[cfg(target_os = "macos")]
    singleton::guard();

    // Second launch. Windows: same rule as a first one, so `Chạy ẩn` keeps a
    // re-launch in the tray instead of flashing a window. macOS (S-06): the
    // user just opened the app from Finder/Spotlight, and a menu-bar app
    // with nothing on screen looks like it did nothing, so always show it.
    let builder = tauri::Builder::default().plugin(tauri_plugin_single_instance::init(
        |app, _args, _cwd| {
            if cfg!(target_os = "macos")
                || !commands::config_cell().read().system.start_minimized
            {
                show_settings(app);
            }
        },
    ));
    // The `--autostart` argument identifies a Windows launch in the `Run`
    // value. It no longer decides what appears on screen: `Chạy ẩn` does
    // that for every launch path alike. Not registered in the Store build:
    // autostart there is the manifest's StartupTask (`autostart.rs`), and the
    // plugin would write the virtualized `Run` key.
    #[cfg(not(feature = "store"))]
    let builder = builder.plugin(
        tauri_plugin_autostart::Builder::new()
            .arg("--autostart")
            .build(),
    );
    let app = builder
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        // No updater plugin: the GitHub Releases check lives in `update.rs`
        // (plain `ureq`), so `plugins.updater` is never needed in the config.
        .setup(|app| {
            let handle = app.handle().clone();
            // M3: menu-bar extra only. No Dock icon, no app menu bar; the tray
            // item and the settings window are the whole surface.
            #[cfg(target_os = "macos")]
            handle.set_activation_policy(tauri::ActivationPolicy::Accessory)?;
            commands::set_app_handle(handle.clone());

            let config = commands::config_cell().read().clone();
            #[cfg(target_os = "macos")]
            macos_shell::apply_theme(&handle, &config.system.theme);

            // The `Run` key is the truth for autostart, not the config file: a
            // user who removed the entry in Windows' own startup settings must
            // see it off, not get it silently re-created. Adopt it before the
            // first window is built, so the switch never lies.
            commands::adopt_autostart_registry(config.system.start_with_windows);

            tray::install(&handle)?;

            // Five seconds after launch, then every 12 h, on its own thread,
            // honouring `system.check_updates`. A failure stays silent and never
            // hides an update already found.
            // The Store build never reaches the network: Store updates it.
            #[cfg(not(feature = "store"))]
            update::schedule_startup_check(handle.clone());

            // Hook, config hand-off and both callbacks. Must run after the tray
            // exists (a hotkey can flip it) and after the config load.
            #[cfg(windows)]
            windows_backend::start();
            // Event tap, config hand-off, both callbacks. Without Accessibility
            // the install fails quietly, the banner shows, and `ensure_installed`
            // retries when the grant appears.
            #[cfg(target_os = "macos")]
            macos_backend::start();

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
            // macOS exception (S-04, M3): without the Accessibility / Input
            // Monitoring grants the app cannot type at all, so the window opens
            // with its banner even when `Chạy ẩn` is on.
            #[cfg(target_os = "macos")]
            let needs_permissions = permissions_missing();
            #[cfg(not(target_os = "macos"))]
            let needs_permissions = false;
            if !config.system.start_minimized || needs_permissions {
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
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                if window.label() == MAIN_LABEL {
                    // Off by default (`KEEP_CLOSED_MAIN_SECS` = 0): destroyed as before.
                    if KEEP_CLOSED_MAIN_SECS != 0 {
                        keep_main_hidden(window.app_handle(), api);
                    }
                }
                // Closed to the tray, not quit: flush the P2 report now so a
                // force-kill afterwards still leaves the file. No-op without
                // `VKB_PERF=1`, and it runs on the backend worker.
                #[cfg(windows)]
                crate::windows_backend::flush_perf();
            }
            WindowEvent::Destroyed => {
                // A window that was capturing a shortcut cannot say it stopped.
                commands::clear_hotkey_capture_for(window.label());
                // A tab request nobody took must not leak into the next window.
                if window.label() == MAIN_LABEL {
                    REQUESTED_TAB.lock().take();
                }
                leave_foreground_if_idle(window.app_handle(), window);
            }
            // Focus lost: a capture field in this window cannot be recording any
            // more, whether or not the page managed to send a blur.
            WindowEvent::Focused(false) => commands::clear_hotkey_capture_for(window.label()),
            #[cfg(target_os = "macos")]
            WindowEvent::Focused(true) => {
                // Back from System Settings, maybe with a new grant.
                // MAC-REVIEW: refocus_modal call removed with the modal stack.
                input_monitoring::refresh();
            }
            // The OS theme flipped: with theme=System the native background follows.
            WindowEvent::ThemeChanged(os) => window_bg::on_os_theme(window, *os),
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
            commands::platform,
            commands::accessibility_status,
            commands::request_accessibility,
            commands::set_hotkey_capture,
            commands::input_monitoring_status,
            commands::request_input_monitoring,
            commands::open_privacy_pane,
            window_ready,
            take_requested_tab,
            #[cfg(not(feature = "store"))]
            update::check_update,
            #[cfg(not(feature = "store"))]
            update::update_status,
            #[cfg(not(feature = "store"))]
            update::download_update,
            #[cfg(not(feature = "store"))]
            update::install_update,
            update::open_url,
        ])
        .build(tauri::generate_context!())
        .expect("error while building VietKB");

    app.run(|_handle, event| match event {
        // Dock click with no window on screen (macOS): the settings window.
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { has_visible_windows, .. } => {
            if !has_visible_windows {
                show_settings(_handle);
            }
        }
        // Destroying the last window asks for an exit. In tray mode the answer
        // is no: the tray icon and the keyboard hook are the app, and the
        // settings window comes back on demand. A requested exit (the tray's
        // quit item) has a code and is always honoured.
        //
        // S-05, verified in tao 0.37 / tauri-runtime-wry 2.12: this event is
        // raised only when the last window is destroyed (`code: None`) or on
        // `RequestExit(code)`. Cmd+Q, Dock > Quit and logout/shutdown run
        // `NSApplication terminate:` -> `applicationWillTerminate` -> process
        // exit; tao implements no `applicationShouldTerminate`, so they never
        // reach this handler and cannot be refused here.
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
            #[cfg(target_os = "macos")]
            if let Err(error) = vkb_macos::uninstall_hook() {
                tracing::warn!(%error, "could not remove the event tap");
            }
        }
        _ => {}
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_held_window_is_taken_once_and_only_by_its_own_timer() {
        // Ids far above the counter range, so no other test can collide with them.
        PENDING.lock().push(Pending {
            label: "t-held".into(),
            id: 900_001,
            started: Instant::now(),
        });
        assert!(take_pending("t-other", None).is_none());
        assert!(take_pending("t-held", Some(900_002)).is_none(), "a stale timer must not show it");
        assert!(take_pending("t-held", Some(900_001)).is_some());
        assert!(take_pending("t-held", None).is_none(), "shown once");
    }

    #[test]
    fn centres_inside_the_work_area_even_when_it_does_not_start_at_zero() {
        // Menu bar 25 px tall: the work area starts below the monitor origin.
        assert_eq!(centred_axis(25, 975, 400), 25 + 287);
        // Negative origin (monitor left of the primary).
        assert_eq!(centred_axis(-1920, 1920, 480), -1920 + 720);
        // Larger than the area: pinned to its origin, not half off-screen.
        assert_eq!(centred_axis(25, 300, 400), 25);
    }

    #[test]
    fn window_kinds_map_to_page_tabs() {
        assert_eq!(tab_for_kind("hotkeys"), Some("hotkeys"));
        assert_eq!(tab_for_kind("macros"), Some("macros"));
        assert_eq!(tab_for_kind("advanced"), Some("advanced"));
        assert_eq!(tab_for_kind("system"), Some("advanced"));
        assert_eq!(tab_for_kind("about"), Some("about"));
        assert_eq!(tab_for_kind("go"), Some("general"));
        assert_eq!(tab_for_kind("other"), None);
    }
}
