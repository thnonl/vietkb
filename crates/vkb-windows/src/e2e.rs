//! Test seam for the zero-focus hook harness. Compiled only with the
//! `e2e-harness` cargo feature, so release builds are unaffected.
//!
//! Two tags make a test process invisible to everything else on the desktop:
//!
//! * **harness tag** — the backend's hook processes *only* events whose
//!   `dwExtraInfo` equals it. Physical keys and the injections of another
//!   VietKB (the owner's installed copy) carry other values and are passed on
//!   untouched, without even updating the backend's modifier tracking.
//! * **inject tag** — stamped on the backend's own injections (replacing the
//!   production marker), so the harness can recognise exactly what the backend
//!   produced and swallow it before it reaches any window.
//!
//! Until [`configure`] runs, the harness tag is `usize::MAX`-ish (`NO_TAG`) and
//! nothing is processed at all.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use crate::input::INJECTED_MAGIC;

/// Sentinel: no event ever carries it, so an unconfigured backend is inert.
pub const NO_TAG: usize = usize::MAX;

static HARNESS_TAG: AtomicUsize = AtomicUsize::new(NO_TAG);
static INJECT_TAG: AtomicUsize = AtomicUsize::new(INJECTED_MAGIC);

/// Harness-tagged events the backend's hook has looked at.
static SEEN: AtomicU64 = AtomicU64::new(0);
/// ... of which the backend swallowed (returned 1 for).
static SWALLOWED: AtomicU64 = AtomicU64::new(0);

/// Set the two tags. Call before the hook is installed.
pub fn configure(harness_tag: usize, inject_tag: usize) {
    assert_ne!(harness_tag, inject_tag, "the two tags must differ");
    assert_ne!(harness_tag, INJECTED_MAGIC, "never reuse the production marker");
    assert_ne!(inject_tag, INJECTED_MAGIC, "never reuse the production marker");
    HARNESS_TAG.store(harness_tag, Ordering::SeqCst);
    INJECT_TAG.store(inject_tag, Ordering::SeqCst);
}

/// Back to the inert state.
pub fn reset() {
    HARNESS_TAG.store(NO_TAG, Ordering::SeqCst);
    INJECT_TAG.store(INJECTED_MAGIC, Ordering::SeqCst);
}

pub(crate) fn inject_tag() -> usize {
    INJECT_TAG.load(Ordering::Relaxed)
}

/// `true` when the hook may process an event with this `dwExtraInfo`.
pub(crate) fn admit(extra: usize) -> bool {
    extra == HARNESS_TAG.load(Ordering::Relaxed)
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
