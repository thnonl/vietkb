//! `WH_KEYBOARD_LL` installation, message loop and watchdog.
//!
//! Design notes (the legacy hook's known weaknesses are fixed here):
//!
//! * The hook owns a **dedicated thread** with its own message loop. The hook is
//!   installed on that thread and every `CallNextHookEx` happens there.
//! * `install()` has a **hard timeout**; startup never hangs when the OS refuses
//!   the hook (P6: cold start to hook-ready ≤ 400 ms).
//! * A **watchdog** thread re-installs the hook when the worker dies (panic, or
//!   the OS unhooking after `LowLevelHooksTimeout`). The worker additionally
//!   re-installs itself every 30 s, so a silently dropped hook heals even if the
//!   thread is still alive.
//! * Hotkeys are matched **inside** the hook (no `RegisterHotKey`), so the
//!   trigger key can be swallowed before the app sees it and modifier-only
//!   combos work.
//! * Unhook runs on every exit path of the worker (`HookSlot::drop`), including
//!   thread unwind; a hard process kill is cleaned up by the OS when the
//!   installing thread disappears.

use std::cell::Cell;
use std::panic::AssertUnwindSafe;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, HHOOK, KBDLLHOOKSTRUCT, MSG, MWMO_INPUTAVAILABLE,
    MsgWaitForMultipleObjectsEx, PM_NOREMOVE, PM_REMOVE, PeekMessageW, PostThreadMessageW,
    QS_ALLINPUT, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, WH_KEYBOARD_LL, WM_QUIT,
};

use crate::globals;
use crate::state::HookRuntime;

/// Posted by `set_config` so the hook thread re-reads the config.
pub const WM_APP_CONFIG: u32 = 0x8000 + 1;

const INSTALL_TIMEOUT: Duration = Duration::from_millis(1500);
const INSTALL_FAILURE_GRACE: Duration = Duration::from_millis(300);
const LOOP_WAIT_MS: u32 = 200;
const HEARTBEAT_TIMEOUT: Duration = Duration::from_millis(2500);
const HOOK_REFRESH: Duration = Duration::from_secs(30);
const WATCHDOG_TICK: Duration = Duration::from_millis(400);

/// Runtime owned by the hook thread; `null` when no worker is running.
static RUNTIME: AtomicPtr<HookRuntime> = AtomicPtr::new(ptr::null_mut());
static WATCHDOG_STOP: AtomicBool = AtomicBool::new(false);
static WATCHDOG: parking_lot::Mutex<Option<thread::JoinHandle<()>>> = parking_lot::Mutex::new(None);

/// Install the hook. Idempotent; fails if the OS does not confirm within
/// [`INSTALL_TIMEOUT`].
pub fn install() -> Result<()> {
    // P2 probe: reads `VKB_PERF` once here (startup), never per keystroke.
    crate::perf::init_from_env();
    if globals::READY.load(Ordering::SeqCst) && globals::ALIVE.load(Ordering::SeqCst) {
        return Ok(());
    }
    globals::STOP.store(false, Ordering::SeqCst);
    WATCHDOG_STOP.store(false, Ordering::SeqCst);
    spawn_worker();

    let started = Instant::now();
    loop {
        if globals::READY.load(Ordering::SeqCst) {
            start_watchdog();
            return Ok(());
        }
        if started.elapsed() >= INSTALL_TIMEOUT {
            break;
        }
        if started.elapsed() >= INSTALL_FAILURE_GRACE
            && !globals::ALIVE.load(Ordering::SeqCst)
            && globals::THREAD_ID.load(Ordering::SeqCst) == 0
        {
            break;
        }
        thread::sleep(Duration::from_millis(5));
    }

    // Leave nothing half-installed.
    globals::STOP.store(true, Ordering::SeqCst);
    Err(anyhow!(
        "keyboard hook did not become ready within {:?}",
        INSTALL_TIMEOUT
    ))
}

