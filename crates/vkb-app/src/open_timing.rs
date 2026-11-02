//! TEMPORARY window-open timing probe (branch `mac-open-timing`, never merged).
//!
//! `VKB_OPEN_TIMING=1`: every window open gets a t0 (start of `open_settings` /
//! `show_aux`) and each phase prints one tab-separated line to stderr:
//!
//! `VKBT <label> <open#> <phase> <ms since t0> <duration ms or ->`
//!
//! The page sends its own marks (wall clock, so the IPC that delivers them is
//! not counted) through `perf_marks`. In this mode the event tap, the hotkeys
//! (they live in the tap) and the update check are never started, so the build
//! cannot intercept keys or raise a permission prompt.
//!
//! `VKB_OPEN_TIMING_LOOP=N` (with the above): open/close the main window N
//! times, then About and Hotkeys 3 times each on top of it, then quit.
//! `VKB_OPEN_TIMING_GAP_MS` sets the idle gap between main-window opens
//! (default 1000); `VKB_OPEN_TIMING_NO_MODALS=1` skips the modal rounds.
//!
//! Rerun (macOS, dev identity, never touches the real VietKB):
//!   cd crates/vkb-app && CARGO_TARGET_DIR=$PWD/../../target-timing \
//!     ../../ui/node_modules/.bin/tauri build --config tauri.dev.conf.json \
//!     --features dev-channel --bundles app
//!   VKB_OPEN_TIMING=1 VKB_OPEN_TIMING_LOOP=6 RUST_LOG=warn \
//!     ".../VietKB Dev.app/Contents/MacOS/vietkb" 2> timing.log
//!   python3 scripts/open_timing_analyze.py timing.log [timing2.log ...]

use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use parking_lot::{Condvar, Mutex};
use tauri::Manager;

struct Open {
    n: u32,
    t0: Instant,
    t0_epoch_ms: f64,
}

static OPENS: Mutex<Option<HashMap<String, Open>>> = Mutex::new(None);
static PAINTED: Mutex<Option<HashMap<String, u32>>> = Mutex::new(None);
static PAINTED_CV: Condvar = Condvar::new();

pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("VKB_OPEN_TIMING").as_deref() == Ok("1"))
}

pub fn loop_count() -> Option<u32> {
    if !enabled() {
        return None;
    }
    std::env::var("VKB_OPEN_TIMING_LOOP").ok()?.parse().ok().filter(|n| *n > 0)
}

fn epoch_ms() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64() * 1000.0).unwrap_or(0.0)
}

/// New t0 for `label`.
pub fn start(label: &str) {
    if !enabled() {
        return;
    }
    let mut opens = OPENS.lock();
    let map = opens.get_or_insert_with(HashMap::new);
    let n = map.get(label).map_or(1, |open| open.n + 1);
    map.insert(label.to_string(), Open { n, t0: Instant::now(), t0_epoch_ms: epoch_ms() });
    drop(opens);
    line(label, n, "t0", 0.0, None);
}

fn line(label: &str, n: u32, phase: &str, at: f64, dur: Option<f64>) {
    let dur = dur.map_or_else(|| "-".to_string(), |d| format!("{d:.2}"));
    eprintln!("VKBT\t{label}\t{n}\t{phase}\t{at:.2}\t{dur}");
}

/// Rust-side mark: ms since `label`'s t0.
pub fn mark(label: &str, phase: &str) {
    mark_dur(label, phase, None);
}

pub fn mark_dur(label: &str, phase: &str, dur: Option<Duration>) {
    if !enabled() {
        return;
    }
    let opens = OPENS.lock();
    let Some(open) = opens.as_ref().and_then(|map| map.get(label)) else { return };
    let at = open.t0.elapsed().as_secs_f64() * 1000.0;
    line(label, open.n, phase, at, dur.map(|d| d.as_secs_f64() * 1000.0));
}

