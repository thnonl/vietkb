//! D-WIN-09 measurement seam: `VKB_INJECT_STRATEGY` (dev-channel builds only).
//!
//! Win11 Notepad types the VNI macro `qa1` -> `QA ONE` right only 25-50% of the
//! time although the emitted `INPUT` sequence is identical for hits and misses
//! (a WinForms target is 20/20). We cannot run Windows here, so this module lets
//! the Windows Lead try several ways of handing a `Replace { deletes, text }` to
//! `SendInput` without a rebuild per idea. Protocol:
//! `docs/evkey-clone-work/qa/win-dwin09-protocol.md`.
//!
//! # Gating
//!
//! * The variable is read **only** when this crate is built with the
//!   `dev-channel` feature (vkb-app forwards its own `dev-channel`). Without the
//!   feature [`requested`] is a constant `None`: a release build never touches the
//!   environment and the legacy path in `state.rs` runs byte-identically.
//! * With the feature but the variable unset (or unparsable) the result is also
//!   `None`: the legacy `input::send_split` path runs unchanged.
//! * Only the hook path (`state.rs::deliver_with`) consults it, only for a
//!   `Replace` (`deletes > 0`) in the default Unicode mode. `send_standard`,
//!   plain typing, the clipboard fallback and the alternate `Injector` sink are
//!   never affected.
//!
//! # Strategies (see [`InjectStrategy`])
//!
//! `split` is the current behaviour (this path only adds timings to the trace);
//! `single` is the pre-split control. Every other strategy keeps the `split`
//! grouping (Backspace block first, text after) and changes one thing:
//!
//! | name | change |
//! |---|---|
//! | `split-delay-N` | text call goes out >= N ms after the Backspace call |
//! | `per-char` | each character (Shift-wrapped press, or packet pair) is its own call |
//! | `scancode-shift` | Shift = `VK_LSHIFT` + scan `0x2A` + `KEYEVENTF_SCANCODE` |
//! | `unicode-after-split` | text as `KEYEVENTF_UNICODE` packets (capitals too) |
//! | `capslock-free` | Shift = plain `VK_LSHIFT` + scan `0x2A` (no `VK_SHIFT`) |
//!
//! # Why `split-delay-N` uses a thread
//!
//! The low-level hook callback must return fast, and events a callback injects are
//! only dequeued after it returns, so sleeping inside it would delay nothing the
//! target can see. The Backspace call is therefore sent on the hook thread as
//! before and the delayed text call(s) are handed to one dedicated injector
//! thread (FIFO, so ordering is preserved). While such a job is pending
//! ([`PENDING`] > 0) the hook waits at the top of the *next physical* key event
//! (at most [`WAIT_CAP`]) before processing it; the OS keeps later keys queued
//! meanwhile. Injected events (our own tag) are never delayed. If the wait ever
//! hits the cap, `keys.log` says `inject-wait TIMEOUT`.
//!
//! A short send in this path calls the normal step-down (`on_short_send`) but is
//! not retried or Shift-repaired; the delayed part is reported as fully sent
//! (its real result is in the `inject-async` trace line).

// Release builds compile the planner (it is unit-tested) but never call it.
#![cfg_attr(not(feature = "dev-channel"), allow(dead_code))]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, OnceLock};
use std::time::{Duration, Instant};

use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, KEYEVENTF_UNICODE, VIRTUAL_KEY,
};

use crate::debuglog;
use crate::input::{self, Held, SendMode, SendResult};
use crate::keys;
use vkb_core::Modifiers;
use vkb_platform::inject_strategy::InjectStrategy;

/// How long a physical key waits for a pending delayed injection.
const WAIT_CAP: Duration = Duration::from_millis(150);

/// Delayed jobs queued or running on the injector thread.
static PENDING: AtomicUsize = AtomicUsize::new(0);

/// Scan code of the left Shift key.
const SCAN_LSHIFT: u16 = 0x2A;

/// The strategy asked for through `VKB_INJECT_STRATEGY`. Always `None` unless the
/// crate is built with `dev-channel`.
#[cfg(feature = "dev-channel")]
pub fn requested() -> Option<InjectStrategy> {
    static CHOSEN: OnceLock<Option<InjectStrategy>> = OnceLock::new();
    *CHOSEN.get_or_init(|| {
        let raw = std::env::var("VKB_INJECT_STRATEGY").ok()?;
        if raw.trim().is_empty() {
            return None;
        }
        let parsed = InjectStrategy::parse(&raw);
        match parsed {
            Some(s) => debuglog::note(&format!("inject-strategy {} (VKB_INJECT_STRATEGY)", s.label())),
            None => debuglog::note(&format!(
                "inject-strategy IGNORED: unknown value {raw:?}; using the default path"
            )),
        }
        parsed
    })
}

