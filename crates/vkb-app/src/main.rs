#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod i18n;
mod running_apps;
#[cfg(target_os = "macos")]
mod macos_shell;
mod tray;
mod update;
#[cfg(any(windows, target_os = "macos"))]
mod autostart;
#[cfg(target_os = "macos")]
mod macos_backend;
#[cfg(target_os = "macos")]
mod singleton;
#[cfg(windows)]
mod windows_backend;

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{Manager, PhysicalPosition, RunEvent, WindowEvent};

/// Set only by the tray's quit item. Everything else is "hide to tray".
static QUITTING: AtomicBool = AtomicBool::new(false);

/// The one label the settings window ever uses. Closing it destroys it; the
/// next open builds a new window under the same label.
const MAIN_LABEL: &str = "main";

/// Height of the custom caption the page draws on Windows. macOS windows use the
/// native title bar (traffic lights) instead, so the page there is that much shorter.
const CAPTION_PX: f64 = 36.0;

/// Estimated native macOS title bar height in points. `inner_size` on a decorated
/// macOS window is the whole frame (title bar included), while the page only gets
/// the area below it, so the frame needs this much on top of the page height
/// (D-MAC-02: without it the footer row was clipped). It is only an estimate, the
/// real value differs between macOS releases: the page measures itself and grows
/// the window by whatever is still hidden (ui/src/lib/fit.ts).
#[cfg(target_os = "macos")]
const TITLEBAR_PT: f64 = 28.0;

/// Height of the slim row that holds the VI | EN switch at the top of the page on
/// macOS (`.mac-langbar` in ui/src/styles/mac.css). Keep equal.
#[cfg(target_os = "macos")]
const MAC_LANGBAR_PX: f64 = 28.0;

/// Window height to request for the current platform: the measured Windows height
/// (caption included) on Windows; on macOS the same page height without the
/// caption, plus the native title bar.
fn content_height(windows_height: f64) -> f64 {
    #[cfg(target_os = "macos")]
    return windows_height - CAPTION_PX + TITLEBAR_PT + MAC_LANGBAR_PX;
    #[cfg(not(target_os = "macos"))]
    {
        let _ = CAPTION_PX;
        windows_height
    }
}

/// Extra main-window height while the Accessibility banner is up (macOS). Keep
/// equal to `AX_BANNER_PX` in ui/src/components/AccessibilityBanner.svelte,
/// which resizes the window again when the grant appears or goes away.
#[cfg(target_os = "macos")]
const BANNER_PX: f64 = 80.0;

/// Content height of the settings window. macOS without the Accessibility grant
/// opens already tall enough for the banner, so it never visibly grows.
fn main_content_height() -> f64 {
    #[cfg(target_os = "macos")]
    if permissions_missing() {
        return content_height(460.0) + BANNER_PX;
    }
    content_height(460.0)
}

/// macOS: the typing engine needs both Accessibility (post the converted text)
/// and Input Monitoring (see the keys). Either one missing means the banner.
#[cfg(target_os = "macos")]
fn permissions_missing() -> bool {
    !vkb_macos::has_accessibility_permission() || !vkb_macos::has_input_monitoring_permission()
}

/// Disable the window a modal was opened from.
///
/// Windows: `EnableWindow(false)`. macOS: no sheet. `set_enabled(false)` there
/// begins a half-transparent sheet the size of the parent; it is ordered above
/// the parent's child windows too, so it painted over the modal and the modal
/// looked hidden (D-MAC-04). The modal stays above its parent as a child window
/// and `refocus_modal` hands focus back to it whenever any other VietKB window
/// is activated while a modal is open.
fn disable_for_modal(owner: &tauri::WebviewWindow) {
    #[cfg(not(target_os = "macos"))]
    if let Err(error) = owner.set_enabled(false) {
        tracing::warn!(%error, "could not disable the owner window for a modal");
    }
    #[cfg(target_os = "macos")]
    let _ = owner;
}

/// One open modal: who opened it (its owner / parent window) and whether it has
/// been asked to close. A closing modal must not win the focus back (it would be
/// re-focused on its way out and swallow the close).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Modal {
    label: String,
    owner: Option<String>,
    closing: bool,
}