/// Run `f`, logging its duration as `phase`.
pub fn timed<T>(label: &str, phase: &str, f: impl FnOnce() -> T) -> T {
    if !enabled() {
        return f();
    }
    let started = Instant::now();
    let out = f();
    mark_dur(label, phase, Some(started.elapsed()));
    out
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageMark {
    name: String,
    epoch_ms: f64,
    dur_ms: Option<f64>,
}

/// Page marks, sent in one batch after the first content frame.
#[tauri::command]
pub fn perf_marks(window: tauri::WebviewWindow, marks: Vec<PageMark>) {
    if !enabled() {
        return;
    }
    let label = window.label().to_string();
    let (n, t0_epoch, recv) = {
        let opens = OPENS.lock();
        let Some(open) = opens.as_ref().and_then(|map| map.get(&label)) else { return };
        (open.n, open.t0_epoch_ms, open.t0.elapsed().as_secs_f64() * 1000.0)
    };
    let mut painted = false;
    for m in &marks {
        line(&label, n, &format!("page:{}", m.name), m.epoch_ms - t0_epoch, m.dur_ms);
        painted |= m.name == "painted";
    }
    line(&label, n, "rust:marks-received", recv, None);
    if painted {
        PAINTED.lock().get_or_insert_with(HashMap::new).insert(label, n);
        PAINTED_CV.notify_all();
    }
}

fn current_n(label: &str) -> u32 {
    OPENS.lock().as_ref().and_then(|map| map.get(label)).map_or(0, |open| open.n)
}

fn wait_painted(label: &str, n: u32, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    let mut painted = PAINTED.lock();
    loop {
        if painted.as_ref().and_then(|map| map.get(label)).is_some_and(|got| *got >= n) {
            return true;
        }
        if PAINTED_CV.wait_until(&mut painted, deadline).timed_out() {
            return false;
        }
    }
}

fn on_main(app: &tauri::AppHandle, f: impl FnOnce(&tauri::AppHandle) + Send + 'static) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || f(&handle));
}

fn wait_gone(app: &tauri::AppHandle, label: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while app.get_webview_window(label).is_some() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn close(app: &tauri::AppHandle, label: &'static str) {
    on_main(app, move |app| {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.close();
        }
    });
    wait_gone(app, label);
}

/// One open of `label` via `open`, until painted + 500 ms.
fn one_open(app: &tauri::AppHandle, label: &'static str, open: fn(&tauri::AppHandle)) {
    let n = current_n(label) + 1;
    on_main(app, open);
    if !wait_painted(label, n, Duration::from_secs(15)) {
        eprintln!("VKBT-WARN\t{label}\t{n}\tno painted mark within 15 s");
    }
    std::thread::sleep(Duration::from_millis(200));
    report_focus(app, label, n);
    std::thread::sleep(Duration::from_millis(300));
}

/// After the open: is VietKB the active app, is the new window key, and what
/// does the system say is frontmost (`lsappinfo front`).
fn report_focus(app: &tauri::AppHandle, label: &'static str, n: u32) {
    let (tx, rx) = std::sync::mpsc::channel();
    on_main(app, move |app| {
        #[cfg(target_os = "macos")]
        let active = objc2::MainThreadMarker::new()
            .map(|mtm| objc2_app_kit::NSApplication::sharedApplication(mtm).isActive());
        #[cfg(not(target_os = "macos"))]
        let active: Option<bool> = None;
        let focused = app.get_webview_window(label).and_then(|w| w.is_focused().ok());
        let _ = tx.send((active, focused));
    });
    let (active, focused) = rx.recv_timeout(Duration::from_secs(2)).unwrap_or((None, None));
    let front = std::process::Command::new("lsappinfo")
        .arg("front")
        .output()
        .ok()
        .and_then(|out| {
            let asn = String::from_utf8_lossy(&out.stdout).trim().to_string();
            std::process::Command::new("lsappinfo")
                .args(["info", "-only", "name", &asn])
                .output()
                .ok()
        })
        .map(|out| {
            let text = String::from_utf8_lossy(&out.stdout).to_string();
            text.split('"').nth(1).unwrap_or("?").to_string()
        })
        .unwrap_or_default();
    eprintln!("VKBF\t{label}\t{n}\tapp_active={active:?}\twindow_focused={focused:?}\tfront={front}");
}

fn open_main(app: &tauri::AppHandle) {
    crate::show_settings(app);
}
fn open_about(app: &tauri::AppHandle) {
    let _ = crate::show_aux(app, "about");
}
fn open_hotkeys(app: &tauri::AppHandle) {
    let _ = crate::show_aux(app, "hotkeys");
}

/// Idle gap between a close and the next open (`VKB_OPEN_TIMING_GAP_MS`,
/// default 1000). A long gap shows what an open after real idle time costs.
fn gap() -> Duration {
    let ms = std::env::var("VKB_OPEN_TIMING_GAP_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(1000);
    Duration::from_millis(ms)
}

pub fn spawn_loop(app: tauri::AppHandle, count: u32) {
    std::thread::spawn(move || {
        // Let the app settle as a menu-bar app first, like a real tray click.
        std::thread::sleep(Duration::from_millis(2000));
        for _ in 0..count {
            one_open(&app, "main", open_main);
            close(&app, "main");
            std::thread::sleep(gap());
        }
        // Modals on top of the main window, the usual way they open.
        one_open(&app, "main", open_main);
        let modal_runs = if std::env::var("VKB_OPEN_TIMING_NO_MODALS").is_ok() { 0 } else { 3 };
        for (label, open) in [("about", open_about as fn(&_)), ("hotkeys", open_hotkeys as fn(&_))] {
            for _ in 0..modal_runs {
                one_open(&app, label, open);
                close(&app, label);
                std::thread::sleep(Duration::from_secs(1));
            }
        }
        close(&app, "main");
        std::thread::sleep(Duration::from_millis(500));
        eprintln!("VKBT-DONE");
        on_main(&app, |app| crate::quit(app));
    });
}
