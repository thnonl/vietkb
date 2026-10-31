//! Session / generation bookkeeping of the Windows clipboard send mode, as one
//! pure state machine (no OS calls, generic over the user-data payload `T`).
//!
//! `clipboard.rs` feeds it events (a paste started, a restore attempt ended, the
//! mode was switched off, the timer fired ...) and executes the [`Action`]s it
//! returns with Win32 calls. All ordering rules live here, so they are unit
//! tested exhaustively:
//!
//! - **Newest wins.** Every session has a monotonic generation. Only the newest
//!   stored generation is ever restored; starting a session, a failed write, or
//!   *any* finished restore retires every stored entry (current, parked,
//!   urgent) with an older generation. A restore in flight that comes back busy
//!   after a newer generation appeared is retired, not requeued.
//! - **Never drop user data in normal operation.** A busy / incomplete restore
//!   is retried with backoff and finally parked, still kept. Data is dropped
//!   only when it is superseded, found foreign, or a bounded deadline (mode-off
//!   5 s, quit 800 ms) runs out; then it is zeroed ([`Action::Retire`]) and the
//!   give-up is reported.
//! - **No snapshotting while a restore is pending** ([`SessionBook::refresh_allowed`]).

use crate::clipboard_plan::{restore_retry, OwnSeqs, SETTLE_MS};

/// Mode-off: total time the restore may keep retrying a busy clipboard.
pub const MODE_OFF_BUDGET_MS: u64 = 5_000;
/// Mode-off retry pause.
pub const MODE_OFF_RETRY_MS: u64 = 250;
/// Quit: fixed deadline; 700 ms here + the shutdown ack stay inside 800 ms.
pub const QUIT_BUDGET_MS: u64 = 700;
/// Quit retry pause.
pub const QUIT_RETRY_MS: u64 = 20;
/// A restore that ran out of normal rounds keeps being retried this often.
pub const PARKED_RETRY_MS: u64 = 5_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    /// The session whose temporary text is on the clipboard (the hook may
    /// continue it).
    Current,
    /// A restore that could not finish and is waiting for its next round.
    Parked,
    /// Data of a failed write: restore as soon as possible.
    Urgent,
}

#[derive(Debug)]
struct Entry<T> {
    generation: u64,
    payload: T,
    own: OwnSeqs,
    last_paste_ms: u64,
    /// The settle wait is over (a requeued restore): no further wait.
    settled: bool,
    retry_at_ms: u64,
    rounds: u32,
    kind: Kind,
}

/// One restore attempt handed to the executor (the payload is out of the book
/// while it runs; hand the whole job back with [`SessionBook::restore_result`]).
#[derive(Debug, PartialEq, Eq)]
pub struct RestoreJob<T> {
    pub generation: u64,
    pub payload: T,
    pub own: OwnSeqs,
    pub rounds: u32,
    pub urgent: bool,
    /// Flush jobs (mode-off / quit): the most this attempt may spend opening a
    /// busy clipboard (deadline - now). `None` = the normal backoff.
    pub max_wait_ms: Option<u64>,
}

/// How a restore attempt ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestoreResult {
    /// Restored. `adopt`: sequence numbers of the restored state, to keep it as
    /// the next snapshot (`None` = nothing to adopt, e.g. an empty clipboard).
    Done { adopt: Option<OwnSeqs> },
    /// The clipboard could not be opened.
    Busy,
    /// Written only partly. `own`: the numbers after that partial write, so the
    /// next round can still prove ownership.
    Incomplete { own: Option<OwnSeqs> },
    /// The clipboard now holds somebody else's content: our copy is obsolete.
    Foreign,
}

/// What the executor must do.
#[derive(Debug, PartialEq, Eq)]
pub enum Action<T> {
    /// Run this restore attempt, then report with `restore_result`.
    Restore(RestoreJob<T>),
    /// Zero and free this user data.
    Retire(T),
    /// A restore ended for good. `restored`: it was written back (adopt it as
    /// the snapshot when `adopt` is set); else it is obsolete: zero it.
    Finished { payload: T, restored: bool, adopt: Option<OwnSeqs> },
    /// (Re)arm the restore timer to fire in `ms`.
    ArmTimer(u64),
    /// Nothing is pending any more: stop the restore timer.
    StopTimer,
    /// Take a fresh snapshot (after a debounce).
    Snapshot,
    /// Do not snapshot now (a restore is pending); the snapshot is stale.
    SkipSnapshot,
    /// Forget the held snapshot.
    DropSnapshot,
    /// A deadline ran out: this generation's data was dropped (log it).
    GiveUp(u64),
}

#[derive(Debug)]
pub struct SessionBook<T> {
    next_generation: u64,
    entries: Vec<Entry<T>>,
    inflight: Option<u64>,
    deadline_ms: Option<u64>,
    flush_retry_ms: u64,
}

impl<T> Default for SessionBook<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SessionBook<T> {
    pub const fn new() -> Self {
        Self {
            next_generation: 0,
            entries: Vec::new(),
            inflight: None,
            deadline_ms: None,
            flush_retry_ms: MODE_OFF_RETRY_MS,
        }
    }

    // ---- queries ---------------------------------------------------------

    /// Anything not yet restored or dropped: the current session, parked or
    /// urgent data, or a restore in flight.
    pub fn pending(&self) -> bool {
        !self.entries.is_empty() || self.inflight.is_some()
    }

    /// A re-snapshot may open the clipboard only when nothing is pending.
    pub fn refresh_allowed(&self) -> bool {
        !self.pending()
    }

    /// A restore attempt is running (its payload is out of the book).
    pub fn inflight(&self) -> bool {
        self.inflight.is_some()
    }

    pub fn current_own(&self) -> Option<OwnSeqs> {
        self.entries.iter().find(|e| e.kind == Kind::Current).map(|e| e.own)
    }

    /// Late close bump: owner + marker still prove the session is ours.
    pub fn set_current_own(&mut self, own: OwnSeqs) {
        if let Some(e) = self.entries.iter_mut().find(|e| e.kind == Kind::Current) {
            e.own = own;
        }
    }

    // ---- hook events -----------------------------------------------------

    fn retire_all(&mut self, out: &mut Vec<Action<T>>) {
        for e in self.entries.drain(..) {
            out.push(Action::Retire(e.payload));
        }
    }