/// Open modals, oldest first. A modal opened from another modal (About from
/// Advanced) is owned by it, so the stack is a chain, not a set of siblings
/// (S-16).
static MODALS: parking_lot::Mutex<Vec<Modal>> = parking_lot::Mutex::new(Vec::new());

/// The window a new modal `label` belongs to: the topmost open modal (so a
/// modal opened from a modal stacks on it), else the main window if it exists.
fn modal_owner(stack: &[Modal], label: &str, main_exists: bool) -> Option<String> {
    stack
        .iter()
        .rev()
        .find(|modal| !modal.closing && modal.label != label)
        .map(|modal| modal.label.clone())
        .or_else(|| main_exists.then(|| MAIN_LABEL.to_string()))
}

/// The topmost modal that is not closing: the one that owns the focus.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn top_modal_of(stack: &[Modal]) -> Option<String> {
    stack
        .iter()
        .rev()
        .find(|modal| !modal.closing)
        .map(|modal| modal.label.clone())
}

/// Whether a modal that is not closing still has `owner` as its owner, i.e.
/// the owner must stay disabled.
#[cfg_attr(target_os = "macos", allow(dead_code))]
fn owner_has_open_modal(stack: &[Modal], owner: &str) -> bool {
    stack
        .iter()
        .any(|modal| !modal.closing && modal.owner.as_deref() == Some(owner))
}

fn modal_opened(label: &str, owner: Option<String>) {
    let mut modals = MODALS.lock();
    modals.retain(|modal| modal.label != label);
    modals.push(Modal { label: label.to_string(), owner, closing: false });
}

fn modal_closing(label: &str) {
    if let Some(entry) = MODALS.lock().iter_mut().find(|modal| modal.label == label) {
        entry.closing = true;
    }
}

/// Forget a destroyed modal; answers with the window that owned it.
fn modal_gone(label: &str) -> Option<String> {
    let mut modals = MODALS.lock();
    let owner = modals.iter().find(|modal| modal.label == label).and_then(|m| m.owner.clone());
    modals.retain(|modal| modal.label != label);
    owner
}