/// Release builds: the environment is never read.
#[cfg(not(feature = "dev-channel"))]
#[inline(always)]
pub fn requested() -> Option<InjectStrategy> {
    None
}

/// The strategy to use for this injection, or `None` for the legacy path.
#[inline]
pub fn active(mode: SendMode, deletes: u32) -> Option<InjectStrategy> {
    if deletes == 0 || mode != SendMode::Unicode {
        return None;
    }
    requested()
}

/// One `SendInput` call of a plan.
#[derive(Clone)]
pub struct Call {
    /// Minimum pause before this call, ms. Only ever set on the second call of
    /// `split-delay-N`.
    pub delay_ms: u32,
    pub inputs: Vec<INPUT>,
}

/// The three pieces of a replacement, in order: chord release + Backspaces,
/// text, and the Shift re-press after a Shift restore chord. Concatenated they
/// are exactly what `input::build_segment_held` / `input::build_restore` build.
struct Parts {
    head: Vec<INPUT>,
    text: Vec<INPUT>,
    tail: Vec<INPUT>,
}

fn parts(deletes: u32, text: &str, chord: Modifiers, held: Held, unicode_text: bool) -> Parts {
    let mut head = Vec::new();
    input::push_chord_release(&mut head, chord);
    for _ in 0..deletes {
        input::push_backspace(&mut head, SendMode::Unicode);
    }
    // The chord's own Shift is up for the rest of the batch (see `build_restore`).
    let during = if chord == Modifiers::NONE {
        held
    } else {
        Held {
            shift: if chord.shift { 0 } else { held.shift },
            caps: held.caps,
        }
    };
    let mut body = Vec::new();
    // `scancode_ascii = deletes > 0` is the D-E2E-01 rule; the unicode variant
    // turns it off so every character is a packet.
    input::push_text(&mut body, text, SendMode::Unicode, !unicode_text && deletes > 0, during);
    let mut tail = Vec::new();
    if chord.shift {
        input::push_shift_down(&mut tail, held.shift);
    }
    Parts { head, text: body, tail }
}

fn is_shift_vk(vk: u16) -> bool {
    matches!(vk, keys::VK_SHIFT | keys::VK_LSHIFT | keys::VK_RSHIFT)
}

/// Rewrite the Shift events *we* injected (never a packet, never the user's).
fn rewrite_shift(events: &mut [INPUT], scancode_flag: bool) {
    for ev in events {
        let mut ki = unsafe { ev.Anonymous.ki };
        if ki.dwFlags.0 & KEYEVENTF_UNICODE.0 != 0
            || ki.dwExtraInfo != input::injected_tag()
            || !matches!(ki.wVk.0, keys::VK_SHIFT | keys::VK_LSHIFT)
        {
            continue;
        }
        ki.wVk = VIRTUAL_KEY(keys::VK_LSHIFT);
        ki.wScan = SCAN_LSHIFT;
        if scancode_flag {
            ki.dwFlags = KEYBD_EVENT_FLAGS(ki.dwFlags.0 | KEYEVENTF_SCANCODE.0);
        }
        ev.Anonymous.ki = ki;
    }
}

