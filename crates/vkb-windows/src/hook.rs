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
    /// `WH_MOUSE_LL` on its own thread (button-down boundary only). Optional: a
    /// refused mouse hook never blocks typing. Not part of the harness build,
    /// which must not observe the owner's mouse.
    #[cfg(not(feature = "e2e-harness"))]
    mouse: std::cell::RefCell<Option<mouse::MouseThread>>,
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

    /// Start the mouse thread, or restart it when it died. Its own check: the
    /// keyboard hook's state says nothing about the mouse hook.
    fn ensure_mouse(&self) {
        #[cfg(not(feature = "e2e-harness"))]
        {
            let mut slot = self.mouse.borrow_mut();
            if slot.as_ref().is_some_and(|thread| !thread.is_alive()) {
                *slot = None; // joins the dead thread
            }
            if slot.is_none() {
                *slot = mouse::MouseThread::start();
            }
        }
    }

    fn unhook_keyboard(&self) {
        if let Some(old) = self.handle.take() {
            unsafe {
                let _ = UnhookWindowsHookEx(old);
            }
        }
    }

    /// Re-install the keyboard hook (the mouse thread refreshes itself).
    fn refresh(&self) -> Result<()> {
        self.unhook_keyboard();
        self.install()
    }
}

impl Drop for HookSlot {
    fn drop(&mut self) {
        self.unhook_keyboard();
        // The mouse thread is torn down together with the keyboard hook.
        #[cfg(not(feature = "e2e-harness"))]
        {
            self.mouse.borrow_mut().take();
        }
    }
}

/// `WH_MOUSE_LL` on a thread of its own, so a stall on the keyboard side
/// (foreground probe, `SendInput` retries, engine work) can never lag the pointer
/// or count against the system's low-level hook timeout.
#[cfg(not(feature = "e2e-harness"))]
mod mouse {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{mpsc, Arc};
    use std::thread;
    use std::time::{Duration, Instant};

    use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::Threading::GetCurrentThreadId;
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, HHOOK, MSG, MWMO_INPUTAVAILABLE,
        MsgWaitForMultipleObjectsEx, PM_NOREMOVE, PM_REMOVE, PeekMessageW, PostThreadMessageW,
        QS_ALLINPUT, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, WH_MOUSE_LL,
        WM_QUIT,
    };

    use crate::globals;

    const REFRESH: Duration = Duration::from_secs(30);
    const RETRY: Duration = Duration::from_secs(5);

    pub(super) struct MouseThread {
        thread_id: u32,
        alive: Arc<AtomicBool>,
        join: Option<thread::JoinHandle<()>>,
    }

    impl MouseThread {
        pub(super) fn start() -> Option<Self> {
            let alive = Arc::new(AtomicBool::new(false));
            let flag = alive.clone();
            let (tx, rx) = mpsc::channel::<u32>();
            let join = thread::Builder::new()
                .name("vkb-mouse".to_string())
                .spawn(move || run(flag, tx))
                .map_err(|err| tracing::error!(error = %err, "cannot spawn the mouse hook thread"))
                .ok()?;
            let thread_id = rx.recv_timeout(Duration::from_millis(1000)).ok()?;
            Some(Self {
                thread_id,
                alive,
                join: Some(join),
            })
        }

        pub(super) fn is_alive(&self) -> bool {
            self.alive.load(Ordering::SeqCst)
        }
    }

    impl Drop for MouseThread {
        fn drop(&mut self) {
            unsafe {
                let _ = PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
            }
            if let Some(join) = self.join.take() {
                let _ = join.join();
            }
        }
    }

    fn install() -> Option<HHOOK> {
        match unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(proc_), HINSTANCE::default(), 0) } {
            Ok(hook) => Some(hook),
            Err(err) => {
                tracing::warn!(error = %err, "mouse hook refused; clicks will not end a word");
                None
            }
        }
    }

    fn run(alive: Arc<AtomicBool>, tx: mpsc::Sender<u32>) {
        let mut msg = MSG::default();
        let _ = unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_NOREMOVE) };
        let _ = tx.send(unsafe { GetCurrentThreadId() });
        alive.store(true, Ordering::SeqCst);

        let mut hook = install();
        let mut stamp = Instant::now();
        'pump: loop {
            unsafe {
                MsgWaitForMultipleObjectsEx(None, 200, QS_ALLINPUT, MWMO_INPUTAVAILABLE);
            }
            while unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE) }.as_bool() {
                if msg.message == WM_QUIT {
                    break 'pump;
                }
                unsafe {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
            // Own handle check: re-install when missing, refresh when old (the
            // OS can drop a hook silently after a timeout).
            match hook {
                None if stamp.elapsed() >= RETRY => {
                    hook = install();
                    stamp = Instant::now();
                }
                Some(old) if stamp.elapsed() >= REFRESH => {
                    unsafe {
                        let _ = UnhookWindowsHookEx(old);
                    }
                    hook = install();
                    stamp = Instant::now();
                }
                _ => {}
            }
        }
        if let Some(old) = hook {
            unsafe {
                let _ = UnhookWindowsHookEx(old);
            }
        }
        alive.store(false, Ordering::SeqCst);
    }

    /// Mouse button-down ends the current word. Nothing else happens here: no
    /// allocation, no lock, never swallowed; the keyboard path consumes the flag.
    unsafe extern "system" fn proc_(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code >= 0 {
            // WM_LBUTTONDOWN, WM_RBUTTONDOWN, WM_MBUTTONDOWN, WM_XBUTTONDOWN.
            if matches!(wparam.0 as u32, 0x0201 | 0x0204 | 0x0207 | 0x020B) {
                globals::note_mouse_down();
            }
        }
        unsafe { CallNextHookEx(None, code, wparam, lparam) }
    }
}