    fn retire_older_than(&mut self, generation: u64, out: &mut Vec<Action<T>>) {
        let mut keep = Vec::new();
        for e in std::mem::take(&mut self.entries) {
            if e.generation < generation {
                out.push(Action::Retire(e.payload));
            } else {
                keep.push(e);
            }
        }
        self.entries = keep;
    }

    /// A paste started from a fresh snapshot (`payload` = the user's data). The
    /// new session is the newest: everything older is retired.
    pub fn start_session(&mut self, payload: T, own: OwnSeqs, now_ms: u64) -> (u64, Vec<Action<T>>) {
        // The hook refuses to start a session while a restore runs (it has no
        // snapshot then: re-snapshotting is blocked while anything is pending).
        debug_assert!(self.inflight.is_none(), "start_session while a restore is in flight");
        let mut out = Vec::new();
        self.retire_all(&mut out);
        self.next_generation += 1;
        let generation = self.next_generation;
        self.entries.push(Entry {
            generation,
            payload,
            own,
            last_paste_ms: now_ms,
            settled: false,
            retry_at_ms: 0,
            rounds: 0,
            kind: Kind::Current,
        });
        (generation, out)
    }

    /// The next paste of the same session.
    pub fn continue_session(&mut self, own: OwnSeqs, now_ms: u64) {
        if let Some(e) = self.entries.iter_mut().find(|e| e.kind == Kind::Current) {
            e.own = own;
            e.last_paste_ms = now_ms;
            e.settled = false;
            e.retry_at_ms = 0;
        }
    }

    /// The batch has been sent: the settle wait counts from now.
    pub fn paste_sent(&mut self, now_ms: u64) -> Vec<Action<T>> {
        match self.entries.iter_mut().find(|e| e.kind == Kind::Current) {
            Some(e) => {
                e.last_paste_ms = now_ms;
                e.settled = false;
                e.retry_at_ms = 0;
                vec![Action::ArmTimer(SETTLE_MS)]
            }
            None => Vec::new(),
        }
    }

    /// Owner + marker prove somebody else wrote: forget the current session.
    pub fn discard_current(&mut self) -> Vec<Action<T>> {
        let mut out = Vec::new();
        if let Some(pos) = self.entries.iter().position(|e| e.kind == Kind::Current) {
            out.push(Action::Retire(self.entries.remove(pos).payload));
        }
        if !self.pending() {
            out.push(Action::StopTimer);
        }
        out
    }

    /// A write on the current session failed half way: restore it now.
    pub fn current_to_urgent(&mut self, own: OwnSeqs, _now_ms: u64) -> Vec<Action<T>> {
        match self.entries.iter_mut().find(|e| e.kind == Kind::Current) {
            Some(e) => {
                e.kind = Kind::Urgent;
                e.own = own;
                e.retry_at_ms = 0;
                vec![Action::ArmTimer(0)]
            }
            None => Vec::new(),
        }
    }

    /// A write from a fresh snapshot failed half way: its data is the newest
    /// user data and must be put back now. Older stored data is retired.
    pub fn urgent_failed_write(
        &mut self,
        payload: T,
        own: OwnSeqs,
        now_ms: u64,
    ) -> (u64, Vec<Action<T>>) {
        let mut out = Vec::new();
        self.retire_all(&mut out);
        self.next_generation += 1;
        let generation = self.next_generation;
        self.entries.push(Entry {
            generation,
            payload,
            own,
            last_paste_ms: now_ms,
            settled: false,
            retry_at_ms: 0,
            rounds: 0,
            kind: Kind::Urgent,
        });
        out.push(Action::ArmTimer(0));
        (generation, out)
    }

    // ---- worker events ---------------------------------------------------

    /// The listener saw a clipboard change. `ours`: provably our own write.
    pub fn notification(&self, ours: bool) -> Vec<Action<T>> {
        if ours {
            Vec::new()
        } else if self.pending() {
            // The restore decides what the new content is; the snapshot is
            // stale and no capture may open the clipboard meanwhile.
            vec![Action::SkipSnapshot]
        } else {
            vec![Action::DropSnapshot, Action::Snapshot]
        }
    }

    fn give_up_all(&mut self, out: &mut Vec<Action<T>>) {
        for e in self.entries.drain(..) {
            out.push(Action::GiveUp(e.generation));
            out.push(Action::Retire(e.payload));
        }
        out.push(Action::StopTimer);
    }

    /// The restore timer fired.
    pub fn timer(&mut self, now_ms: u64) -> Vec<Action<T>> {
        let mut out = Vec::new();
        if self.inflight.is_some() {
            return out; // the running attempt re-arms the timer itself
        }
        if self.entries.is_empty() {
            out.push(Action::StopTimer);
            return out;
        }
        if self.deadline_ms.is_some_and(|d| now_ms >= d) {
            self.give_up_all(&mut out);
            return out;
        }
        let newest = self
            .entries
            .iter()
            .enumerate()
            .max_by_key(|(i, e)| (e.generation, *i))
            .map(|(i, _)| i)
            .unwrap_or(0);
        let generation = self.entries[newest].generation;
        // Only the newest generation is ever restored.
        self.retire_older_than(generation, &mut out);
        let e = &self.entries[0];
        let flushing = self.deadline_ms.is_some();
        let retry_wait = e.retry_at_ms.saturating_sub(now_ms);
        let settle_wait = match e.kind {
            Kind::Current if !flushing && !e.settled => {
                SETTLE_MS.saturating_sub(now_ms.saturating_sub(e.last_paste_ms))
            }
            _ => 0,
        };
        let wait = retry_wait.max(settle_wait);
        if wait > 0 {
            out.push(Action::ArmTimer(wait));
            return out;
        }
        let e = self.entries.remove(0);
        self.inflight = Some(e.generation);
        let max_wait_ms = self.deadline_ms.map(|d| d.saturating_sub(now_ms));
        out.push(Action::Restore(RestoreJob {
            max_wait_ms,
            generation: e.generation,
            payload: e.payload,
            own: e.own,
            rounds: e.rounds,
            urgent: e.kind == Kind::Urgent,
        }));
        out
    }