/// One group per typed character: a Shift-wrapped press stays together, a
/// surrogate pair is never cut.
fn group_per_char(events: Vec<INPUT>) -> Vec<Vec<INPUT>> {
    let mut out = Vec::new();
    let mut cur = Vec::new();
    let mut shift_open = 0i32;
    for ev in events {
        let ki = unsafe { ev.Anonymous.ki };
        let unicode = ki.dwFlags.0 & KEYEVENTF_UNICODE.0 != 0;
        let up = ki.dwFlags.0 & KEYEVENTF_KEYUP.0 != 0;
        cur.push(ev);
        if !unicode && is_shift_vk(ki.wVk.0) {
            shift_open += if up { -1 } else { 1 };
            if shift_open <= 0 {
                shift_open = 0;
                out.push(std::mem::take(&mut cur));
            }
        } else if up && shift_open == 0 && !(unicode && (0xD800..0xDC00).contains(&ki.wScan)) {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn call(inputs: Vec<INPUT>) -> Call {
    Call { delay_ms: 0, inputs }
}

/// Pure: the calls a strategy sends for one `Replace`. `chord` is
/// `Modifiers::NONE` unless a `restore_word` chord fired this injection.
pub fn plan(strategy: InjectStrategy, deletes: u32, text: &str, chord: Modifiers, held: Held) -> Vec<Call> {
    let unicode_text = strategy == InjectStrategy::UnicodeAfterSplit;
    let Parts { head, mut text, tail } = parts(deletes, text, chord, held, unicode_text);
    match strategy {
        InjectStrategy::ScancodeShift => rewrite_shift(&mut text, true),
        InjectStrategy::CapslockFree => rewrite_shift(&mut text, false),
        _ => {}
    }
    if strategy == InjectStrategy::Single {
        let mut all = head;
        all.extend(text);
        all.extend(tail);
        return vec![call(all)];
    }
    let mut rest = text;
    rest.extend(tail);
    if rest.is_empty() {
        return vec![call(head)];
    }
    let mut calls = vec![call(head)];
    match strategy {
        InjectStrategy::PerChar => calls.extend(group_per_char(rest).into_iter().map(call)),
        InjectStrategy::SplitDelay(n) => calls.push(Call {
            delay_ms: u32::from(n),
            inputs: rest,
        }),
        _ => calls.push(call(rest)),
    }
    calls
}

/// Build and send one `Replace` with `strategy`. `sent` counts events across
/// calls, as `input::send_split` does.
pub fn deliver(strategy: InjectStrategy, deletes: u32, text: &str, chord: Modifiers, held: Held) -> SendResult {
    run(strategy, deletes, text, plan(strategy, deletes, text, chord, held))
}

struct Job {
    label: String,
    started: Instant,
    calls: Vec<Call>,
}

fn run(strategy: InjectStrategy, deletes: u32, text: &str, calls: Vec<Call>) -> SendResult {
    let total: u32 = calls.iter().map(|c| c.inputs.len() as u32).sum();
    let started = Instant::now();
    let tracing_on = debuglog::enabled();
    let mut trace = String::new();
    let mut result = SendResult {
        total,
        ..SendResult::default()
    };
    let mut iter = calls.into_iter();
    let mut deferred = None;
    while let Some(c) = iter.next() {
        if c.delay_ms > 0 {
            let mut rest = vec![c];
            rest.extend(&mut iter);
            deferred = Some(rest);
            break;
        }
        let begin = started.elapsed();
        let r = input::send_detailed(&c.inputs);
        let end = started.elapsed();
        if tracing_on {
            use std::fmt::Write as _;
            let _ = write!(
                trace,
                " [{}ev @{}us +{}us sent={}]",
                c.inputs.len(),
                begin.as_micros(),
                (end - begin).as_micros(),
                r.sent
            );
        }
        result.sent += r.sent;
        result.attempts += r.attempts;
        result.error = r.error;
        if r.sent < r.total {
            break;
        }
    }
    if let Some(rest) = deferred {
        let events: u32 = rest.iter().map(|c| c.inputs.len() as u32).sum();
        let delays: Vec<u32> = rest.iter().map(|c| c.delay_ms).collect();
        enqueue(Job {
            label: strategy.label(),
            started,
            calls: rest,
        });
        result.sent += events;
        if tracing_on {
            use std::fmt::Write as _;
            let _ = write!(trace, " deferred={events}ev delays_ms={delays:?}");
        }
    }
    if tracing_on {
        debuglog::note(&format!(
            "inject strategy={} deletes={} text={:?} total={}ev sent={}/{} lasterror={} total_us={}{}",
            strategy.label(),
            deletes,
            text,
            total,
            result.sent,
            result.total,
            result.error,
            started.elapsed().as_micros(),
            trace
        ));
    }
    result
}

fn injector() -> &'static mpsc::Sender<Job> {
    static TX: OnceLock<mpsc::Sender<Job>> = OnceLock::new();
    TX.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<Job>();
        let spawned = std::thread::Builder::new()
            .name("vkb-inject-dev".to_string())
            .spawn(move || {
                for job in rx {
                    run_job(job);
                    PENDING.fetch_sub(1, Ordering::SeqCst);
                }
            });
        if let Err(err) = spawned {
            tracing::error!(error = %err, "cannot spawn the dev injector thread");
        }
        tx
    })
}