/// Modality without a disabled parent: when any VietKB window other than the
/// topmost modal is activated, the topmost modal takes the focus back. Covers
/// the main window behind a modal, a modal behind a modal it opened, and a
/// modal opened from the menu before the main window existed (S-16). Windows
/// disables the owner instead and needs none of this.
#[cfg(target_os = "macos")]
fn refocus_modal(app: &tauri::AppHandle, activated: &str) {
    let Some(top) = top_modal_of(&MODALS.lock()) else { return };
    if top == activated {
        return;
    }
    if let Some(modal) = app.get_webview_window(&top) {
        let _ = modal.set_focus();
    }
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
    enter_foreground(app);
    if let Some(window) = app.get_webview_window(MAIN_LABEL) {
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
    //   advanced  36 caption + 1 + content 295 (after removing two 36 px rows
    //             and one 28 px row: 488 - 72 - 28) + 56 footer = 388 → 388
    //   macros    36 caption + 1 + 16 + 36 toolbar + 12 + 406 table
    //             (36 header + 8×46 rows + 2 border) + 9 + 28 footer
    //             + 16 sheet pad = 560 → 560
    //   about     36 caption + 1 + 12 pad + 493 measured content (story +
    //             update block + links) + 14 pad = 556, inside the 560 px hard
    //             cap the owner set; the sheet scrolls only if a longer
    //             changelog is clipped.
    let (label, title, width, height) = match kind {
        "hotkeys" => ("hotkeys", "rwin.hotkeys", 520.0, 360.0),
        // macOS drops the `Tương thích` section (33 px heading + 28 px row, 61 px,
        // rounded to the 4 px grid) because its only option is Windows-only.
        "advanced" => (
            "advanced",
            "rwin.advanced",
            560.0,
            if cfg!(target_os = "macos") { 328.0 } else { 388.0 },
        ),
        "macros" => ("macros", "rwin.macros", 720.0, 560.0),
        "about" => ("about", "rwin.about", 560.0, 556.0),
        other => return Err(i18n::tf("rwin.invalid", &[("kind", other)])),
    };

    enter_foreground(app);
    let main = app.get_webview_window(MAIN_LABEL);

    if let Some(window) = app.get_webview_window(label) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        // Already open: keep the owner it was opened with, and make it the
        // topmost modal again.
        let owner_label = MODALS
            .lock()
            .iter()
            .find(|modal| modal.label == label)
            .and_then(|modal| modal.owner.clone());
        if let Some(owner) = owner_label.as_deref().and_then(|name| app.get_webview_window(name)) {
            disable_for_modal(&owner);
        }
        modal_opened(label, owner_label);
        return Ok(());
    }

    // The window this modal belongs to (S-16): the topmost open modal, so About
    // opened from Advanced stacks on Advanced instead of becoming its sibling,
    // else the main window, else nothing (opened from the menu with no main).
    let owner_label = modal_owner(&MODALS.lock(), label, main.is_some());
    let owner = owner_label.as_deref().and_then(|name| app.get_webview_window(name));

    let url = tauri::WebviewUrl::App(format!("index.html?win={kind}").into());
    let mut builder = tauri::WebviewWindowBuilder::new(app, label, url)
        .title(i18n::t(title))
        .inner_size(width, content_height(height))
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        // macOS keeps the native title bar (traffic lights); the page hides its
        // own caption there (`data-platform="macos"`).
        .decorations(cfg!(target_os = "macos"))
        // Frameless windows get no system border, so the native shadow is the
        // only thing separating this window from whatever is behind it. The 1 px
        // ring the page draws on top is the edge itself (ui/src/styles/v4.css).
        .shadow(true)
        .transparent(false)
        .visible(false);

    // Owned by its opener: no orphan in the taskbar, and Windows tears it down
    // first when the owner goes away.
    if let Some(owner) = &owner {
        #[cfg(windows)]
        {
            builder = builder.owner(owner).map_err(|error| error.to_string())?;
        }
        // macOS has no owner/owned notion; `parent` adds it as a child window.
        #[cfg(target_os = "macos")]
        {
            builder = builder.parent(owner).map_err(|error| error.to_string())?;
        }
        #[cfg(not(any(windows, target_os = "macos")))]
        let _ = owner;
    }
    let window = builder.build().map_err(|error| error.to_string())?;

    window.show().map_err(|error| error.to_string())?;
    match owner.as_ref() {
        Some(parent) => centre_on_parent(&window, Some(parent)),
        // Opened from the tray: no parent to centre on, so centre on the screen
        // the window landed on instead of leaving it at the corner.
        None => centre_on_monitor(&window, app),
    }
    // Modal: the owner stops accepting input until this window is gone.
    if let Some(owner) = &owner {
        disable_for_modal(owner);
    }
    modal_opened(label, owner_label);
    enter_foreground(app);
    window.set_focus().map_err(|error| error.to_string())?;
    Ok(())
}