    /// A restore attempt ended.
    pub fn restore_result(
        &mut self,
        job: RestoreJob<T>,
        result: RestoreResult,
        now_ms: u64,
    ) -> Vec<Action<T>> {
        let mut out = Vec::new();
        self.inflight = None;
        match result {
            RestoreResult::Done { adopt } => {
                out.push(Action::Finished { payload: job.payload, restored: true, adopt });
                // Any finished restore makes every older generation obsolete.
                self.retire_older_than(job.generation, &mut out);
            }
            RestoreResult::Foreign => {
                out.push(Action::Finished { payload: job.payload, restored: false, adopt: None });
                self.retire_older_than(job.generation, &mut out);
            }
            RestoreResult::Busy | RestoreResult::Incomplete { .. } => {
                let own = match result {
                    RestoreResult::Incomplete { own: Some(own) } => own,
                    _ => job.own,
                };
                if self.entries.iter().any(|e| e.generation > job.generation) {
                    // A newer generation exists: this copy is obsolete.
                    out.push(Action::Retire(job.payload));
                } else if self.deadline_ms.is_some_and(|d| now_ms >= d) {
                    out.push(Action::GiveUp(job.generation));
                    out.push(Action::Retire(job.payload));
                    self.give_up_all(&mut out);
                    return out;
                } else {
                    let rounds = job.rounds + 1;
                    let delay = if self.deadline_ms.is_some() {
                        self.flush_retry_ms
                    } else {
                        restore_retry(job.rounds).map_or(PARKED_RETRY_MS, u64::from)
                    };
                    let has_current = self.entries.iter().any(|e| e.kind == Kind::Current);
                    let kind = if job.urgent {
                        Kind::Urgent
                    } else if has_current {
                        Kind::Parked
                    } else {
                        Kind::Current
                    };
                    self.entries.push(Entry {
                        generation: job.generation,
                        payload: job.payload,
                        own,
                        // The settle wait is long over.
                        last_paste_ms: now_ms,
                        settled: true,
                        retry_at_ms: now_ms + delay,
                        rounds,
                        kind,
                    });
                    out.push(Action::ArmTimer(delay));
                    return out;
                }
            }
        }
        if self.pending() {
            // Whatever is still stored needs a timer; `timer` computes the wait.
            out.push(Action::ArmTimer(0));
        } else {
            out.push(Action::StopTimer);
        }
        out
    }

    // ---- mode / lifecycle ------------------------------------------------

    /// The mode is on again: no deadline.
    pub fn mode_on(&mut self) {
        self.deadline_ms = None;
    }

    /// Mode off: restore once (the newest), drop every snapshot, retry a busy
    /// clipboard for at most [`MODE_OFF_BUDGET_MS`], then zero and give up.
    pub fn mode_off(&mut self, now_ms: u64) -> Vec<Action<T>> {
        self.begin_flush(now_ms, MODE_OFF_BUDGET_MS, MODE_OFF_RETRY_MS)
    }

    /// Quit: same, with the fixed [`QUIT_BUDGET_MS`] deadline (urgent jobs
    /// included).
    pub fn quit(&mut self, now_ms: u64) -> Vec<Action<T>> {
        self.begin_flush(now_ms, QUIT_BUDGET_MS, QUIT_RETRY_MS)
    }