/// Remove the hook and stop the watchdog. Safe when nothing is installed.
pub fn uninstall() {
    WATCHDOG_STOP.store(true, Ordering::SeqCst);
    globals::STOP.store(true, Ordering::SeqCst);

    let thread_id = globals::THREAD_ID.load(Ordering::SeqCst);
    if thread_id != 0 {
        unsafe {
            let _ = PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
        }
    }

    let deadline = Instant::now() + Duration::from_millis(1000);
    while globals::ALIVE.load(Ordering::SeqCst) && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }

    if let Some(handle) = WATCHDOG.lock().take() {
        let _ = handle.join();
    }
    globals::READY.store(false, Ordering::SeqCst);
}

pub fn is_ready() -> bool {
    globals::READY.load(Ordering::SeqCst)
}

/// Ask the hook thread to re-read `globals::CONFIG`.
pub fn notify_config_changed() {
    let thread_id = globals::THREAD_ID.load(Ordering::SeqCst);
    if thread_id != 0 {
        unsafe {
            let _ = PostThreadMessageW(thread_id, WM_APP_CONFIG, WPARAM(0), LPARAM(0));
        }
    }
}

// ---------------------------------------------------------------------------
// Worker thread
// ---------------------------------------------------------------------------

#[derive(Default)]
struct HookSlot {
    handle: Cell<Option<HHOOK>>,
}

impl HookSlot {
    fn install(&self) -> Result<()> {
        let handle = unsafe {
            SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(keyboard_proc),
                HINSTANCE::default(),
                0,
            )
        }
        .map_err(|err| anyhow!("SetWindowsHookExW failed: {err}"))?;
        self.handle.set(Some(handle));
        Ok(())
    }

    fn is_installed(&self) -> bool {
        self.handle.get().is_some()
    }

    fn refresh(&self) -> Result<()> {
        if let Some(old) = self.handle.take() {
            unsafe {
                let _ = UnhookWindowsHookEx(old);
            }
        }
        self.install()
    }
}

impl Drop for HookSlot {
    fn drop(&mut self) {
        if let Some(old) = self.handle.take() {
            unsafe {
                let _ = UnhookWindowsHookEx(old);
            }
        }
    }
}

fn spawn_worker() {
    if let Err(err) = thread::Builder::new()
        .name("vkb-hook".to_string())
        .spawn(worker_main)
    {
        tracing::error!(error = %err, "cannot spawn the hook thread");
    }
}

fn worker_main() {
    globals::ALIVE.store(true, Ordering::SeqCst);

    let config = globals::snapshot();
    let sink = globals::SINK.lock().take();
    let runtime_ptr = Box::into_raw(Box::new(HookRuntime::new(config, sink)));
    RUNTIME.store(runtime_ptr, Ordering::Release);

    // Create the thread's message queue before publishing its id, otherwise
    // PostThreadMessageW would race the queue creation.
    let mut msg = MSG::default();
    let _ = unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_NOREMOVE) };
    let thread_id = unsafe { GetCurrentThreadId() };
    globals::THREAD_ID.store(thread_id, Ordering::SeqCst);

    let hook = HookSlot::default();
    if let Err(err) = hook.install() {
        tracing::error!(error = %err, "low-level keyboard hook refused");
        teardown(runtime_ptr);
        return;
    }
    globals::READY.store(true, Ordering::SeqCst);

    let mut last_refresh = Instant::now();
    let mut running = true;

    while running && !globals::STOP.load(Ordering::Relaxed) {
        globals::HEARTBEAT.fetch_add(1, Ordering::Relaxed);

        unsafe {
            // Wake on input or every LOOP_WAIT_MS; `MWMO_INPUTAVAILABLE` keeps
            // already-queued messages from being missed.
            MsgWaitForMultipleObjectsEx(
                None,
                LOOP_WAIT_MS,
                QS_ALLINPUT,
                MWMO_INPUTAVAILABLE,
            );
        }

        loop {
            let has_message = unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE) }.as_bool();
            if !has_message {
                break;
            }
            if msg.message == WM_QUIT {
                running = false;
                break;
            }
            if msg.message == WM_APP_CONFIG {
                let config = globals::snapshot();
                let ptr = RUNTIME.load(Ordering::Acquire);
                if let Some(config) = config {
                    if !ptr.is_null() {
                        unsafe {
                            (*ptr).apply_config(config);
                        }
                    }
                }
                continue;
            }
            unsafe {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        // Re-install when the OS dropped the hook or a refresh failed.
        if !hook.is_installed() {
            if hook.install().is_ok() {
                globals::READY.store(true, Ordering::SeqCst);
                tracing::info!("keyboard hook re-installed");
            }
        } else if last_refresh.elapsed() >= HOOK_REFRESH {
            last_refresh = Instant::now();
            if let Err(err) = hook.refresh() {
                tracing::warn!(error = %err, "hook refresh failed; retrying");
                globals::READY.store(false, Ordering::SeqCst);
            }
        }
    }

    globals::READY.store(false, Ordering::SeqCst);
    drop(hook);
    teardown(runtime_ptr);
}

