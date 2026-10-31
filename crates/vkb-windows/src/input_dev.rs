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
//! The Notepad spell-check / autocorrect strategies (D-WIN-09 / D-WIN-11: the
//! Win11 Notepad text services rewrite the word right after a Backspace + replace
//! burst) change *what* is sent, not only how it is grouped:
//!
//! | name | change |
//! |---|---|
//! | `select-replace` | Shift+Left x N (`VK_LSHIFT`, `VK_LEFT`, Shift released) selects the N chars, then the text (VK path) in a second call |
//! | `select-replace-unicode` | same selection, the text as `KEYEVENTF_UNICODE` packets |
//! | `word-retype` | delete the whole word (engine head + the changed tail) and retype the whole new word, ONE call |
//! | `word-retype-split` | same, the Backspace block is its own call |
//! | `unicode-after-split-delay-N` | Backspace call, >= N ms, text as packets (injector thread) |
//! | `boundary-hold-N` | the boundary key (space/punctuation) after our text is forwarded >= N ms later |
//!
//! Select strategies never select across a line start: the selection is used
//! only when the engine knows the N chars are the tail of the word it tracks
//! (a word never spans a line start: Enter and caret keys end it), the word has
//! no combining mark or surrogate (Shift+Left moves by grapheme, Backspace by
//! code point), the user holds no Shift and the text is not empty. Otherwise the
//! injection is the plain `split` (trace: `fallback=split`). `word-retype` needs
//! the head the engine trimmed ([`Context`]); with none it equals `single` /
//! `split`.
//!
//! `boundary-hold-N` has three parts. (1) A boundary character that ends our
//! text (macro `qa1` -> `QA ONE `) is split off and sent N ms after the rest,
//! from the injector thread. (2) A physical boundary key that arrives within N ms
//! of our last injection waits until N ms have passed (top of the hook
//! callback). (3) A boundary key that the same event lets through after our
//! text (`ReplaceThenPass`: Enter/Tab) is held by sleeping N ms before the
//! callback returns.
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

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{mpsc, OnceLock};
use std::time::{Duration, Instant};

use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, KEYEVENTF_UNICODE, VIRTUAL_KEY,
};
#[cfg(test)]
use windows::Win32::UI::Input::KeyboardAndMouse::KEYEVENTF_EXTENDEDKEY;

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
/// Scan code of the (extended) Left arrow.
const SCAN_LEFT: u16 = 0x4B;

/// Microseconds since [`epoch`] (+1) of the end of the last `boundary-hold`
/// injection (`0` = none) and a counter bumped by every such stamp.
static LAST_INJECT_US: AtomicU64 = AtomicU64::new(0);
static INJECT_SEQ: AtomicU64 = AtomicU64::new(0);

fn epoch() -> Instant {
    static T0: OnceLock<Instant> = OnceLock::new();
    *T0.get_or_init(Instant::now)
}

fn stamp_inject() {
    LAST_INJECT_US.store(epoch().elapsed().as_micros() as u64 + 1, Ordering::SeqCst);
    INJECT_SEQ.fetch_add(1, Ordering::SeqCst);
}

/// What the engine knew about the word when it produced this injection. Built in
/// `state.rs::process` only when a strategy is requested; an injection it does
/// not describe ([`Context::applies_to`] false) is planned with
/// [`Context::default`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Context {
    /// The (trimmed) injection this describes.
    pub deletes: u32,
    pub insert: String,
    /// Characters the engine shows for the current word before the key; `None`
    /// = unknown. A word never spans a line start.
    pub word_len: Option<u32>,
    /// Every character of the word is a BMP, non-combining character.
    pub selectable: bool,
    /// The unchanged head `keep_unchanged_head` left out of the replacement.
    pub head: String,
}

impl Context {
    pub fn applies_to(&self, deletes: u32, text: &str) -> bool {
        self.word_len.is_some() && self.deletes == deletes && self.insert == text
    }
}

/// `false` for a combining mark (Shift+Left and Backspace disagree on the count)
/// or a character outside the BMP.
pub fn selectable_char(c: char) -> bool {
    (c as u32) <= 0xFFFF
        && !matches!(c,
            '\u{0300}'..='\u{036F}'
            | '\u{1AB0}'..='\u{1AFF}'
            | '\u{1DC0}'..='\u{1DFF}'
            | '\u{20D0}'..='\u{20FF}'
            | '\u{FE20}'..='\u{FE2F}')
}