/// One keyboard event through the runtime: perf probe, panic guard, decision.
/// Shared by the real hook procedure and the in-process harness.
pub(crate) fn process_event(runtime: &mut HookRuntime, message: u32, info: &KBDLLHOOKSTRUCT) -> bool {
    // P2 probe: this is the single env-gated check on the hot path. `enabled()`
    // is a cached relaxed bool; with `VKB_PERF` unset the body is skipped, so no
    // clock read and no allocation happen for this keystroke.
    if crate::perf::enabled() {
        crate::perf::begin();
    }
    // A panic must never cross the FFI boundary (or kill the hook thread).
    std::panic::catch_unwind(AssertUnwindSafe(|| runtime.on_event(message, info).swallow))
        .unwrap_or(false)
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
    if crate::debuglog::enabled() {
        crate::debuglog::note("hook-ready");
    }

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

        // The mouse hook lives on its own thread; keep that thread alive.
        hook.ensure_mouse();

        // Re-install when the OS dropped the hook or a refresh failed.
        if !hook.is_installed() {
            if hook.install().is_ok() {
                globals::READY.store(true, Ordering::SeqCst);
                tracing::info!("keyboard hook re-installed");
            }
        } else if !cfg!(feature = "e2e-harness") && last_refresh.elapsed() >= HOOK_REFRESH {
            // (Skipped under the harness: a refresh re-installs the hook as the
            // newest one and would reorder the sandwich.)
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

    // Zero-focus harness: only events carrying the harness tag are ours; every
    // other event (physical keys, another VietKB's injections) is passed on
    // untouched.
    #[cfg(feature = "e2e-harness")]
    let harness_event = {
        let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        if !crate::e2e::admit(info.dwExtraInfo) {
            return unsafe { CallNextHookEx(None, code, wparam, lparam) };
        }
        true
    };

    let swallow = {
        let runtime = unsafe { &mut *runtime_ptr };
        let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        process_event(runtime, wparam.0 as u32, info)
    };

    #[cfg(feature = "e2e-harness")]
    if harness_event {
        crate::e2e::note_decision(swallow);
    }

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

#[cfg(all(test, not(feature = "e2e-harness")))]
mod tests {
    /// A shipped build stamps the production marker on its injections.
    #[test]
    fn default_build_uses_the_production_marker() {
        assert_eq!(crate::input::injected_tag(), crate::input::INJECTED_MAGIC);
    }
}