/// Centre a parentless modal (opened from the tray) on its monitor's work area.
///
/// An unplaced window's `current_monitor()` is wherever the system dropped it,
/// not where the user is (S-20), so this uses the same rule as the main window:
/// the monitor under the cursor, centred twice (DPI re-measure).
fn centre_on_monitor(window: &tauri::WebviewWindow, _app: &tauri::AppHandle) {
    centre_on_launch_monitor(window);
    centre_on_launch_monitor(window);
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
    .title(i18n::t("rwin.main"))
    .inner_size(480.0, main_content_height())
    .resizable(false)
    .maximizable(false)
    .decorations(cfg!(target_os = "macos"))
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
    vkb_macos::set_data_dir(vkb_config::paths::config_dir());

    // M8: serialise concurrent launches before the single-instance plugin can be
    // raced into creating a second tray icon / event tap (singleton.rs).
    #[cfg(target_os = "macos")]
    singleton::guard();

    let app = tauri::Builder::default()
        // Second launch. Windows: same rule as a first one, so `Chạy ẩn` keeps a
        // re-launch in the tray instead of flashing a window. macOS (S-06): the
        // user just opened the app from Finder/Spotlight, and a menu-bar app
        // with nothing on screen looks like it did nothing, so always show it.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if cfg!(target_os = "macos")
                || !commands::config_cell().read().system.start_minimized
            {
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

            // Five seconds after launch, off the UI thread, honouring
            // `system.check_updates`. A failure stays silent; the About window
            // shows the reason when the owner opens it.
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
        // Modals are owned by the main window and the main window is disabled
        // while one is open, so this is also where they are taken down: closing
        // the parent closes its modals first (no orphan window), and closing a
        // modal hands the main window its input back.
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { .. } => {
                modal_closing(window.label());
                if window.label() == MAIN_LABEL {
                    close_aux(window.app_handle());
                }
                // Closed to the tray, not quit: flush the P2 report now so a
                // force-kill afterwards still leaves the file. No-op without
                // `VKB_PERF=1`, and it runs on the backend worker.
                #[cfg(windows)]
                crate::windows_backend::flush_perf();
            }
            WindowEvent::Destroyed => {
                let owner = modal_gone(window.label());
                // A window that was capturing a shortcut cannot say it stopped.
                commands::clear_hotkey_capture_for(window.label());
                match window.label() {
                    MAIN_LABEL => close_aux(window.app_handle()),
                    // Windows: hand input back to whoever opened the modal (the
                    // main window, or the modal it was stacked on).
                    #[cfg(not(target_os = "macos"))]
                    label if AUX_LABELS.contains(&label) => {
                        let owner = owner.as_deref().unwrap_or(MAIN_LABEL);
                        // Another open modal may still belong to this owner.
                        if !owner_has_open_modal(&MODALS.lock(), owner) {
                            if let Some(owner) = window.app_handle().get_webview_window(owner) {
                                let _ = owner.set_enabled(true);
                            }
                        }
                    }
                    _ => {}
                }
                #[cfg(target_os = "macos")]
                let _ = owner;
                leave_foreground_if_idle(window.app_handle(), window);
            }
            // Focus lost: a capture field in this window cannot be recording any
            // more, whether or not the page managed to send a blur.
            WindowEvent::Focused(false) => commands::clear_hotkey_capture_for(window.label()),
            #[cfg(target_os = "macos")]
            WindowEvent::Focused(true) => {
                refocus_modal(window.app_handle(), window.label());
            }
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
            update::check_update,
            update::update_status,
            update::download_update,
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

    fn modal(label: &str, owner: Option<&str>, closing: bool) -> Modal {
        Modal { label: label.into(), owner: owner.map(Into::into), closing }
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
    fn a_modal_opened_from_a_modal_stacks_on_it() {
        let stack = vec![modal("advanced", Some("main"), false)];
        assert_eq!(modal_owner(&stack, "about", true).as_deref(), Some("advanced"));
        assert_eq!(top_modal_of(&stack).as_deref(), Some("advanced"));
    }

    #[test]
    fn a_modal_opens_on_main_when_alone_and_on_nothing_without_main() {
        assert_eq!(modal_owner(&[], "hotkeys", true).as_deref(), Some("main"));
        assert_eq!(modal_owner(&[], "hotkeys", false), None);
    }

    #[test]
    fn an_owner_stays_disabled_while_another_modal_belongs_to_it() {
        let stack = vec![modal("hotkeys", Some("main"), false)];
        assert!(owner_has_open_modal(&stack, "main"));
        assert!(!owner_has_open_modal(&stack, "advanced"));
        let closing = vec![modal("hotkeys", Some("main"), true)];
        assert!(!owner_has_open_modal(&closing, "main"));
    }

    #[test]
    fn closing_modals_neither_own_nor_take_the_focus() {
        let stack = vec![
            modal("advanced", Some("main"), false),
            modal("about", Some("advanced"), true),
        ];
        assert_eq!(top_modal_of(&stack).as_deref(), Some("advanced"));
        assert_eq!(modal_owner(&stack, "macros", true).as_deref(), Some("advanced"));
    }
}