fn teardown(runtime_ptr: *mut HookRuntime) {
    RUNTIME.store(ptr::null_mut(), Ordering::Release);
    globals::THREAD_ID.store(0, Ordering::SeqCst);
    globals::READY.store(false, Ordering::SeqCst);
    unsafe {
        drop(Box::from_raw(runtime_ptr));
    }
    // Last: the watchdog may respawn as soon as this flips.
    globals::ALIVE.store(false, Ordering::SeqCst);
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // `code < 0`: the hook must pass the message on without touching it.
    if code < 0 {
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }
    let runtime_ptr = RUNTIME.load(Ordering::Acquire);
    if runtime_ptr.is_null() {
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    // A panic must never cross the FFI boundary (or kill the hook thread).
    // P2 probe: this is the single env-gated check on the hot path. `enabled()`
    // is a cached relaxed bool; with `VKB_PERF` unset the body is skipped, so no
    // clock read and no allocation happen for this keystroke.
    if crate::perf::enabled() {
        crate::perf::begin();
    }
    let swallow = std::panic::catch_unwind(AssertUnwindSafe(|| {
        let runtime = unsafe { &mut *runtime_ptr };
        let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        runtime.on_event(wparam.0 as u32, info).swallow
    }))
    .unwrap_or(false);

    if swallow {
        LRESULT(1)
    } else {
        unsafe { CallNextHookEx(None, code, wparam, lparam) }
    }
}

// ---------------------------------------------------------------------------
// Watchdog
// ---------------------------------------------------------------------------

fn start_watchdog() {
    let mut slot = WATCHDOG.lock();
    if slot.is_some() {
        return;
    }
    WATCHDOG_STOP.store(false, Ordering::SeqCst);
    match thread::Builder::new()
        .name("vkb-watchdog".to_string())
        .spawn(watchdog_loop)
    {
        Ok(handle) => *slot = Some(handle),
        Err(err) => tracing::error!(error = %err, "cannot spawn the hook watchdog"),
    }
}

fn watchdog_loop() {
    let mut last_beat = globals::HEARTBEAT.load(Ordering::Relaxed);
    let mut stale_since = Instant::now();

    while !WATCHDOG_STOP.load(Ordering::Relaxed) {
        thread::sleep(WATCHDOG_TICK);
        if WATCHDOG_STOP.load(Ordering::Relaxed) {
            break;
        }

        if !globals::ALIVE.load(Ordering::SeqCst) {
            // The worker died (panic, failed install, or the OS killed it).
            tracing::warn!("hook thread is gone; reinstalling");
            globals::READY.store(false, Ordering::SeqCst);
            spawn_worker();
            last_beat = globals::HEARTBEAT.load(Ordering::Relaxed);
            stale_since = Instant::now();
            continue;
        }

        let beat = globals::HEARTBEAT.load(Ordering::Relaxed);
        if beat != last_beat {
            last_beat = beat;
            stale_since = Instant::now();
        } else if stale_since.elapsed() >= HEARTBEAT_TIMEOUT {
            tracing::warn!("hook thread is not pumping messages; restarting it");
            let thread_id = globals::THREAD_ID.load(Ordering::SeqCst);
            if thread_id != 0 {
                unsafe {
                    let _ = PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
                }
            }
            stale_since = Instant::now();
        }
    }
}