fn enqueue(job: Job) {
    PENDING.fetch_add(1, Ordering::SeqCst);
    if let Err(mpsc::SendError(job)) = injector().send(job) {
        // The injector thread is gone: do it inline rather than lose the text.
        run_job(job);
        PENDING.fetch_sub(1, Ordering::SeqCst);
    }
}

fn run_job(job: Job) {
    for c in job.calls {
        if c.delay_ms > 0 {
            std::thread::sleep(Duration::from_millis(u64::from(c.delay_ms)));
        }
        let begin = job.started.elapsed();
        let r = input::send_detailed(&c.inputs);
        if debuglog::enabled() {
            debuglog::note(&format!(
                "inject-async strategy={} delay_ms={} {}ev @{}us (since first call) +{}us sent={}/{} lasterror={}",
                job.label,
                c.delay_ms,
                c.inputs.len(),
                begin.as_micros(),
                (job.started.elapsed() - begin).as_micros(),
                r.sent,
                r.total,
                r.error
            ));
        }
    }
}

/// Top of every hook event: a physical key waits (bounded) while a delayed
/// injection is pending, so it cannot overtake the replacement text.
#[inline]
pub fn wait_pending(extra_info: usize) {
    if PENDING.load(Ordering::SeqCst) == 0 || extra_info == input::injected_tag() {
        return;
    }
    let begin = Instant::now();
    while PENDING.load(Ordering::SeqCst) > 0 {
        if begin.elapsed() >= WAIT_CAP {
            debuglog::note(&format!(
                "inject-wait TIMEOUT after {}us; processing the key anyway",
                begin.elapsed().as_micros()
            ));
            return;
        }
        std::thread::sleep(Duration::from_micros(500));
    }
    if debuglog::enabled() {
        debuglog::note(&format!("inject-wait {}us", begin.elapsed().as_micros()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// (vk, scan, flags) per event.
    fn sig(inputs: &[INPUT]) -> Vec<(u16, u16, u32, usize)> {
        inputs
            .iter()
            .map(|i| {
                let ki = unsafe { i.Anonymous.ki };
                (ki.wVk.0, ki.wScan, ki.dwFlags.0, ki.dwExtraInfo)
            })
            .collect()
    }

    fn flat(calls: &[Call]) -> Vec<INPUT> {
        calls.iter().flat_map(|c| c.inputs.iter().copied()).collect()
    }

    fn lens(calls: &[Call]) -> Vec<usize> {
        calls.iter().map(|c| c.inputs.len()).collect()
    }

    fn delays(calls: &[Call]) -> Vec<u32> {
        calls.iter().map(|c| c.delay_ms).collect()
    }

    const NO_HELD: Held = Held { shift: 0, caps: false };

    /// `qa1` -> `QA ONE` after two Backspaces: the case under investigation.
    const QA_ONE: &str = "QA ONE";

    fn is_packet(i: &INPUT) -> bool {
        unsafe { i.Anonymous.ki }.dwFlags.0 & KEYEVENTF_UNICODE.0 != 0
    }

    #[test]
    fn single_equals_the_default_one_call_batch() {
        let want = input::build_segment_held(3, QA_ONE, SendMode::Unicode, NO_HELD);
        let calls = plan(InjectStrategy::Single, 3, QA_ONE, Modifiers::NONE, NO_HELD);
        assert_eq!(calls.len(), 1);
        assert_eq!(sig(&calls[0].inputs), sig(&want));
    }

    #[test]
    fn split_matches_send_split_grouping_and_events() {
        let want = input::build_segment_held(3, QA_ONE, SendMode::Unicode, NO_HELD);
        let at = input::split_point(&want, 3, SendMode::Unicode);
        let calls = plan(InjectStrategy::Split, 3, QA_ONE, Modifiers::NONE, NO_HELD);
        assert_eq!(lens(&calls), vec![at, want.len() - at]);
        assert_eq!(sig(&flat(&calls)), sig(&want));
        assert_eq!(delays(&calls), vec![0, 0]);
        // First call is only Backspaces.
        assert!(sig(&calls[0].inputs).iter().all(|e| e.0 == keys::VK_BACK));
        assert_eq!(calls[0].inputs.len(), 6);
    }

    #[test]
    fn split_with_a_restore_chord_matches_build_restore() {
        let chord = Modifiers {
            shift: true,
            ctrl: true,
            ..Modifiers::NONE
        };
        let held = Held { shift: keys::SHIFT_R, caps: false };
        let want = input::build_restore(2, "Tâ", SendMode::Unicode, chord, held);
        let at = input::split_point(&want, 2, SendMode::Unicode);
        let calls = plan(InjectStrategy::Split, 2, "Tâ", chord, held);
        assert_eq!(sig(&flat(&calls)), sig(&want));
        assert_eq!(calls[0].inputs.len(), at, "chord release stays with the deletes");
        // The final Shift re-press rides with the text call.
        let last = *calls.last().unwrap().inputs.last().unwrap();
        let ki = unsafe { last.Anonymous.ki };
        assert_eq!((ki.wVk.0, ki.dwFlags.0 & KEYEVENTF_KEYUP.0), (keys::VK_RSHIFT, 0));
    }

    #[test]
    fn every_strategy_keeps_the_delete_block_first_and_alone() {
        for s in [
            InjectStrategy::Split,
            InjectStrategy::SplitDelay(10),
            InjectStrategy::PerChar,
            InjectStrategy::ScancodeShift,
            InjectStrategy::UnicodeAfterSplit,
            InjectStrategy::CapslockFree,
        ] {
            let calls = plan(s, 2, QA_ONE, Modifiers::NONE, NO_HELD);
            assert_eq!(lens(&calls)[0], 4, "{s:?}");
            assert!(calls.len() >= 2, "{s:?}");
            assert!(sig(&calls[0].inputs).iter().all(|e| e.0 == keys::VK_BACK), "{s:?}");
        }
    }

    #[test]
    fn split_delay_puts_the_delay_on_the_text_call_only() {
        let calls = plan(InjectStrategy::SplitDelay(25), 3, QA_ONE, Modifiers::NONE, NO_HELD);
        assert_eq!(calls.len(), 2);
        assert_eq!(delays(&calls), vec![0, 25]);
        let split = plan(InjectStrategy::Split, 3, QA_ONE, Modifiers::NONE, NO_HELD);
        assert_eq!(sig(&flat(&calls)), sig(&flat(&split)), "same events as split");
    }

    #[test]
    fn only_split_delay_has_delays() {
        for s in [
            InjectStrategy::Split,
            InjectStrategy::Single,
            InjectStrategy::PerChar,
            InjectStrategy::ScancodeShift,
            InjectStrategy::UnicodeAfterSplit,
            InjectStrategy::CapslockFree,
        ] {
            let calls = plan(s, 3, QA_ONE, Modifiers::NONE, NO_HELD);
            assert!(calls.iter().all(|c| c.delay_ms == 0), "{s:?}");
        }
    }

    #[test]
    fn per_char_is_one_call_per_character() {
        // "QA ONE": Q, A, Space, O, N, E -> 6 text calls after the delete call.
        let calls = plan(InjectStrategy::PerChar, 3, QA_ONE, Modifiers::NONE, NO_HELD);
        assert_eq!(lens(&calls), vec![6, 4, 4, 2, 4, 4, 4]);
        // Shift-wrapped capitals: Shift down first, Shift up last.
        for c in [&calls[1], &calls[2], &calls[4], &calls[5], &calls[6]] {
            let s = sig(&c.inputs);
            assert_eq!(s[0].0, keys::VK_SHIFT);
            assert_eq!(s[0].2 & KEYEVENTF_KEYUP.0, 0);
            assert_eq!(s[3].0, keys::VK_SHIFT);
            assert_ne!(s[3].2 & KEYEVENTF_KEYUP.0, 0);
        }
        let split = plan(InjectStrategy::Split, 3, QA_ONE, Modifiers::NONE, NO_HELD);
        assert_eq!(sig(&flat(&calls)), sig(&flat(&split)), "regrouping only");
    }

    #[test]
    fn per_char_keeps_packets_and_surrogates_whole() {
        let calls = plan(InjectStrategy::PerChar, 1, "tă😀", Modifiers::NONE, NO_HELD);
        // delete, t (2), ă packet (2), emoji = 2 packets (4) in ONE call.
        assert_eq!(lens(&calls), vec![2, 2, 2, 4]);
    }

    #[test]
    fn scancode_shift_rewrites_only_the_text_shift() {
        let calls = plan(InjectStrategy::ScancodeShift, 2, "Qa", Modifiers::NONE, NO_HELD);
        let text = sig(&calls[1].inputs);
        // Shift down, Q down, Q up, Shift up, a down, a up.
        assert_eq!(text.len(), 6);
        assert_eq!((text[0].0, text[0].1), (keys::VK_LSHIFT, 0x2A));
        assert_eq!(text[0].2, KEYEVENTF_SCANCODE.0);
        assert_eq!(text[3].2, KEYEVENTF_SCANCODE.0 | KEYEVENTF_KEYUP.0);
        assert_eq!(text[0].3, input::injected_tag());
        // Letters keep VK + scan and no SCANCODE flag.
        assert_eq!(text[1].0, 0x51);
        assert_eq!(text[1].1, keys::scan_for_vk(0x51));
        assert_eq!(text[1].2 & KEYEVENTF_SCANCODE.0, 0);
        assert!(text.iter().all(|e| e.0 != keys::VK_SHIFT));
        // Backspaces untouched.
        assert!(sig(&calls[0].inputs).iter().all(|e| e.0 == keys::VK_BACK && e.2 & KEYEVENTF_SCANCODE.0 == 0));
    }

    #[test]
    fn capslock_free_uses_lshift_without_the_scancode_flag() {
        let calls = plan(InjectStrategy::CapslockFree, 2, "Qa", Modifiers::NONE, NO_HELD);
        let text = sig(&calls[1].inputs);
        assert_eq!((text[0].0, text[0].1, text[0].2), (keys::VK_LSHIFT, 0x2A, 0));
        assert_eq!(
            (text[3].0, text[3].1, text[3].2),
            (keys::VK_LSHIFT, 0x2A, KEYEVENTF_KEYUP.0)
        );
        assert!(text.iter().all(|e| e.0 != keys::VK_SHIFT));
        assert_eq!(text[1].0, 0x51);
    }

    #[test]
    fn rewrite_leaves_foreign_shift_and_packets_alone() {
        let mut ev = Vec::new();
        input::push_key(&mut ev, keys::VK_SHIFT, 0x2A, false);
        // A Shift not tagged as ours (the user's) must never be rewritten.
        let mut foreign = ev.clone();
        let mut ki = unsafe { foreign[0].Anonymous.ki };
        ki.dwExtraInfo = 0;
        foreign[0].Anonymous.ki = ki;
        let before = sig(&foreign);
        rewrite_shift(&mut foreign, true);
        assert_eq!(sig(&foreign), before);
    }

    #[test]
    fn unicode_after_split_sends_every_character_as_a_packet() {
        let calls = plan(InjectStrategy::UnicodeAfterSplit, 3, QA_ONE, Modifiers::NONE, NO_HELD);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].inputs.len(), 6);
        assert_eq!(calls[1].inputs.len(), 6 * 2, "6 characters, down+up each");
        assert!(calls[1].inputs.iter().all(is_packet));
        let units: Vec<u16> = sig(&calls[1].inputs).iter().step_by(2).map(|e| e.1).collect();
        assert_eq!(units, "QA ONE".encode_utf16().collect::<Vec<_>>());
    }

    #[test]
    fn unicode_after_split_still_types_newlines_as_return() {
        let calls = plan(InjectStrategy::UnicodeAfterSplit, 1, "a\nb", Modifiers::NONE, NO_HELD);
        let text = sig(&calls[1].inputs);
        assert_eq!(text.len(), 6);
        assert_eq!(text[2].0, keys::VK_RETURN);
    }

    #[test]
    fn an_empty_text_is_a_single_delete_call() {
        for s in [
            InjectStrategy::Split,
            InjectStrategy::SplitDelay(5),
            InjectStrategy::PerChar,
            InjectStrategy::UnicodeAfterSplit,
        ] {
            let calls = plan(s, 2, "", Modifiers::NONE, NO_HELD);
            assert_eq!(lens(&calls), vec![4], "{s:?}");
        }
    }

    #[test]
    fn a_held_shift_is_never_wrapped_or_rewritten() {
        let held = Held { shift: keys::SHIFT_L, caps: false };
        let calls = plan(InjectStrategy::ScancodeShift, 1, "AB", Modifiers::NONE, held);
        let want = input::build_segment_held(1, "AB", SendMode::Unicode, held);
        assert_eq!(sig(&flat(&calls)), sig(&want), "no Shift events to rewrite");
    }

    #[test]
    fn active_requires_a_unicode_replace() {
        // Without the env var nothing is requested, whatever the shape.
        assert_eq!(active(SendMode::Unicode, 0), None);
        assert_eq!(active(SendMode::Standard, 3), None);
        #[cfg(not(feature = "dev-channel"))]
        assert_eq!(active(SendMode::Unicode, 3), None);
    }
}
