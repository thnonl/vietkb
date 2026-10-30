//! Test seam for the zero-focus E2E harness. Compiled only with the
//! `e2e-harness` cargo feature, so release builds are unaffected.
//!
//! Two modes use it:
//!
//! * **in-process (default)** - no global events at all. The harness builds
//!   `KBDLLHOOKSTRUCT` values and calls [`feed`], which runs the same
//!   `hook::process_event` path as the real hook procedure. The backend's
//!   injections never reach `SendInput`: [`set_capture`] diverts them into a
//!   buffer that [`take_captured`] drains.
//! * **live hooks (`--live-hooks`)** - a real `WH_KEYBOARD_LL` sandwich; here the
//!   tags below isolate the harness from other keyboard traffic.
//!
//! Tags:
//!
//! * **harness tag** - the backend's hook processes *only* events whose
//!   `dwExtraInfo` equals it; everything else is passed untouched.
//! * **inject tag** - stamped on the backend's own injections instead of the
//!   production marker.
//!
//! Until [`configure`] runs, nothing is admitted (`CONFIGURED` is false), even
//! though `usize::MAX` is a legal `dwExtraInfo`.

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

use parking_lot::Mutex;
use windows::Win32::UI::Input::KeyboardAndMouse::{INPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE};
use windows::Win32::UI::WindowsAndMessaging::KBDLLHOOKSTRUCT;

use crate::input::INJECTED_MAGIC;
use crate::state::HookRuntime;

static CONFIGURED: AtomicBool = AtomicBool::new(false);
static HARNESS_TAG: AtomicUsize = AtomicUsize::new(0);
static INJECT_TAG: AtomicUsize = AtomicUsize::new(INJECTED_MAGIC);

/// Harness-tagged events the backend's hook has looked at.
static SEEN: AtomicU64 = AtomicU64::new(0);
/// ... of which the backend swallowed (returned 1 for).
static SWALLOWED: AtomicU64 = AtomicU64::new(0);

/// Negative control: behave like the pre-D-E2E-03 build (a release-mode hotkey
/// trigger key-down reaches the application).
static LEGACY_TRIGGER_LEAK: AtomicBool = AtomicBool::new(false);

/// Set the two tags. Call before the hook is installed / events are fed.
pub fn configure(harness_tag: usize, inject_tag: usize) {
    assert_ne!(harness_tag, inject_tag, "the two tags must differ");
    assert_ne!(harness_tag, INJECTED_MAGIC, "never reuse the production marker");
    assert_ne!(inject_tag, INJECTED_MAGIC, "never reuse the production marker");
    HARNESS_TAG.store(harness_tag, Ordering::SeqCst);
    INJECT_TAG.store(inject_tag, Ordering::SeqCst);
    CONFIGURED.store(true, Ordering::SeqCst);
}

/// Back to the inert state.
pub fn reset() {
    CONFIGURED.store(false, Ordering::SeqCst);
    HARNESS_TAG.store(0, Ordering::SeqCst);
    INJECT_TAG.store(INJECTED_MAGIC, Ordering::SeqCst);
    CAPTURE_ON.store(false, Ordering::SeqCst);
    SHORT_BATCHES.lock().clear();
    LEGACY_TRIGGER_LEAK.store(false, Ordering::SeqCst);
}

pub(crate) fn inject_tag() -> usize {
    INJECT_TAG.load(Ordering::Relaxed)
}

/// `true` when the hook may process an event with this `dwExtraInfo`.
pub(crate) fn admit(extra: usize) -> bool {
    CONFIGURED.load(Ordering::Relaxed) && extra == HARNESS_TAG.load(Ordering::Relaxed)
}

pub(crate) fn note_decision(swallowed: bool) {
    SEEN.fetch_add(1, Ordering::SeqCst);
    if swallowed {
        SWALLOWED.fetch_add(1, Ordering::SeqCst);
    }
}

/// `(seen, swallowed)` for harness-tagged events at the backend's hook.
pub fn counters() -> (u64, u64) {
    (SEEN.load(Ordering::SeqCst), SWALLOWED.load(Ordering::SeqCst))
}

pub fn set_legacy_trigger_leak(on: bool) {
    LEGACY_TRIGGER_LEAK.store(on, Ordering::SeqCst);
}

pub(crate) fn legacy_trigger_leak() -> bool {
    LEGACY_TRIGGER_LEAK.load(Ordering::Relaxed)
}

// ---------------------------------------------------------------------------
// In-process transport
// ---------------------------------------------------------------------------

/// One injected key event, as the backend built it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Captured {
    /// `VK_PACKET` for a `KEYEVENTF_UNICODE` event, the virtual key otherwise.
    pub vk: u16,
    pub scan: u16,
    pub up: bool,
    pub extra: usize,
}

static CAPTURE_ON: AtomicBool = AtomicBool::new(false);
/// Event counts the next batches accept, front first (a short `SendInput`);
/// once empty every batch is taken whole.
static SHORT_BATCHES: Mutex<Vec<usize>> = Mutex::new(Vec::new());
static CAPTURED: Mutex<Vec<Captured>> = Mutex::new(Vec::new());

/// Divert the backend's injections into a buffer instead of `SendInput`.
pub fn set_capture(on: bool) {
    CAPTURE_ON.store(on, Ordering::SeqCst);
    CAPTURED.lock().clear();
}

/// Make the next captured batches short: batch `i` accepts only `events[i]` of
/// its events, as a `SendInput` that a blocked target cuts off.
pub fn short_next_batches(events: &[usize]) {
    let mut queue = SHORT_BATCHES.lock();
    queue.clear();
    queue.extend(events.iter().rev());
}

/// Called by the injector in place of `SendInput`; `Some(n)` = the first `n`
/// events were captured (`n == inputs.len()` unless a short batch was armed).
pub(crate) fn capture(inputs: &[INPUT]) -> Option<usize> {
    if !CAPTURE_ON.load(Ordering::Relaxed) {
        return None;
    }
    let accepted = SHORT_BATCHES
        .lock()
        .pop()
        .map_or(inputs.len(), |limit| limit.min(inputs.len()));
    let mut out = CAPTURED.lock();
    for input in &inputs[..accepted] {
        let ki = unsafe { input.Anonymous.ki };
        let unicode = ki.dwFlags.0 & KEYEVENTF_UNICODE.0 != 0;
        out.push(Captured {
            vk: if unicode {
                crate::keys::VK_PACKET
            } else {
                ki.wVk.0
            },
            scan: ki.wScan,
            up: ki.dwFlags.0 & KEYEVENTF_KEYUP.0 != 0,
            extra: ki.dwExtraInfo,
        });
    }
    Some(accepted)
}

pub fn take_captured() -> Vec<Captured> {
    std::mem::take(&mut *CAPTURED.lock())
}

/// Run one harness event through the real hook decision path
/// (`hook::process_event`). Returns `true` when the hook would swallow it;
/// events that are not harness-tagged are passed, exactly as in live mode.
pub fn feed(runtime: &mut HookRuntime, message: u32, info: &KBDLLHOOKSTRUCT) -> bool {
    if !admit(info.dwExtraInfo) {
        return false;
    }
    let swallow = crate::hook::process_event(runtime, message, info);
    note_decision(swallow);
    swallow
}

/// Simulate the mouse hook seeing a button-down (what `mouse_proc` does).
pub fn mouse_button_down() {
    crate::globals::note_mouse_down();
}