    fn begin_flush(&mut self, now_ms: u64, budget_ms: u64, retry_ms: u64) -> Vec<Action<T>> {
        self.deadline_ms = Some(now_ms + budget_ms);
        self.flush_retry_ms = retry_ms;
        // Every stored payload is due NOW: a flush attempts each one at least
        // once before it may give up, whatever retry pace it was parked at.
        for e in &mut self.entries {
            e.retry_at_ms = now_ms;
        }
        let mut out = vec![Action::DropSnapshot];
        out.extend(self.timer(now_ms));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Book = SessionBook<&'static str>;

    fn own(n: u32) -> OwnSeqs {
        OwnSeqs { in_open: n, after_close: n + 1 }
    }

    fn restores(actions: &[Action<&'static str>]) -> Vec<(u64, &'static str)> {
        actions
            .iter()
            .filter_map(|a| match a {
                Action::Restore(j) => Some((j.generation, j.payload)),
                _ => None,
            })
            .collect()
    }

    fn retired(actions: &[Action<&'static str>]) -> Vec<&'static str> {
        actions
            .iter()
            .filter_map(|a| match a {
                Action::Retire(p) => Some(*p),
                _ => None,
            })
            .collect()
    }

    fn job(actions: Vec<Action<&'static str>>) -> RestoreJob<&'static str> {
        actions
            .into_iter()
            .find_map(|a| match a {
                Action::Restore(j) => Some(j),
                _ => None,
            })
            .expect("a restore")
    }

    fn done() -> RestoreResult {
        RestoreResult::Done { adopt: Some(own(90)) }
    }

    // -- settle wait ----------------------------------------------------

    #[test]
    fn settle_wait_counts_from_the_paste() {
        let mut b = Book::new();
        let (g, out) = b.start_session("user", own(1), 0);
        assert_eq!(g, 1);
        assert!(out.is_empty());
        assert_eq!(b.paste_sent(100), vec![Action::ArmTimer(SETTLE_MS)]);
        // Early timer: rearm for the remainder, nothing restored.
        assert_eq!(b.timer(600), vec![Action::ArmTimer(SETTLE_MS - 500)]);
        // Due.
        assert_eq!(restores(&b.timer(100 + SETTLE_MS)), vec![(1, "user")]);
    }

    #[test]
    fn a_further_paste_restarts_the_wait() {
        let mut b = Book::new();
        b.start_session("user", own(1), 0);
        b.paste_sent(0);
        b.continue_session(own(3), 1000);
        b.paste_sent(1000);
        assert_eq!(b.timer(SETTLE_MS), vec![Action::ArmTimer(1000)]);
        assert_eq!(restores(&b.timer(1000 + SETTLE_MS)).len(), 1);
    }

    // -- never drop in normal operation -----------------------------------

    #[test]
    fn busy_restore_is_requeued_and_never_dropped() {
        let mut b = Book::new();
        b.start_session("user", own(1), 0);
        b.paste_sent(0);
        let mut now = SETTLE_MS;
        let mut j = job(b.timer(now));
        // Far more busy rounds than MAX_RESTORE_ROUNDS: always kept.
        for round in 0..20u32 {
            let out = b.restore_result(j, RestoreResult::Busy, now);
            assert!(retired(&out).is_empty(), "round {round}: data must be kept");
            assert!(!out.iter().any(|a| matches!(a, Action::GiveUp(_))));
            let Some(Action::ArmTimer(delay)) = out.last() else { panic!("{out:?}") };
            now += *delay;
            j = job(b.timer(now));
            assert_eq!(j.generation, 1);
            assert_eq!(j.rounds, round + 1);
        }
        assert!(b.pending());
    }

    #[test]
    fn busy_backoff_then_parked_pace() {
        let mut b = Book::new();
        b.start_session("user", own(1), 0);
        b.paste_sent(0);
        let mut j = job(b.timer(SETTLE_MS));
        let mut delays = Vec::new();
        for _ in 0..8 {
            let out = b.restore_result(j, RestoreResult::Busy, 10_000);
            let Some(Action::ArmTimer(d)) = out.last() else { panic!() };
            delays.push(*d);
            j = job(b.timer(10_000 + d));
        }
        assert_eq!(&delays[..6], &[500, 500, 1000, 1000, 2000, 2000]);
        assert_eq!(&delays[6..], &[PARKED_RETRY_MS, PARKED_RETRY_MS]);
    }

    #[test]
    fn incomplete_restore_keeps_the_data_and_the_new_proof() {
        let mut b = Book::new();
        b.start_session("user", own(1), 0);
        b.paste_sent(0);
        let j = job(b.timer(SETTLE_MS));
        let out = b.restore_result(j, RestoreResult::Incomplete { own: Some(own(40)) }, SETTLE_MS);
        assert!(retired(&out).is_empty());
        // The session is current again and carries the post-write numbers.
        assert_eq!(b.current_own(), Some(own(40)));
        let j = job(b.timer(SETTLE_MS + 500));
        assert_eq!(j.own, own(40));
    }

    #[test]
    fn busy_without_new_numbers_keeps_the_old_proof() {
        let mut b = Book::new();
        b.start_session("user", own(1), 0);
        b.paste_sent(0);
        let j = job(b.timer(SETTLE_MS));
        b.restore_result(j, RestoreResult::Incomplete { own: None }, SETTLE_MS);
        assert_eq!(b.current_own(), Some(own(1)));
    }

    #[test]
    fn a_session_continues_while_its_restore_is_requeued() {
        let mut b = Book::new();
        b.start_session("user", own(1), 0);
        b.paste_sent(0);
        let j = job(b.timer(SETTLE_MS));
        b.restore_result(j, RestoreResult::Busy, SETTLE_MS);
        // The requeued session is current again: the hook may paste into it.
        assert!(b.current_own().is_some());
        b.continue_session(own(5), SETTLE_MS + 10);
        b.paste_sent(SETTLE_MS + 10);
        assert_eq!(b.timer(SETTLE_MS + 20), vec![Action::ArmTimer(SETTLE_MS - 10)]);
    }

    // -- done / foreign ----------------------------------------------------

    #[test]
    fn done_hands_the_data_back_for_adoption() {
        let mut b = Book::new();
        b.start_session("user", own(1), 0);
        b.paste_sent(0);
        let j = job(b.timer(SETTLE_MS));
        let out = b.restore_result(j, done(), SETTLE_MS);
        assert_eq!(
            out,
            vec![
                Action::Finished { payload: "user", restored: true, adopt: Some(own(90)) },
                Action::StopTimer
            ]
        );
        assert!(!b.pending());
        assert!(b.refresh_allowed());
    }

    #[test]
    fn foreign_content_makes_the_copy_obsolete() {
        let mut b = Book::new();
        b.start_session("user", own(1), 0);
        b.paste_sent(0);
        let j = job(b.timer(SETTLE_MS));
        let out = b.restore_result(j, RestoreResult::Foreign, SETTLE_MS);
        assert_eq!(
            out[0],
            Action::Finished { payload: "user", restored: false, adopt: None }
        );
        assert!(!b.pending());
    }

    // -- the newest generation wins ----------------------------------------

    #[test]
    fn new_session_retires_every_older_entry() {
        let mut b = Book::new();
        b.start_session("a", own(1), 0);
        b.paste_sent(0);
        let j = job(b.timer(SETTLE_MS));
        b.restore_result(j, RestoreResult::Busy, SETTLE_MS); // a: current again
        let (_, out) = b.start_session("b", own(5), SETTLE_MS + 1);
        assert_eq!(retired(&out), vec!["a"]);
        assert_eq!(b.current_own(), Some(own(5)));
    }

    #[test]
    fn parked_plus_new_session_never_restores_the_old_one() {
        let mut b = Book::new();
        b.start_session("old", own(1), 0);
        b.paste_sent(0);
        let j = job(b.timer(SETTLE_MS));
        // A newer generation appears while "old" is being restored (an urgent
        // job; the hook itself never starts a session during a restore).
        let (g, _) = b.urgent_failed_write("mid", own(3), SETTLE_MS);
        assert_eq!(g, 2);
        let out = b.restore_result(j, RestoreResult::Busy, SETTLE_MS);
        // "old" came back busy after a newer generation appeared: retired,
        // and the timer is re-armed for what remains.
        assert_eq!(retired(&out), vec!["old"]);
        assert_eq!(out.last(), Some(&Action::ArmTimer(0)));
        assert_eq!(restores(&b.timer(SETTLE_MS + 1)), vec![(2, "mid")]);
    }

    #[test]
    fn two_urgent_jobs_first_busy_never_restores_over_the_second() {
        // job2 arrives while job1's restore is in flight.
        let mut b = Book::new();
        let (g1, out) = b.urgent_failed_write("u1", own(1), 0);
        assert_eq!(g1, 1);
        assert_eq!(out, vec![Action::ArmTimer(0)]);
        let j1 = job(b.timer(0));
        let (g2, _) = b.urgent_failed_write("u2", own(5), 1);
        assert_eq!(g2, 2);
        // job1 comes back busy: a newer generation is stored, so it is retired.
        let out = b.restore_result(j1, RestoreResult::Busy, 2);
        assert_eq!(retired(&out), vec!["u1"]);
        // job2 restores and finishes; nothing of generation 1 is left.
        let j2 = job(b.timer(10));
        assert_eq!(j2.generation, 2);
        let out = b.restore_result(j2, done(), 10);
        assert!(matches!(out[0], Action::Finished { restored: true, .. }));
        assert!(!b.pending());
    }

    #[test]
    fn two_urgent_jobs_queued_together_restore_only_the_newest() {
        let mut b = Book::new();
        b.urgent_failed_write("u1", own(1), 0);
        // Second failure while the first is still queued: the first is retired.
        let (_, out) = b.urgent_failed_write("u2", own(5), 1);
        assert_eq!(retired(&out), vec!["u1"]);
        assert_eq!(restores(&b.timer(2)), vec![(2, "u2")]);
    }

    #[test]
    fn a_requeued_older_job_is_retired_by_a_newer_done() {
        // job1 busy and requeued (nothing newer yet); job2 appears later.
        let mut b = Book::new();
        b.urgent_failed_write("u1", own(1), 0);
        let j1 = job(b.timer(0));
        let out = b.restore_result(j1, RestoreResult::Busy, 1);
        assert!(retired(&out).is_empty());
        let (_, out) = b.urgent_failed_write("u2", own(5), 2);
        assert_eq!(retired(&out), vec!["u1"]);
    }

    #[test]
    fn any_done_retires_older_entries() {
        // An older urgent entry is stored when a newer session finishes.
        let mut b = Book::new();
        b.urgent_failed_write("u1", own(1), 0); // gen 1, urgent
        let j = job(b.timer(0));
        // Fabricate a parked entry of gen 1 and a current of gen 3 around it.
        b.restore_result(j, RestoreResult::Busy, 0); // gen 1 back (urgent)
        // A newer restore job completes: simulate by starting gen 2 manually.
        let (g2, out) = b.start_session("s2", own(7), 1);
        assert_eq!(retired(&out), vec!["u1"]);
        b.paste_sent(1);
        let j2 = job(b.timer(1 + SETTLE_MS));
        assert_eq!(j2.generation, g2);
        let out = b.restore_result(j2, done(), 1 + SETTLE_MS);
        assert!(matches!(out[0], Action::Finished { restored: true, .. }));
        assert!(!b.pending());
    }

    #[test]
    fn newest_is_picked_when_several_are_stored() {
        let mut b = Book::new();
        b.urgent_failed_write("u1", own(1), 0);
        let j1 = job(b.timer(0));
        b.restore_result(j1, RestoreResult::Busy, 0); // u1 stored (urgent, retry later)
        // Retry time reached: the only entry is restored.
        assert_eq!(restores(&b.timer(10_000)), vec![(1, "u1")]);
    }

    // -- urgent ------------------------------------------------------------

    #[test]
    fn current_to_urgent_restores_immediately_and_keeps_the_generation() {
        let mut b = Book::new();
        let (g, _) = b.start_session("user", own(1), 0);
        b.paste_sent(0);
        assert_eq!(b.current_to_urgent(own(8), 5), vec![Action::ArmTimer(0)]);
        let j = job(b.timer(5)); // no settle wait for urgent data
        assert_eq!((j.generation, j.own, j.urgent), (g, own(8), true));
    }

    #[test]
    fn urgent_job_stays_urgent_when_busy() {
        let mut b = Book::new();
        b.urgent_failed_write("u", own(1), 0);
        let j = job(b.timer(0));
        b.restore_result(j, RestoreResult::Busy, 0);
        let j = job(b.timer(500));
        assert!(j.urgent);
        assert_eq!(j.rounds, 1);
    }

    // -- mode off ----------------------------------------------------------

    #[test]
    fn mode_off_restores_once_and_drops_snapshots() {
        let mut b = Book::new();
        b.start_session("user", own(1), 0);
        b.paste_sent(0);
        let out = b.mode_off(100); // settle not over, but flushing
        assert_eq!(out[0], Action::DropSnapshot);
        assert_eq!(restores(&out), vec![(1, "user")]);
    }

    #[test]
    fn mode_off_busy_retries_to_the_budget_then_zeroes_and_stops() {
        let mut b = Book::new();
        b.start_session("user", own(1), 0);
        b.paste_sent(0);
        let out = b.mode_off(0);
        let mut j = job(out);
        let mut now = 0;
        let mut rounds = 0;
        loop {
            let out = b.restore_result(j, RestoreResult::Busy, now);
            if out.iter().any(|a| matches!(a, Action::GiveUp(_))) {
                // Zeroed, timer stopped.
                assert_eq!(retired(&out), vec!["user"]);
                assert!(out.contains(&Action::StopTimer));
                break;
            }
            let Some(Action::ArmTimer(d)) = out.last() else { panic!("{out:?}") };
            assert_eq!(*d, MODE_OFF_RETRY_MS);
            now += *d;
            rounds += 1;
            assert!(rounds < 100, "must terminate");
            let next = b.timer(now);
            if next.iter().any(|a| matches!(a, Action::GiveUp(_))) {
                assert_eq!(retired(&next), vec!["user"]);
                assert!(next.contains(&Action::StopTimer));
                break;
            }
            j = job(next);
        }
        assert!(now <= MODE_OFF_BUDGET_MS);
        assert!(!b.pending());
    }

    #[test]
    fn a_flush_attempts_a_parked_entry_immediately() {
        let mut b = Book::new();
        b.urgent_failed_write("u1", own(1), 0);
        let j1 = job(b.timer(0));
        b.restore_result(j1, RestoreResult::Busy, 0); // stored, retry at 500
        let out = b.mode_off(10);
        assert_eq!(out[0], Action::DropSnapshot);
        // Due right now, not at its retry time.
        let j = job(out);
        assert_eq!((j.generation, j.max_wait_ms), (1, Some(MODE_OFF_BUDGET_MS)));
    }

    #[test]
    fn quit_with_a_parked_entry_attempts_once_then_gives_up() {
        let mut b = Book::new();
        b.start_session("user", own(1), 0);
        b.paste_sent(0);
        // Park it at the slow pace (many busy rounds).
        let mut now = SETTLE_MS;
        let mut j = job(b.timer(now));
        for _ in 0..8 {
            let out = b.restore_result(j, RestoreResult::Busy, now);
            let Some(Action::ArmTimer(d)) = out.last() else { panic!() };
            now += *d;
            j = job(b.timer(now));
        }
        let out = b.restore_result(j, RestoreResult::Busy, now);
        assert_eq!(out.last(), Some(&Action::ArmTimer(PARKED_RETRY_MS)));
        // Quit with the next retry 5 s away: exactly one attempt right now ...
        let out = b.quit(now + 10);
        let j = job(out);
        assert_eq!(j.max_wait_ms, Some(QUIT_BUDGET_MS));
        // ... which used up the whole budget, so the busy result is the end.
        let out = b.restore_result(j, RestoreResult::Busy, now + 10 + QUIT_BUDGET_MS);
        assert_eq!(out[0], Action::GiveUp(1));
        assert_eq!(retired(&out), vec!["user"]);
        assert!(out.contains(&Action::StopTimer));
        assert!(!b.pending());
    }

    #[test]
    fn flush_jobs_carry_the_remaining_budget() {
        let mut b = Book::new();
        b.urgent_failed_write("u", own(1), 0);
        let j = job(b.quit(100));
        assert_eq!(j.max_wait_ms, Some(QUIT_BUDGET_MS));
        b.restore_result(j, RestoreResult::Busy, 100 + 300);
        let j = job(b.timer(100 + 300 + QUIT_RETRY_MS));
        assert_eq!(j.max_wait_ms, Some(QUIT_BUDGET_MS - 300 - QUIT_RETRY_MS));
        // Normal restores keep the backoff (no cap).
        let mut n = Book::new();
        n.urgent_failed_write("u", own(1), 0);
        assert_eq!(job(n.timer(0)).max_wait_ms, None);
    }

    #[test]
    fn leftover_entries_get_a_timer_after_every_result() {
        // Done of an older job while a newer entry is stored.
        let mut b = Book::new();
        b.urgent_failed_write("a", own(1), 0);
        let j = job(b.timer(0));
        b.urgent_failed_write("b", own(5), 1);
        let out = b.restore_result(j, RestoreResult::Foreign, 2);
        assert_eq!(out.last(), Some(&Action::ArmTimer(0)));
    }

    #[test]
    fn a_requeued_session_needs_no_second_settle_wait() {
        let mut b = Book::new();
        // Early in the process: now < SETTLE_MS must not delay the retry.
        b.start_session("user", own(1), 0);
        let j = job(b.mode_off(0));
        b.mode_on();
        b.restore_result(j, RestoreResult::Busy, 0);
        assert_eq!(restores(&b.timer(500)).len(), 1);
    }

    #[test]
    fn mode_on_clears_the_deadline() {
        let mut b = Book::new();
        b.start_session("user", own(1), 0);
        let j = job(b.mode_off(0));
        b.restore_result(j, done(), 1);
        b.mode_on();
        // Not flushing any more: a new session waits for its settle time.
        b.start_session("again", own(9), 100);
        b.paste_sent(100);
        assert_eq!(b.timer(200), vec![Action::ArmTimer(SETTLE_MS - 100)]);
    }

    // -- quit --------------------------------------------------------------

    #[test]
    fn quit_busy_gives_up_at_the_fixed_deadline_and_drains_urgent() {
        let mut b = Book::new();
        b.urgent_failed_write("u", own(1), 0);
        let out = b.quit(1000);
        let mut j = job(out);
        let mut now = 1000;
        let mut gave_up = false;
        while now <= 1000 + QUIT_BUDGET_MS + QUIT_RETRY_MS {
            let out = b.restore_result(j, RestoreResult::Busy, now);
            if out.iter().any(|a| matches!(a, Action::GiveUp(_))) {
                assert_eq!(retired(&out), vec!["u"]);
                gave_up = true;
                break;
            }
            now += QUIT_RETRY_MS;
            let next = b.timer(now);
            if next.iter().any(|a| matches!(a, Action::GiveUp(_))) {
                assert_eq!(retired(&next), vec!["u"]);
                gave_up = true;
                break;
            }
            j = job(next);
        }
        assert!(gave_up);
        assert!(now - 1000 <= QUIT_BUDGET_MS + QUIT_RETRY_MS);
        assert!(!b.pending());
    }

    #[test]
    fn quit_restores_immediately_when_the_clipboard_is_free() {
        let mut b = Book::new();
        b.start_session("user", own(1), 0);
        b.paste_sent(0);
        let out = b.quit(50);
        let j = job(out);
        let out = b.restore_result(j, done(), 51);
        assert!(matches!(out[0], Action::Finished { restored: true, .. }));
        assert!(!b.pending());
    }

    // -- notifications -----------------------------------------------------

    #[test]
    fn our_own_notification_changes_nothing() {
        let mut b = Book::new();
        b.start_session("user", own(1), 0);
        assert!(b.notification(true).is_empty());
        assert!(b.pending());
    }

    #[test]
    fn notification_during_a_pending_restore_skips_snapshotting() {
        let mut b = Book::new();
        b.start_session("user", own(1), 0);
        b.paste_sent(0);
        // Foreign-looking notification with the session current ...
        assert_eq!(b.notification(false), vec![Action::SkipSnapshot]);
        // ... with the restore in flight ...
        let j = job(b.timer(SETTLE_MS));
        assert_eq!(b.notification(false), vec![Action::SkipSnapshot]);
        assert!(!b.refresh_allowed());
        // ... and while the data is parked after a busy round.
        b.restore_result(j, RestoreResult::Busy, SETTLE_MS);
        assert_eq!(b.notification(false), vec![Action::SkipSnapshot]);
        assert!(!b.refresh_allowed());
        // The session is untouched by all of this.
        assert!(b.current_own().is_some());
    }

    #[test]
    fn notification_without_a_pending_restore_resnapshots() {
        let b = Book::new();
        assert_eq!(b.notification(false), vec![Action::DropSnapshot, Action::Snapshot]);
        assert!(b.refresh_allowed());
    }

    #[test]
    fn refresh_is_blocked_while_urgent_or_parked_data_is_pending() {
        let mut b = Book::new();
        b.urgent_failed_write("u", own(1), 0);
        assert!(!b.refresh_allowed());
        let j = job(b.timer(0));
        assert!(!b.refresh_allowed(), "in flight");
        b.restore_result(j, done(), 0);
        assert!(b.refresh_allowed());
    }

    #[test]
    fn foreign_discard_and_late_bump() {
        let mut b = Book::new();
        b.start_session("user", own(1), 0);
        b.set_current_own(own(20));
        assert_eq!(b.current_own(), Some(own(20)));
        let out = b.discard_current();
        assert_eq!(retired(&out), vec!["user"]);
        assert!(out.contains(&Action::StopTimer));
        assert_eq!(b.current_own(), None);
    }

    #[test]
    fn timer_with_nothing_pending_stops() {
        let mut b = Book::new();
        assert_eq!(b.timer(5), vec![Action::StopTimer]);
    }

    // -- deterministic randomized property tests ---------------------------

    use std::collections::{HashMap, HashSet};

    /// xorshift64*, no dependencies.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 >> 12;
            self.0 ^= self.0 << 25;
            self.0 ^= self.0 >> 27;
            self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
        }
        fn below(&mut self, n: u64) -> u64 {
            self.next() % n.max(1)
        }
    }

    impl<T> SessionBook<T> {
        fn stored_payloads(&self) -> Vec<(u64, &T)> {
            self.entries.iter().map(|e| (e.generation, &e.payload)).collect()
        }
    }

    /// Why `apply` is running: decides which retirements are legitimate.
    #[derive(Clone, Copy)]
    enum Ctx {
        /// `discard_current` (the only event that may retire without a newer
        /// generation).
        Discard,
        /// The event created or finished this generation; everything it retires
        /// must be older.
        Gen(u64),
        /// Anything else: a retirement needs a newer generation in the book.
        Plain,
    }

    /// The model executor: tracks where every payload id is.
    struct Model {
        book: SessionBook<u32>,
        now: u64,
        armed: Option<u64>,
        next_id: u32,
        created: Vec<u32>,
        gens: HashMap<u32, u64>,
        exited: HashSet<u32>,
        inflight: Option<RestoreJob<u32>>,
        flushing: bool,
        /// Absolute time of the flush deadline while one is active.
        deadline: Option<u64>,
        last_done_gen: u64,
        restores: HashMap<u32, u32>,
        gave_up: u32,
    }

    impl Model {
        fn new() -> Self {
            Self {
                book: SessionBook::new(),
                now: 1,
                armed: None,
                next_id: 0,
                created: Vec::new(),
                gens: HashMap::new(),
                exited: HashSet::new(),
                inflight: None,
                flushing: false,
                deadline: None,
                last_done_gen: 0,
                restores: HashMap::new(),
                gave_up: 0,
            }
        }

        fn new_id(&mut self) -> u32 {
            self.next_id += 1;
            self.created.push(self.next_id);
            self.next_id
        }

        fn exit(&mut self, id: u32) {
            assert!(self.created.contains(&id), "unknown payload {id}");
            assert!(self.exited.insert(id), "payload {id} exited twice");
        }

        fn apply(&mut self, actions: Vec<Action<u32>>, ctx: Ctx) {
            // The newest generation alive after this batch: stored, restoring, or
            // the one the event itself created / finished.
            let batch_restore = actions
                .iter()
                .filter_map(|a| match a {
                    Action::Restore(j) => Some(j.generation),
                    _ => None,
                })
                .max();
            let newest = [
                self.book.stored_payloads().iter().map(|(g, _)| *g).max(),
                self.inflight.as_ref().map(|j| j.generation),
                batch_restore,
                match ctx {
                    Ctx::Gen(g) => Some(g),
                    _ => None,
                },
            ]
            .into_iter()
            .flatten()
            .max()
            .unwrap_or(0);
            for a in actions {
                match a {
                    Action::Restore(job) => {
                        assert!(self.inflight.is_none(), "two restores in flight");
                        // Only the newest generation: everything else was retired.
                        assert!(
                            self.book.stored_payloads().is_empty(),
                            "a restore started with other data still stored"
                        );
                        assert!(
                            job.generation > self.last_done_gen,
                            "restore of gen {} after a finished gen {}",
                            job.generation,
                            self.last_done_gen
                        );
                        if self.flushing {
                            assert!(job.max_wait_ms.is_some(), "flush job without a budget");
                        } else {
                            assert_eq!(job.max_wait_ms, None);
                        }
                        *self.restores.entry(job.payload).or_default() += 1;
                        self.inflight = Some(job);
                    }
                    Action::Retire(id) => {
                        // Outside a flush data is only retired when superseded
                        // by a newer generation (or by discard_current).
                        if !self.flushing && !matches!(ctx, Ctx::Discard) {
                            assert!(
                                self.gens[&id] < newest,
                                "payload {id} (gen {}) retired outside a flush with no newer generation (newest {newest})",
                                self.gens[&id]
                            );
                        }
                        self.exit(id)
                    }
                    Action::Finished { payload, .. } => self.exit(payload),
                    Action::ArmTimer(ms) => self.armed = Some(self.now + ms),
                    Action::StopTimer => self.armed = None,
                    Action::GiveUp(_) => {
                        assert!(self.flushing, "give-up outside a flush");
                        let deadline = self.deadline.expect("a flush has a deadline");
                        assert!(
                            self.now >= deadline,
                            "gave up at {} before the deadline {deadline}",
                            self.now
                        );
                        self.gave_up += 1;
                    }
                    Action::Snapshot | Action::SkipSnapshot | Action::DropSnapshot => {}
                }
            }
        }

        fn result(&mut self, r: &mut Rng) {
            let Some(job) = self.inflight.take() else { return };
            let generation = job.generation;
            let result = match r.below(6) {
                0 => RestoreResult::Done { adopt: Some(own(70)) },
                1 => RestoreResult::Done { adopt: None },
                2 => RestoreResult::Busy,
                3 => RestoreResult::Incomplete { own: Some(own(71)) },
                4 => RestoreResult::Incomplete { own: None },
                _ => RestoreResult::Foreign,
            };
            let done = matches!(result, RestoreResult::Done { .. } | RestoreResult::Foreign);
            let actions = self.book.restore_result(job, result, self.now);
            if done {
                self.last_done_gen = self.last_done_gen.max(generation);
            }
            let ctx = if done { Ctx::Gen(generation) } else { Ctx::Plain };
            self.apply(actions, ctx);
        }

        fn check(&self) {
            let stored: Vec<u32> =
                self.book.stored_payloads().into_iter().map(|(_, p)| *p).collect();
            let unique: HashSet<u32> = stored.iter().copied().collect();
            assert_eq!(unique.len(), stored.len(), "payload stored twice");
            // Conservation: every payload is in exactly one place or exited once.
            let inflight_id = self.inflight.as_ref().map(|j| j.payload);
            for id in &self.created {
                let places = u32::from(stored.contains(id))
                    + u32::from(inflight_id == Some(*id))
                    + u32::from(self.exited.contains(id));
                assert_eq!(places, 1, "payload {id} is in {places} places");
            }
            // Single flight, in step with the book.
            assert_eq!(self.inflight.is_some(), self.book.inflight());
            // Timer liveness.
            if self.book.pending() && !self.book.inflight() {
                assert!(self.armed.is_some(), "pending data without an armed timer");
            }
            assert!(stored.len() <= 2, "more than two stored entries");
        }

        fn step(&mut self, r: &mut Rng) {
            self.now += r.below(1500);
            if self.armed.is_some_and(|t| self.now >= t) {
                self.armed = None;
                let actions = self.book.timer(self.now);
                self.apply(actions, Ctx::Plain);
            }
            match r.below(16) {
                0 | 1 if !self.flushing && self.inflight.is_none() => {
                    let id = self.new_id();
                    let (generation, a) = self.book.start_session(id, own(id), self.now);
                    self.gens.insert(id, generation);
                    self.apply(a, Ctx::Gen(generation));
                    let a = self.book.paste_sent(self.now);
                    self.apply(a, Ctx::Plain);
                }
                2 if !self.flushing && self.book.current_own().is_some() => {
                    self.book.continue_session(own(9), self.now);
                    let a = self.book.paste_sent(self.now);
                    self.apply(a, Ctx::Plain);
                }
                3 if !self.flushing => {
                    let a = self.book.current_to_urgent(own(11), self.now);
                    self.apply(a, Ctx::Plain);
                }
                4 if !self.flushing => {
                    let id = self.new_id();
                    let (generation, a) = self.book.urgent_failed_write(id, own(id), self.now);
                    self.gens.insert(id, generation);
                    self.apply(a, Ctx::Gen(generation));
                }
                5 => {
                    let a = self.book.discard_current();
                    self.apply(a, Ctx::Discard);
                }
                6 => {
                    let a = self.book.timer(self.now);
                    self.apply(a, Ctx::Plain);
                }
                7..=10 => self.result(r),
                11 => {
                    let a = self.book.notification(r.below(2) == 0);
                    self.apply(a, Ctx::Plain);
                }
                12 if !self.flushing && r.below(4) == 0 => {
                    self.flushing = true;
                    let (a, budget) = if r.below(2) == 0 {
                        (self.book.mode_off(self.now), MODE_OFF_BUDGET_MS)
                    } else {
                        (self.book.quit(self.now), QUIT_BUDGET_MS)
                    };
                    self.deadline = Some(self.now + budget);
                    self.apply(a, Ctx::Plain);
                }
                13 if self.flushing && r.below(3) == 0 => {
                    self.flushing = false;
                    self.deadline = None;
                    self.book.mode_on();
                }
                _ => {}
            }
            self.check();
        }
    }

    #[test]
    fn randomized_events_keep_every_invariant() {
        for seed in 1..=8u64 {
            let mut r = Rng(0x9E37_79B9_7F4A_7C15 ^ (seed << 17));
            let mut m = Model::new();
            for _ in 0..10_000 {
                m.step(&mut r);
            }
            // The run exercised the interesting paths.
            assert!(m.next_id > 100, "seed {seed}: too few payloads");
            assert!(m.restores.values().any(|n| *n > 1), "seed {seed}: no retried restore");
            assert!(m.gave_up > 0, "seed {seed}: no flush gave up");
        }
    }

    #[test]
    fn flush_with_an_always_busy_clipboard_attempts_and_ends_in_time() {
        let mut flushes = 0;
        for seed in 1..=300u64 {
            let mut r = Rng(0xD1B5_4A32_D192_ED03 ^ (seed << 13));
            let mut m = Model::new();
            // Warm up without flushing.
            for _ in 0..r.below(40) {
                m.step(&mut r);
                if m.flushing {
                    break;
                }
            }
            if m.flushing {
                continue;
            }
            // Payloads present at the flush, and the newest generation among them.
            let mut present: Vec<u32> =
                m.book.stored_payloads().into_iter().map(|(_, p)| *p).collect();
            if let Some(j) = &m.inflight {
                present.push(j.payload);
            }
            let newest = present.iter().copied().max_by_key(|id| m.gens[id]);
            // Attempts of the newest payload BEFORE the flush: it must grow.
            let attempts_before =
                newest.map(|n| m.restores.get(&n).copied().unwrap_or(0));
            let quit = r.below(2) == 0;
            let (budget, pace) = if quit {
                (QUIT_BUDGET_MS, QUIT_RETRY_MS)
            } else {
                (MODE_OFF_BUDGET_MS, MODE_OFF_RETRY_MS)
            };
            m.flushing = true;
            m.deadline = Some(m.now + budget);
            flushes += 1;
            let start = m.now;
            let actions = if quit { m.book.quit(m.now) } else { m.book.mode_off(m.now) };
            m.apply(actions, Ctx::Plain);
            // Always-busy executor, time follows the timers exactly.
            for _ in 0..10_000 {
                if let Some(job) = m.inflight.take() {
                    let spent = r.below(job.max_wait_ms.unwrap_or(0) + 1);
                    m.now += spent;
                    let a = m.book.restore_result(job, RestoreResult::Busy, m.now);
                    m.apply(a, Ctx::Plain);
                    m.check();
                    continue;
                }
                match m.armed {
                    Some(t) => {
                        m.now = m.now.max(t);
                        m.armed = None;
                        let a = m.book.timer(m.now);
                        m.apply(a, Ctx::Plain);
                        m.check();
                    }
                    None => break,
                }
            }
            assert!(!m.book.pending(), "seed {seed}: a flush left data behind");
            assert!(m.inflight.is_none());
            for id in &present {
                assert!(m.exited.contains(id), "seed {seed}: payload {id} never exited");
            }
            // The newest payload present at the flush got a NEW attempt during it.
            if let (Some(n), Some(before)) = (newest, attempts_before) {
                let after = m.restores.get(&n).copied().unwrap_or(0);
                assert!(
                    after > before,
                    "seed {seed}: newest payload {n} had {before} attempts at the flush, {after} after"
                );
            }
            assert!(
                m.now - start <= budget + pace,
                "seed {seed}: flush took {} ms (budget {budget} + pace {pace})",
                m.now - start
            );
        }
        assert!(flushes > 100, "too few flushes exercised: {flushes}");
    }
}