/// A boundary character for the text services: space, tab or ASCII punctuation.
/// Newlines are left out (they are typed as Return).
pub fn is_boundary_char(c: char) -> bool {
    c == ' ' || c == '\t' || c.is_ascii_punctuation()
}

/// A physical boundary key: space, tab, Return or an OEM punctuation key.
pub fn is_boundary_vk(vk: u16) -> bool {
    matches!(vk, keys::VK_SPACE | keys::VK_RETURN | keys::VK_TAB | 0xBA..=0xC0 | 0xDB..=0xDF)
}

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
    /// The trailing boundary character, split off for `boundary-hold-N`.
    boundary: Vec<INPUT>,
    tail: Vec<INPUT>,
}

fn parts(
    deletes: u32,
    text: &str,
    chord: Modifiers,
    held: Held,
    unicode_text: bool,
    select: bool,
    hold_boundary: bool,
) -> Parts {
    let mut head = Vec::new();
    input::push_chord_release(&mut head, chord);
    if select {
        push_selection(&mut head, deletes);
    } else {
        for _ in 0..deletes {
            input::push_backspace(&mut head, SendMode::Unicode);
        }
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
    let mut boundary = Vec::new();
    // `scancode_ascii = deletes > 0` is the D-E2E-01 rule; the unicode variant
    // turns it off so every character is a packet.
    let scancode_ascii = !unicode_text && deletes > 0;
    let last = text.chars().next_back().filter(|c| hold_boundary && is_boundary_char(*c));
    if let Some(c) = last {
        let main = &text[..text.len() - c.len_utf8()];
        input::push_text(&mut body, main, SendMode::Unicode, scancode_ascii, during);
        input::push_text(&mut boundary, &c.to_string(), SendMode::Unicode, scancode_ascii, during);
    } else {
        input::push_text(&mut body, text, SendMode::Unicode, scancode_ascii, during);
    }
    let mut tail = Vec::new();
    if chord.shift {
        input::push_shift_down(&mut tail, held.shift);
    }
    Parts {
        head,
        text: body,
        boundary,
        tail,
    }
}

/// `Shift` down (`VK_LSHIFT`), `Left` x `n` (extended), `Shift` up.
fn push_selection(out: &mut Vec<INPUT>, n: u32) {
    input::push_key(out, keys::VK_LSHIFT, SCAN_LSHIFT, false);
    for _ in 0..n {
        input::push_key_ext(out, keys::VK_LEFT, SCAN_LEFT, false);
        input::push_key_ext(out, keys::VK_LEFT, SCAN_LEFT, true);
    }
    input::push_key(out, keys::VK_LSHIFT, SCAN_LSHIFT, true);
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

/// `true` when a select strategy may select instead of deleting.
fn can_select(deletes: u32, text: &str, held: Held, ctx: &Context) -> bool {
    deletes > 0
        && !text.is_empty()
        && !held.shift_down()
        && ctx.selectable
        && ctx.word_len.is_some_and(|len| deletes <= len)
}

/// The strategy actually used: a select strategy that cannot select safely
/// degrades to `Split`.
pub fn effective(strategy: InjectStrategy, deletes: u32, text: &str, held: Held, ctx: &Context) -> InjectStrategy {
    match strategy {
        InjectStrategy::SelectReplace | InjectStrategy::SelectReplaceUnicode
            if !can_select(deletes, text, held, ctx) =>
        {
            InjectStrategy::Split
        }
        s => s,
    }
}

/// [`plan_with`] without engine context: select strategies fall back to
/// `split`, `word-retype*` retype nothing extra.
pub fn plan(strategy: InjectStrategy, deletes: u32, text: &str, chord: Modifiers, held: Held) -> Vec<Call> {
    plan_with(strategy, deletes, text, chord, held, &Context::default())
}

/// Pure: the calls a strategy sends for one `Replace`. `chord` is
/// `Modifiers::NONE` unless a `restore_word` chord fired this injection.
pub fn plan_with(
    strategy: InjectStrategy,
    deletes: u32,
    text: &str,
    chord: Modifiers,
    held: Held,
    ctx: &Context,
) -> Vec<Call> {
    let strategy = effective(strategy, deletes, text, held, ctx);
    // word-retype: the unchanged head the engine trimmed goes back in.
    let retyped: String;
    let (deletes, text) = if matches!(strategy, InjectStrategy::WordRetype | InjectStrategy::WordRetypeSplit)
        && ctx.applies_to(deletes, text)
        && !ctx.head.is_empty()
    {
        retyped = format!("{}{}", ctx.head, text);
        (deletes + ctx.head.chars().count() as u32, retyped.as_str())
    } else {
        (deletes, text)
    };
    let select = matches!(
        strategy,
        InjectStrategy::SelectReplace | InjectStrategy::SelectReplaceUnicode
    );
    let unicode_text = matches!(
        strategy,
        InjectStrategy::UnicodeAfterSplit
            | InjectStrategy::UnicodeAfterSplitDelay(_)
            | InjectStrategy::SelectReplaceUnicode
    );
    let hold = matches!(strategy, InjectStrategy::BoundaryHold(_));
    let Parts {
        head,
        mut text,
        boundary,
        tail,
    } = parts(deletes, text, chord, held, unicode_text, select, hold);
    match strategy {
        InjectStrategy::ScancodeShift => rewrite_shift(&mut text, true),
        InjectStrategy::CapslockFree => rewrite_shift(&mut text, false),
        _ => {}
    }
    if matches!(strategy, InjectStrategy::Single | InjectStrategy::WordRetype) {
        let mut all = head;
        all.extend(text);
        all.extend(tail);
        return vec![call(all)];
    }
    if let InjectStrategy::BoundaryHold(n) = strategy {
        if !boundary.is_empty() {
            // head | text | (>= n ms) boundary + Shift re-press
            let mut last = boundary;
            last.extend(tail);
            let mut calls = vec![call(head)];
            if !text.is_empty() {
                calls.push(call(text));
            }
            calls.push(Call {
                delay_ms: u32::from(n),
                inputs: last,
            });
            return calls;
        }
    }
    let mut rest = text;
    rest.extend(tail);
    if rest.is_empty() {
        return vec![call(head)];
    }
    let mut calls = vec![call(head)];
    match strategy {
        InjectStrategy::PerChar => calls.extend(group_per_char(rest).into_iter().map(call)),
        InjectStrategy::SplitDelay(n) | InjectStrategy::UnicodeAfterSplitDelay(n) => calls.push(Call {
            delay_ms: u32::from(n),
            inputs: rest,
        }),
        _ => calls.push(call(rest)),
    }
    calls
}

/// Build and send one `Replace` with `strategy`. `sent` counts events across
/// calls, as `input::send_split` does.
pub fn deliver(
    strategy: InjectStrategy,
    deletes: u32,
    text: &str,
    chord: Modifiers,
    held: Held,
    ctx: &Context,
) -> SendResult {
    let used = effective(strategy, deletes, text, held, ctx);
    let mut label = strategy.label();
    if used != strategy {
        label.push_str(&format!(" fallback={}", used.label()));
    }
    run(
        strategy,
        label,
        deletes,
        text,
        plan_with(strategy, deletes, text, chord, held, ctx),
    )
}

struct Job {
    label: String,
    stamp: bool,
    started: Instant,
    calls: Vec<Call>,
}

fn run(strategy: InjectStrategy, label: String, deletes: u32, text: &str, calls: Vec<Call>) -> SendResult {
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
    if matches!(strategy, InjectStrategy::BoundaryHold(_)) {
        stamp_inject();
    }
    if let Some(rest) = deferred {
        let events: u32 = rest.iter().map(|c| c.inputs.len() as u32).sum();
        let delays: Vec<u32> = rest.iter().map(|c| c.delay_ms).collect();
        enqueue(Job {
            label: label.clone(),
            stamp: matches!(strategy, InjectStrategy::BoundaryHold(_)),
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
            label,
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
    if job.stamp {
        stamp_inject();
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

/// Time still to wait so that `n_ms` have passed since the stamp `last_us`
/// (both in microseconds on the [`epoch`] clock; `last_us == 0` = no stamp).
fn remaining(now_us: u64, last_us: u64, n_ms: u64) -> Duration {
    if last_us == 0 {
        return Duration::ZERO;
    }
    let due = last_us.saturating_add(n_ms * 1000);
    Duration::from_micros(due.saturating_sub(now_us))
}

fn boundary_hold_ms() -> Option<u64> {
    match requested() {
        Some(InjectStrategy::BoundaryHold(n)) => Some(u64::from(n)),
        _ => None,
    }
}

fn sleep_until_hold(n_ms: u64, what: &str) {
    let wait = remaining(
        epoch().elapsed().as_micros() as u64 + 1,
        LAST_INJECT_US.load(Ordering::SeqCst),
        n_ms,
    );
    if !wait.is_zero() {
        std::thread::sleep(wait);
        if debuglog::enabled() {
            debuglog::note(&format!("boundary-hold {what} {}us", wait.as_micros()));
        }
    }
}

/// Counter of `boundary-hold` injections; the hook reads it before an event.
#[inline]
pub fn inject_seq() -> u64 {
    INJECT_SEQ.load(Ordering::SeqCst)
}

/// `boundary-hold-N`, part 2: a physical boundary key-down waits until N ms
/// after our last injection.
#[inline]
pub fn hold_boundary_before(extra_info: usize, vk: u16, down: bool) {
    if !down || extra_info == input::injected_tag() || !is_boundary_vk(vk) {
        return;
    }
    if let Some(n) = boundary_hold_ms() {
        sleep_until_hold(n, "before");
    }
}

/// `boundary-hold-N`, part 3: the event injected text and lets a boundary key
/// through (`ReplaceThenPass`): delay returning, so the key reaches the app N ms
/// after our text. `seq0` is [`inject_seq`] taken before the event.
#[inline]
pub fn hold_boundary_after(seq0: u64, swallowed: bool, extra_info: usize, vk: u16, down: bool) {
    if swallowed || !down || extra_info == input::injected_tag() || !is_boundary_vk(vk) {
        return;
    }
    if inject_seq() == seq0 {
        return;
    }
    if let Some(n) = boundary_hold_ms() {
        sleep_until_hold(n, "after");
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

    // -- Notepad spell-check / autocorrect strategies --------------------------

    /// The engine context of an injection that rewrites a whole `word_len`-char
    /// word of which `head` was trimmed.
    fn ctx(deletes: u32, insert: &str, word_len: u32, head: &str) -> Context {
        Context {
            deletes,
            insert: insert.to_string(),
            word_len: Some(word_len),
            selectable: true,
            head: head.to_string(),
        }
    }

    fn split_calls(deletes: u32, text: &str) -> Vec<Call> {
        plan(InjectStrategy::Split, deletes, text, Modifiers::NONE, NO_HELD)
    }

    #[test]
    fn select_replace_selects_with_shift_left_and_types_the_text_after() {
        let c = ctx(3, QA_ONE, 3, "");
        let calls = plan_with(InjectStrategy::SelectReplace, 3, QA_ONE, Modifiers::NONE, NO_HELD, &c);
        assert_eq!(calls.len(), 2);
        assert_eq!(delays(&calls), vec![0, 0]);
        let sel = sig(&calls[0].inputs);
        // Shift down, 3 x (Left down, Left up), Shift up.
        assert_eq!(sel.len(), 2 + 3 * 2);
        let tag = input::injected_tag();
        assert_eq!(sel[0], (keys::VK_LSHIFT, SCAN_LSHIFT, 0, tag));
        assert_eq!(sel[7], (keys::VK_LSHIFT, SCAN_LSHIFT, KEYEVENTF_KEYUP.0, tag));
        for pair in sel[1..7].chunks(2) {
            assert_eq!((pair[0].0, pair[0].1), (keys::VK_LEFT, SCAN_LEFT));
            assert_eq!(pair[0].2, KEYEVENTF_EXTENDEDKEY.0);
            assert_eq!(pair[1].2, KEYEVENTF_EXTENDEDKEY.0 | KEYEVENTF_KEYUP.0);
        }
        assert!(sel.iter().all(|e| e.0 != keys::VK_BACK && e.0 != keys::VK_SHIFT));
        // The text is exactly the split text call (Shift + VK capitals).
        assert_eq!(sig(&calls[1].inputs), sig(&split_calls(3, QA_ONE)[1].inputs));
        assert!(!calls[1].inputs.iter().any(is_packet));
    }

    #[test]
    fn select_replace_unicode_types_every_character_as_a_packet() {
        let c = ctx(2, QA_ONE, 2, "");
        let calls = plan_with(InjectStrategy::SelectReplaceUnicode, 2, QA_ONE, Modifiers::NONE, NO_HELD, &c);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].inputs.len(), 2 + 2 * 2);
        assert!(calls[0].inputs.iter().all(|i| !is_packet(i)));
        assert_eq!(calls[1].inputs.len(), 6 * 2);
        assert!(calls[1].inputs.iter().all(is_packet));
    }

    #[test]
    fn select_strategies_fall_back_to_backspaces_when_a_selection_is_unsafe() {
        let good = ctx(2, "ab", 2, "");
        let held = Held { shift: keys::SHIFT_L, caps: false };
        let mut not_selectable = good.clone();
        not_selectable.selectable = false;
        let cases: Vec<(&str, Held, Context)> = vec![
            ("no context", NO_HELD, Context::default()),
            ("more deletes than the word (line start)", NO_HELD, ctx(2, "ab", 1, "")),
            ("combining mark or surrogate on screen", NO_HELD, not_selectable),
            ("user holds Shift", held, good.clone()),
        ];
        for s in [InjectStrategy::SelectReplace, InjectStrategy::SelectReplaceUnicode] {
            for (why, h, c) in &cases {
                let calls = plan_with(s, 2, "ab", Modifiers::NONE, *h, c);
                let want = plan(InjectStrategy::Split, 2, "ab", Modifiers::NONE, *h);
                assert_eq!(sig(&flat(&calls)), sig(&flat(&want)), "{s:?}: {why}");
                assert_eq!(lens(&calls), lens(&want), "{s:?}: {why}");
                assert!(sig(&calls[0].inputs).iter().all(|e| e.0 == keys::VK_BACK), "{s:?}: {why}");
            }
        }
    }

    #[test]
    fn select_with_an_empty_text_deletes_instead_of_leaving_a_selection() {
        let c = ctx(2, "", 2, "");
        let calls = plan_with(InjectStrategy::SelectReplace, 2, "", Modifiers::NONE, NO_HELD, &c);
        assert_eq!(lens(&calls), vec![4]);
        assert!(sig(&calls[0].inputs).iter().all(|e| e.0 == keys::VK_BACK));
    }

    #[test]
    fn select_after_a_restore_chord_releases_the_chord_first() {
        let chord = Modifiers { shift: true, ctrl: true, ..Modifiers::NONE };
        let c = ctx(2, "Tâ", 2, "");
        let calls = plan_with(InjectStrategy::SelectReplace, 2, "Tâ", chord, NO_HELD, &c);
        let first = sig(&calls[0].inputs);
        let sel_start = first.iter().position(|e| e.2 == 0 && e.0 == keys::VK_LSHIFT).unwrap();
        assert!(sel_start > 0, "chord key-ups come first");
        assert!(first[..sel_start].iter().all(|e| e.2 & KEYEVENTF_KEYUP.0 != 0));
        assert!(first.iter().all(|e| e.0 != keys::VK_BACK));
    }

    #[test]
    fn word_retype_deletes_the_whole_word_and_retypes_it_in_one_call() {
        // The word `Vie` + `e`: the engine trimmed the head `Vi`, rewrite 1 -> `ê`.
        let c = ctx(1, "ê", 3, "Vi");
        let calls = plan_with(InjectStrategy::WordRetype, 1, "ê", Modifiers::NONE, NO_HELD, &c);
        let want = input::build_segment_held(3, "Viê", SendMode::Unicode, NO_HELD);
        assert_eq!(calls.len(), 1);
        assert_eq!(sig(&calls[0].inputs), sig(&want));
    }

    #[test]
    fn word_retype_split_has_the_backspace_block_alone() {
        let c = ctx(1, "ê", 3, "Vi");
        let calls = plan_with(InjectStrategy::WordRetypeSplit, 1, "ê", Modifiers::NONE, NO_HELD, &c);
        let want = input::build_segment_held(3, "Viê", SendMode::Unicode, NO_HELD);
        assert_eq!(lens(&calls)[0], 6);
        assert_eq!(calls.len(), 2);
        assert!(sig(&calls[0].inputs).iter().all(|e| e.0 == keys::VK_BACK));
        assert_eq!(sig(&flat(&calls)), sig(&want));
    }

    #[test]
    fn word_retype_without_a_head_is_single_or_split() {
        let none = Context::default();
        let headless = ctx(2, "tă", 2, "");
        // A context built for another injection is ignored.
        let other = ctx(5, "zzzzz", 9, "abcd");
        for c in [&none, &headless, &other] {
            let one = plan_with(InjectStrategy::WordRetype, 2, "tă", Modifiers::NONE, NO_HELD, c);
            let single = plan(InjectStrategy::Single, 2, "tă", Modifiers::NONE, NO_HELD);
            assert_eq!(sig(&flat(&one)), sig(&flat(&single)));
            assert_eq!(one.len(), 1);
            let two = plan_with(InjectStrategy::WordRetypeSplit, 2, "tă", Modifiers::NONE, NO_HELD, c);
            assert_eq!(sig(&flat(&two)), sig(&flat(&split_calls(2, "tă"))));
            assert_eq!(lens(&two), lens(&split_calls(2, "tă")));
        }
    }

    #[test]
    fn word_retype_retypes_a_head_capital_with_shift_and_vk() {
        let c = ctx(2, "ệt", 3, "V");
        let calls = plan_with(InjectStrategy::WordRetypeSplit, 2, "ệt", Modifiers::NONE, NO_HELD, &c);
        let text = sig(&calls[1].inputs);
        assert_eq!(text[0].0, keys::VK_SHIFT);
        assert_eq!(text[1].0, 0x56);
    }

    #[test]
    fn unicode_after_split_delay_is_a_backspace_call_then_delayed_packets() {
        let calls = plan(InjectStrategy::UnicodeAfterSplitDelay(25), 3, QA_ONE, Modifiers::NONE, NO_HELD);
        assert_eq!(calls.len(), 2);
        assert_eq!(delays(&calls), vec![0, 25]);
        assert!(sig(&calls[0].inputs).iter().all(|e| e.0 == keys::VK_BACK));
        assert_eq!(calls[0].inputs.len(), 6);
        assert!(calls[1].inputs.iter().all(is_packet));
        assert_eq!(calls[1].inputs.len(), 12);
        let plain = plan(InjectStrategy::UnicodeAfterSplit, 3, QA_ONE, Modifiers::NONE, NO_HELD);
        assert_eq!(sig(&flat(&calls)), sig(&flat(&plain)), "same events, only the pause is new");
    }

    #[test]
    fn boundary_hold_sends_the_trailing_space_later_on_its_own() {
        // Macro `qa1` -> `QA ONE `: the expansion ends with the Space.
        let text = "QA ONE ";
        let calls = plan(InjectStrategy::BoundaryHold(40), 2, text, Modifiers::NONE, NO_HELD);
        assert_eq!(calls.len(), 3);
        assert_eq!(delays(&calls), vec![0, 0, 40]);
        assert_eq!(calls[0].inputs.len(), 4);
        // Space: VK_SPACE down + up, nothing else in the last call.
        let last = sig(&calls[2].inputs);
        assert_eq!(last.len(), 2);
        assert_eq!(last[0].0, keys::VK_SPACE);
        // Nothing lost or reordered against `split`.
        assert_eq!(sig(&flat(&calls)), sig(&flat(&split_calls(2, text))));
    }

    #[test]
    fn boundary_hold_handles_punctuation_and_a_boundary_only_text() {
        let calls = plan(InjectStrategy::BoundaryHold(10), 1, "ok.", Modifiers::NONE, NO_HELD);
        assert_eq!(delays(&calls), vec![0, 0, 10]);
        assert_eq!(sig(&flat(&calls)), sig(&flat(&split_calls(1, "ok."))));
        let only = plan(InjectStrategy::BoundaryHold(10), 1, " ", Modifiers::NONE, NO_HELD);
        assert_eq!(delays(&only), vec![0, 10]);
        assert_eq!(lens(&only), vec![2, 2]);
    }

    #[test]
    fn boundary_hold_without_a_trailing_boundary_is_split() {
        for t in ["gõ", "Việt", "QA ONE", "a\n"] {
            let calls = plan(InjectStrategy::BoundaryHold(40), 2, t, Modifiers::NONE, NO_HELD);
            assert_eq!(delays(&calls), vec![0, 0], "{t:?}");
            assert_eq!(lens(&calls), lens(&split_calls(2, t)), "{t:?}");
            assert_eq!(sig(&flat(&calls)), sig(&flat(&split_calls(2, t))), "{t:?}");
        }
    }

    #[test]
    fn boundary_hold_keeps_the_shift_repress_in_the_last_call() {
        let chord = Modifiers { shift: true, ctrl: true, ..Modifiers::NONE };
        let held = Held { shift: keys::SHIFT_R, caps: false };
        let calls = plan(InjectStrategy::BoundaryHold(5), 2, "ab ", chord, held);
        let want = input::build_restore(2, "ab ", SendMode::Unicode, chord, held);
        assert_eq!(sig(&flat(&calls)), sig(&want));
        let last = *calls.last().unwrap().inputs.last().unwrap();
        let ki = unsafe { last.Anonymous.ki };
        assert_eq!((ki.wVk.0, ki.dwFlags.0 & KEYEVENTF_KEYUP.0), (keys::VK_RSHIFT, 0));
        assert_eq!(calls.last().unwrap().delay_ms, 5);
    }

    #[test]
    fn the_select_and_retype_strategies_have_no_delays() {
        let c = ctx(2, "ab", 2, "");
        for s in [
            InjectStrategy::SelectReplace,
            InjectStrategy::SelectReplaceUnicode,
            InjectStrategy::WordRetype,
            InjectStrategy::WordRetypeSplit,
        ] {
            let calls = plan_with(s, 2, "ab", Modifiers::NONE, NO_HELD, &c);
            assert!(calls.iter().all(|c| c.delay_ms == 0), "{s:?}");
        }
    }

    #[test]
    fn effective_degrades_only_the_select_strategies() {
        let c = Context::default();
        assert_eq!(
            effective(InjectStrategy::SelectReplace, 2, "ab", NO_HELD, &c),
            InjectStrategy::Split
        );
        assert_eq!(
            effective(InjectStrategy::SelectReplaceUnicode, 2, "ab", NO_HELD, &c),
            InjectStrategy::Split
        );
        assert_eq!(
            effective(InjectStrategy::WordRetype, 2, "ab", NO_HELD, &c),
            InjectStrategy::WordRetype
        );
        let ok = ctx(2, "ab", 2, "");
        assert_eq!(
            effective(InjectStrategy::SelectReplace, 2, "ab", NO_HELD, &ok),
            InjectStrategy::SelectReplace
        );
    }

    #[test]
    fn boundary_classes() {
        for c in [' ', '\t', '.', ',', ';', '!', '?', ')'] {
            assert!(is_boundary_char(c), "{c:?}");
        }
        for c in ['a', 'Z', '5', 'ê', '\n', '\r', '😀'] {
            assert!(!is_boundary_char(c), "{c:?}");
        }
        for vk in [keys::VK_SPACE, keys::VK_RETURN, keys::VK_TAB, 0xBC, 0xBE, 0xBF, 0xDE] {
            assert!(is_boundary_vk(vk), "{vk:#x}");
        }
        for vk in [0x41, 0x30, keys::VK_BACK, keys::VK_LEFT, keys::VK_LSHIFT] {
            assert!(!is_boundary_vk(vk), "{vk:#x}");
        }
        assert!(selectable_char('ệ'));
        assert!(!selectable_char('\u{0302}'));
        assert!(!selectable_char('😀'));
    }

    #[test]
    fn remaining_counts_from_the_last_injection() {
        assert_eq!(remaining(5_000, 0, 40), Duration::ZERO, "no injection yet");
        assert_eq!(remaining(10_000, 10_000, 40), Duration::from_millis(40));
        assert_eq!(remaining(30_000, 10_000, 40), Duration::from_millis(20));
        assert_eq!(remaining(60_000, 10_000, 40), Duration::ZERO, "already past");
    }

    #[test]
    fn the_context_only_applies_to_its_own_injection() {
        let c = ctx(2, "ab", 3, "x");
        assert!(c.applies_to(2, "ab"));
        assert!(!c.applies_to(3, "ab"));
        assert!(!c.applies_to(2, "ac"));
        assert!(!Context::default().applies_to(0, ""));
    }
}
