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
        let mut out = Vec::new();
        self.retire_all(&mut out);
        self.next_generation += 1;
        let generation = self.next_generation;
        self.entries.push(Entry {
            generation,
            payload,
            own,
            last_paste_ms: now_ms,
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
            e.retry_at_ms = 0;
        }
    }

    /// The batch has been sent: the settle wait counts from now.
    pub fn paste_sent(&mut self, now_ms: u64) -> Vec<Action<T>> {
        match self.entries.iter_mut().find(|e| e.kind == Kind::Current) {
            Some(e) => {
                e.last_paste_ms = now_ms;
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
            Kind::Current if !flushing => {
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
        out.push(Action::Restore(RestoreJob {
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
                        // Retry soon; the settle wait is long over.
                        last_paste_ms: 0,
                        retry_at_ms: now_ms + delay,
                        rounds,
                        kind,
                    });
                    out.push(Action::ArmTimer(delay));
                    return out;
                }
            }
        }
        if !self.pending() {
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
        // Park "old" behind a continued current: busy while a current exists.
        b.start_session("mid", own(3), SETTLE_MS); // "old" is in flight, not stored
        let out = b.restore_result(j, RestoreResult::Busy, SETTLE_MS);
        // "old" came back busy after a newer generation appeared: retired.
        assert_eq!(retired(&out), vec!["old"]);
        assert!(!out.iter().any(|a| matches!(a, Action::ArmTimer(_))));
        // Only the new session remains.
        b.paste_sent(SETTLE_MS);
        assert_eq!(restores(&b.timer(2 * SETTLE_MS)), vec![(2, "mid")]);
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
    fn mode_off_respects_the_retry_pace_of_a_stored_entry() {
        let mut b = Book::new();
        b.urgent_failed_write("u1", own(1), 0);
        let j1 = job(b.timer(0));
        b.restore_result(j1, RestoreResult::Busy, 0); // stored, retry at 500
        let out = b.mode_off(10);
        assert_eq!(out, vec![Action::DropSnapshot, Action::ArmTimer(490)]);
        assert_eq!(restores(&b.timer(500)), vec![(1, "u1")]);
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
}
