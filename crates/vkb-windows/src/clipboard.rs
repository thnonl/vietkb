//! Opt-in clipboard send mode (config `typing.use_clipboard`).
//!
//! **Never** on the default path: charter C4 forbids it, and the fast
//! `SendInput` path covers every app we ship for. This exists only for the
//! stubborn targets where Unicode injection is ignored.
//!
//! Ordering is by construction (D-WIN-12..15, `70-impl-notes-windows.md` §9):
//! the replacement happens **synchronously inside the hook callback**, exactly
//! like the Unicode path, so no physical key can overtake it.
//!
//! - **Hook thread (never blocks).** [`replace`] compares
//!   `GetClipboardSequenceNumber` with a snapshot that is already waiting. Match
//!   -> single-try `OpenClipboard`, write our text (with the history/cloud
//!   opt-out formats), close, then ONE `SendInput` (Backspaces + `Ctrl+V`
//!   chord, Shift re-read live). Anything else (mode off, no/lossy/stale
//!   snapshot, clipboard busy, owner hung, any failed call) -> the text is
//!   typed with Unicode `SendInput`, synchronously. The clipboard owner window
//!   is created on the hook thread, which pumps messages, so other programs'
//!   `EmptyClipboard` (`WM_DESTROYCLIPBOARD`) never waits on us.
//! - **Worker thread (pumped).** Owns the fresh snapshot: while the mode is on
//!   it listens with `AddClipboardFormatListener` and re-snapshots on every
//!   change (all `HGLOBAL` formats byte for byte, GDI handle formats never
//!   dereferenced, lossy / oversized / hung owner -> no snapshot). It also does
//!   the restore: after [`SETTLE_MS`] without a paste, only if the sequence
//!   number is still the one right after our write, retried with backoff.
//!   [`shutdown`] restores at once. Snapshots are dropped as soon as the mode is
//!   switched off.
//!
//! Cost of the design (opt-in mode only): every clipboard change is copied once
//! (caps in `clipboard_plan`), and reading the data forces delayed-render owners
//! to render. The release profile is `panic = "abort"`, so `catch_unwind` only
//! protects dev/test builds; the release guarantee is that every call is checked.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::Once;
use std::thread;
use std::time::{Duration, Instant};

use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use vkb_platform::clipboard_plan::{
    classify_format, classify_write, decide_paste, ownership, Ownership, flag_blocks_snapshot, format_count_ok, notification_is_ours,
    snapshot_stable, REFRESH_DELAY_MS,
    null_owner_ok, pick_newest, uses_hook_owner, plain_text_only, text_rescue_allowed, urgent_proof, restore_action, restore_action_text, restore_due, restore_retry, settle_remaining_ms,
    text_fingerprint, snapshot_verdict, within_budget, FormatClass,
    OwnSeqs, PasteDecision, RestoreAction, Verdict, WriteVerdict, OWNED_MARKER, PRIVACY_FORMATS,
    REGISTERED_FIRST, SETTLE_MS,
};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{
    GetLastError, GlobalFree, SetLastError, HANDLE, HGLOBAL, HWND, LPARAM, LRESULT, WIN32_ERROR,
    WPARAM,
};
use windows::Win32::System::DataExchange::{
    AddClipboardFormatListener, CloseClipboard, EmptyClipboard, EnumClipboardFormats,
    GetClipboardData, GetClipboardFormatNameW, GetClipboardOwner, GetClipboardSequenceNumber,
    IsClipboardFormatAvailable, OpenClipboard, RegisterClipboardFormatW, RemoveClipboardFormatListener, SetClipboardData,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, IsHungAppWindow,
    KillTimer, PostMessageW, RegisterClassW, SendMessageTimeoutW, SetTimer, TranslateMessage,
    HWND_MESSAGE, MSG, SMTO_ABORTIFHUNG, SMTO_BLOCK, WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSW,
};

use crate::input::{self, Held, SendMode};
use crate::globals;
use vkb_platform::Modifiers;

/// `CF_UNICODETEXT`.
pub const CF_UNICODETEXT: u32 = 13;

const WM_NULL: u32 = 0;
const WM_TIMER: u32 = 0x0113;
const WM_CLIPBOARDUPDATE: u32 = 0x031D;
const WM_APP_PASTED: u32 = 0x8000 + 0x21;
const WM_APP_GC: u32 = 0x8000 + 0x24;
const WM_APP_URGENT: u32 = 0x8000 + 0x25;
const WM_APP_MODE: u32 = 0x8000 + 0x22;
const WM_APP_FLUSH: u32 = 0x8000 + 0x23;
const TIMER_SETTLE: usize = 1;
const TIMER_REFRESH: usize = 2;

#[derive(Debug, PartialEq, Eq)]
struct FormatData {
    format: u32,
    bytes: Vec<u8>,
}

impl Drop for FormatData {
    /// The user's clipboard must not linger in freed memory.
    fn drop(&mut self) {
        self.bytes.fill(0);
        // Keep the compiler from eliding the stores before the free.
        std::hint::black_box(&mut self.bytes);
        std::sync::atomic::compiler_fence(Ordering::SeqCst);
    }
}

/// A faithful copy of the clipboard as of sequence number `seq`.
struct Snap {
    seq: u32,
    data: Vec<FormatData>,
}

/// The user's clipboard while our temporary text is on it.
struct Session {
    data: Vec<FormatData>,
    /// Sequence numbers of our last write.
    own: OwnSeqs,
    /// When the last paste batch was sent (the settle wait counts from here).
    last_write: Instant,
    /// Fingerprint of the text we wrote: a manager that re-sets it without our
    /// marker still leaves "our" content.
    text_fp: u64,
    /// Restore rounds already spent.
    rounds: u32,
    /// Came from a failed write: restore on owner or sequence proof.
    urgent: bool,
    /// Monotonic per session (under `SHARED`): only the newest is ever
    /// restored, an older copy must never overwrite a newer one.
    generation: u64,
}

#[derive(Default)]
struct Shared {
    snapshot: Option<Snap>,
    session: Option<Session>,
    /// Sequence numbers of the write that put the user's data back (the
    /// adopted snapshot): that notification is ours, no re-capture.
    restored_seq: Option<OwnSeqs>,
    /// Old user data the hook retired; the worker zeroes and frees it so the
    /// hook thread never pays for large buffers.
    garbage: Vec<Vec<FormatData>>,
    /// Data of failed writes the worker must put back (never restored on the
    /// hook thread: up to 64 MB).
    urgent: Vec<(Vec<FormatData>, OwnSeqs)>,
    /// Source of `Session::generation`.
    next_generation: u64,
    /// Sessions whose restore ran out of rounds: kept (never dropped) for the
    /// mode-off / shutdown flush.
    stranded: Vec<Session>,
}

static SHARED: Mutex<Shared> = Mutex::new(Shared {
    snapshot: None,
    session: None,
    restored_seq: None,
    garbage: Vec::new(),
    urgent: Vec::new(),
    next_generation: 0,
    stranded: Vec::new(),
});

/// Hand old user data to the worker for zeroing and freeing.
fn retire(shared: &mut Shared, data: Vec<FormatData>) {
    if data.is_empty() {
        return;
    }
    // Zeroing and freeing (up to 64 MB) happens on the worker, never here.
    shared.garbage.push(data);
    if let Some(worker) = hwnd_of(&WORKER_HWND) {
        // SAFETY: posting to a window we created.
        let _ = unsafe { PostMessageW(worker, WM_APP_GC, WPARAM(0), LPARAM(0)) };
    }
    // No worker: the data stays queued until one exists (or the process ends).
}

/// Take the newest of the active session and the parked ones; every older one is
/// retired (never restored over a newer one).
fn take_newest(shared: &mut Shared) -> Option<Session> {
    let mut all: Vec<Session> = Vec::new();
    all.extend(shared.session.take());
    all.append(&mut shared.stranded);
    let gens: Vec<u64> = all.iter().map(|s| s.generation).collect();
    let newest = all.remove(pick_newest(&gens)?);
    for old in all {
        retire(shared, old.data);
    }
    Some(newest)
}

/// A restore of generation `generation` finished (restored or foreign): every
/// older parked session is obsolete.
fn retire_stranded_older(shared: &mut Shared, generation: u64) {
    for s in std::mem::take(&mut shared.stranded) {
        if s.generation < generation {
            retire(shared, s.data);
        } else {
            shared.stranded.push(s);
        }
    }
}

/// At most one parked session: keep the newest, retire the rest.
fn keep_newest_stranded(shared: &mut Shared) {
    let all = std::mem::take(&mut shared.stranded);
    let gens: Vec<u64> = all.iter().map(|s| s.generation).collect();
    let keep = pick_newest(&gens);
    for (i, s) in all.into_iter().enumerate() {
        if Some(i) == keep {
            shared.stranded.push(s);
        } else {
            retire(shared, s.data);
        }
    }
}

static ENABLED: AtomicBool = AtomicBool::new(false);
/// Message-only clipboard owner on the hook thread (pumped there). 0 = none.
static HOOK_OWNER: AtomicIsize = AtomicIsize::new(0);
/// The worker's listener window. 0 = not running yet.
static WORKER_HWND: AtomicIsize = AtomicIsize::new(0);
static WORKER_ONCE: Once = Once::new();
static FLUSH_ACK: Mutex<Option<Sender<()>>> = Mutex::new(None);

fn hwnd_of(slot: &AtomicIsize) -> Option<HWND> {
    match slot.load(Ordering::SeqCst) {
        0 => None,
        raw => Some(HWND(raw as *mut core::ffi::c_void)),
    }
}

// ---------------------------------------------------------------------------
// Hook thread side
// ---------------------------------------------------------------------------

/// Create the clipboard owner window. Call on the hook thread, which pumps
/// messages (a window nobody pumps would hang other programs' `EmptyClipboard`).
pub fn create_hook_owner() {
    // SAFETY: a message-only window of the system "STATIC" class, destroyed by
    // `destroy_hook_owner` on the same thread.
    let window = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("STATIC"),
            w!(""),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            None,
            None,
            None,
        )
    };
    match window {
        Ok(hwnd) => HOOK_OWNER.store(hwnd.0 as isize, Ordering::SeqCst),
        Err(err) => tracing::warn!(error = %err, "clipboard owner window not created"),
    }
}

/// Counterpart of [`create_hook_owner`] (hook thread, on its way out).
pub fn destroy_hook_owner() {
    // Under the shared lock, the same lock a restore reads the owner under when
    // it takes its session. The handle can still go stale between that read
    // and its use, hence the NULL-owner fallback in `restore_session`.
    let _shared = SHARED.lock();
    let raw = HOOK_OWNER.swap(0, Ordering::SeqCst);
    if raw != 0 {
        // SAFETY: created on this thread.
        let _ = unsafe { DestroyWindow(HWND(raw as *mut core::ffi::c_void)) };
    }
}

/// Switch the mode on/off (idempotent). Off drops every snapshot and restores a
/// pending session; on starts listening and takes the first snapshot.
pub fn set_enabled(on: bool) {
    if ENABLED.swap(on, Ordering::SeqCst) == on {
        return;
    }
    if on {
        ensure_worker();
    }
    if let Some(hwnd) = hwnd_of(&WORKER_HWND) {
        // SAFETY: posting to a window we created.
        let _ = unsafe { PostMessageW(hwnd, WM_APP_MODE, WPARAM(0), LPARAM(0)) };
    }
}

/// Replace `deletes` characters with `text`, synchronously. Pastes through the
/// clipboard when that is provably lossless right now, else types the text.
///
/// `chord` is the modifier chord whose key-up fired this rewrite (released
/// first, as in `build_restore`); `held` is the hook's model of the keys down.
pub fn replace(deletes: u32, text: &str, mode: SendMode, chord: Modifiers, held: Held) {
    let pasted = catch_unwind(AssertUnwindSafe(|| try_paste(deletes, text, mode, chord, held)))
        .unwrap_or(false);
    if !pasted {
        type_directly(deletes, text, mode, chord, held);
    }
}

/// Unicode `SendInput` delivery (the safe fallback), same shape as the
/// non-clipboard path.
fn type_directly(deletes: u32, text: &str, mode: SendMode, chord: Modifiers, held: Held) {
    let inputs = if chord == Modifiers::NONE {
        input::build_segment_held(deletes, text, mode, held)
    } else {
        input::build_restore(deletes, text, mode, chord, held)
    };
    input::send_split(&inputs, deletes, mode);
}

/// Does the current clipboard owner answer a message within `timeout_ms`?
/// `EmptyClipboard` sends `WM_DESTROYCLIPBOARD` to the previous owner
/// synchronously and `GetClipboardData` may ask a delayed-render owner to
/// render; an owner that does not answer would stall the caller.
///
/// Known limit (documented in section 9): an owner that answers `WM_NULL` but
/// then blocks inside its `WM_DESTROYCLIPBOARD` handler can still stall the
/// hook. Typing instead whenever the previous owner is not ours would make the
/// mode unusable (the owner is the program the user copied from), so the probe
/// is the bound.
fn clipboard_owner_responsive(timeout_ms: u32) -> bool {
    let Ok(prev) = (unsafe { GetClipboardOwner() }) else { return true };
    if prev.is_invalid() || Some(prev) == hwnd_of(&HOOK_OWNER) {
        return true;
    }
    if unsafe { IsHungAppWindow(prev) }.as_bool() {
        return false;
    }
    // A window we may not message (UIPI: higher integrity) also answers 0, so
    // the probe fails closed: the hook types instead of pasting.
    // SAFETY: plain call with a short timeout; no result pointer.
    let answer = unsafe {
        SendMessageTimeoutW(
            prev,
            WM_NULL,
            WPARAM(0),
            LPARAM(0),
            SMTO_ABORTIFHUNG | SMTO_BLOCK,
            timeout_ms,
            None,
        )
    };
    answer.0 != 0
}

/// `true` = the replacement was delivered. `false` = nothing was sent and the
/// clipboard holds what it held (or was restored): the caller types instead.
fn try_paste(deletes: u32, text: &str, mode: SendMode, chord: Modifiers, held: Held) -> bool {
    if !ENABLED.load(Ordering::SeqCst) {
        return false;
    }
    // Only the hook thread has the pumped owner window; other sinks type.
    if unsafe { GetCurrentThreadId() } != globals::THREAD_ID.load(Ordering::SeqCst) {
        return false;
    }
    let Some(owner) = hwnd_of(&HOOK_OWNER) else { return false };
    // Never wait on the worker: a busy lock just means "type".
    let Some(mut shared) = SHARED.try_lock() else { return false };
    // The mode may have been switched off while we waited for the lock.
    if !ENABLED.load(Ordering::SeqCst) {
        return false;
    }
    let current = unsafe { GetClipboardSequenceNumber() };
    // A moved number alone does not make our session foreign (late close bump):
    // owner window + marker decide, and the recorded number is refreshed.
    if let Some(session) = shared.session.as_mut() {
        if !session.own.matches(current) {
            if let Ownership::OursAtNewSeq(now) =
                ownership(Some(session.own), owner_marker_ours(Some(owner)), current)
            {
                session.own.after_close = now;
            }
        }
    }
    let decision = decide_paste(
        true,
        shared.session.as_ref().map(|s| s.own),
        shared.snapshot.as_ref().map(|s| s.seq),
        current,
    );
    if decision == PasteDecision::TypeDirectly {
        // A session whose number moved is NOT dropped here: the restore decides
        // (it also recognises our text re-set by a clipboard manager).
        return false;
    }
    // A busy previous owner would block EmptyClipboard (WM_DESTROYCLIPBOARD).
    if !clipboard_owner_responsive(20) {
        return false;
    }
    // Single try: the hook never retries.
    let Some(guard) = ClipboardGuard::open(Some(owner), 1, 0) else { return false };
    // Nobody can write while we hold it: the decision must still hold.
    if unsafe { GetClipboardSequenceNumber() } != current {
        return false;
    }
    let Some(in_open) = write_temp_text(text) else {
        // Possibly emptied half way. Close, hand the user's data to the worker
        // (up to 64 MB: never restored on the hook thread) and type instead.
        // The clipboard is still open here: record its sequence number (and the
        // one after the close) as the proof the worker may restore against.
        let failed_open = unsafe { GetClipboardSequenceNumber() };
        drop(guard);
        let failed_own =
            OwnSeqs { in_open: failed_open, after_close: unsafe { GetClipboardSequenceNumber() } };
        let data = match decision {
            PasteDecision::ContinueSession => shared.session.take().map(|s| s.data),
            _ => shared.snapshot.take().map(|s| s.data),
        };
        if let Some(data) = data {
            shared.urgent.push((data, failed_own));
            if let Some(worker) = hwnd_of(&WORKER_HWND) {
                let _ = unsafe { PostMessageW(worker, WM_APP_URGENT, WPARAM(0), LPARAM(0)) };
            }
        }
        return false;
    };
    // CloseClipboard advances the sequence number (D-WIN-16), so the number that
    // identifies OUR write is only known after the close, and only trusted when
    // owner + marker + a stable bracket say the clipboard is ours.
    drop(guard);
    let own = match verify_write(Some(owner), in_open) {
        WriteVerdict::Ours(after) | WriteVerdict::Unstable(after) => {
            // Unstable keeps the session: user data is never discarded on
            // uncertainty (the latest number is recorded; the in-open one stays
            // ours too).
            OwnSeqs { in_open, after_close: after }
        }
        WriteVerdict::Foreign => {
            // Owner/marker prove somebody else wrote: the clipboard is theirs
            // now. Forget the old copy and type.
            let old = match decision {
                PasteDecision::ContinueSession => shared.session.take().map(|s| s.data),
                _ => shared.snapshot.take().map(|s| s.data),
            };
            if let Some(old) = old {
                retire(&mut shared, old);
            }
            return false;
        }
    };
    match decision {
        PasteDecision::StartSession => {
            if let Some(old) = shared.session.take() {
                retire(&mut shared, old.data); // stale session: not freed on the hook thread
            }
            if let Some(snap) = shared.snapshot.take() {
                // A new session supersedes every parked (older) one.
                for old in std::mem::take(&mut shared.stranded) {
                    retire(&mut shared, old.data);
                }
                shared.next_generation += 1;
                let generation = shared.next_generation;
                shared.session = Some(Session {
                    data: snap.data,
                    own,
                    last_write: Instant::now(),
                    text_fp: text_fingerprint(text),
                    rounds: 0,
                    urgent: false,
                    generation,
                });
            }
        }
        _ => {
            if let Some(session) = shared.session.as_mut() {
                session.own = own;
                session.last_write = Instant::now();
                session.text_fp = text_fingerprint(text);
            }
        }
    }
    drop(shared);

    let inputs = input::build_paste(deletes, mode, chord, held);
    let sent = input::send(&inputs);
    if let Some(worker) = hwnd_of(&WORKER_HWND) {
        // The settle wait starts when the batch has been SENT (the worker stamps
        // the time on receipt): (re)arm it whatever happened to the batch.
        let _ = unsafe { PostMessageW(worker, WM_APP_PASTED, WPARAM(0), LPARAM(0)) };
    }
    if sent == 0 {
        // Blocked outright (UIPI): nothing reached the target, type instead.
        return false;
    }
    if (sent as usize) < inputs.len() {
        tracing::warn!(sent, total = inputs.len(), "clipboard paste batch cut short");
    }
    true
}

/// Put the user's clipboard back now and stop snapshotting (app quit, hook
/// uninstall). Bounded wait.
pub fn shutdown() {
    ENABLED.store(false, Ordering::SeqCst);
    let Some(worker) = hwnd_of(&WORKER_HWND) else { return };
    let (ack_tx, ack_rx) = channel();
    *FLUSH_ACK.lock() = Some(ack_tx);
    // SAFETY: posting to a window we created.
    if unsafe { PostMessageW(worker, WM_APP_FLUSH, WPARAM(0), LPARAM(0)) }.is_ok() {
        let _ = ack_rx.recv_timeout(Duration::from_millis(800));
    }
    FLUSH_ACK.lock().take();
}

// ---------------------------------------------------------------------------
// Worker thread (message loop): snapshots and restore
// ---------------------------------------------------------------------------

fn ensure_worker() {
    WORKER_ONCE.call_once(|| {
        let spawned = thread::Builder::new().name("vkb-clipboard".to_string()).spawn(worker_main);
        if let Err(err) = spawned {
            tracing::error!(error = %err, "cannot start the clipboard worker");
        }
    });
}

fn worker_main() {
    // SAFETY: standard class + message-only window owned by this thread.
    let hwnd = unsafe {
        let hinstance = GetModuleHandleW(None).map(Into::into).unwrap_or_default();
        let class = WNDCLASSW {
            lpfnWndProc: Some(worker_proc),
            hInstance: hinstance,
            lpszClassName: w!("VietKbClipboardWorker"),
            ..Default::default()
        };
        RegisterClassW(&class);
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("VietKbClipboardWorker"),
            w!(""),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            None,
            hinstance,
            None,
        )
    };
    let Ok(hwnd) = hwnd else {
        tracing::error!("clipboard worker window not created");
        return;
    };
    WORKER_HWND.store(hwnd.0 as isize, Ordering::SeqCst);
    apply_mode(hwnd);
    let mut msg = MSG::default();
    // SAFETY: standard message loop; GetMessageW returns 0 on WM_QUIT, -1 on error.
    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.0 > 0 {
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

unsafe extern "system" fn worker_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_CLIPBOARDUPDATE => {
            let _ = catch_unwind(|| on_update(hwnd));
            LRESULT(0)
        }
        WM_APP_PASTED => {
            // The batch has been sent: the settle wait counts from now, and each
            // further paste restarts it.
            if let Some(session) = SHARED.lock().session.as_mut() {
                session.last_write = Instant::now();
            }
            unsafe { SetTimer(hwnd, TIMER_SETTLE, SETTLE_MS as u32, None) };
            LRESULT(0)
        }
        WM_APP_URGENT => {
            let _ = catch_unwind(|| restore_urgent(hwnd));
            LRESULT(0)
        }
        WM_TIMER if wparam.0 == TIMER_REFRESH => {
            let _ = unsafe { KillTimer(hwnd, TIMER_REFRESH) };
            if ENABLED.load(Ordering::SeqCst) {
                let _ = catch_unwind(|| refresh_snapshot(hwnd));
            }
            LRESULT(0)
        }
        WM_TIMER if wparam.0 == TIMER_SETTLE => {
            let _ = unsafe { KillTimer(hwnd, TIMER_SETTLE) };
            let _ = catch_unwind(|| restore_if_due(hwnd));
            LRESULT(0)
        }
        WM_APP_GC => {
            collect_garbage();
            LRESULT(0)
        }
        WM_APP_MODE => {
            let _ = catch_unwind(|| apply_mode(hwnd));
            LRESULT(0)
        }
        WM_APP_FLUSH => {
            let _ = unsafe { KillTimer(hwnd, TIMER_SETTLE) };
            let _ = unsafe { KillTimer(hwnd, TIMER_REFRESH) };
            let _ = catch_unwind(|| {
                // One locked section takes the session and clears the snapshot.
                restore_all_now(hwnd, true);
                collect_garbage();
                let _ = unsafe { RemoveClipboardFormatListener(hwnd) };
            });
            if let Some(ack) = FLUSH_ACK.lock().take() {
                let _ = ack.send(());
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

/// Forget every copy of the user's data kept for pasting. `restored_seq` goes
/// with it: on re-enable the first snapshot is a fresh capture (the restored
/// clipboard still carries the privacy flags we added, so it may stay
/// unsnapshotted until the user copies again; keeping the data in memory just
/// to avoid that would break the "nothing held while the mode is off" rule).
fn clear_snapshot() {
    let mut shared = SHARED.lock();
    shared.snapshot = None;
    shared.restored_seq = None;
}

/// The mode flipped (or the worker just started): listen + first snapshot, or
/// restore + forget everything.
fn apply_mode(hwnd: HWND) {
    if ENABLED.load(Ordering::SeqCst) {
        // SAFETY: our own window.
        let _ = unsafe { AddClipboardFormatListener(hwnd) };
        refresh_snapshot(hwnd);
    } else {
        let _ = unsafe { KillTimer(hwnd, TIMER_SETTLE) };
        let _ = unsafe { KillTimer(hwnd, TIMER_REFRESH) };
        let _ = unsafe { RemoveClipboardFormatListener(hwnd) };
        restore_all_now(hwnd, false);
        collect_garbage();
    }
}

/// `WM_CLIPBOARDUPDATE`: the clipboard changed (our own writes included).
fn on_update(hwnd: HWND) {
    if !ENABLED.load(Ordering::SeqCst) {
        clear_snapshot();
        return;
    }
    {
        let mut shared = SHARED.lock();
        // Read AFTER taking the lock: the hook records its write's sequence
        // number under this lock, so a paste in flight cannot look foreign.
        let current = unsafe { GetClipboardSequenceNumber() };
        let session_own = shared.session.as_ref().map(|s| s.own);
        if notification_is_ours(session_own, shared.restored_seq, current) {
            return; // our temporary text, or the user's data we put back
        }
        // The number is not one we recorded: before dropping anything, ask
        // owner + marker whether it is still our content (late bump).
        if session_own.is_some() || shared.restored_seq.is_some() {
            let proof = owner_marker_ours(hwnd_of(&HOOK_OWNER));
            if let Ownership::OursAtNewSeq(now) = ownership(session_own, proof, current) {
                if let Some(session) = shared.session.as_mut() {
                    session.own.after_close = now;
                }
                return;
            }
            if let Ownership::OursAtNewSeq(now) = ownership(shared.restored_seq, proof, current) {
                if let Some(restored) = shared.restored_seq.as_mut() {
                    restored.after_close = now;
                }
                if let Some(snap) = shared.snapshot.as_mut() {
                    snap.seq = now;
                }
                return;
            }
        }
        shared.snapshot = None; // stale until the new one is ready
        if session_own.is_some() {
            // Not proven ours, but NOT dropped either: a clipboard manager may
            // have re-set our text. The restore decides (sequence / owner /
            // marker / text), and re-snapshots when the content is foreign.
            return;
        }
        shared.restored_seq = None;
    }
    // Never open the clipboard right away: a target may be about to open it for
    // a paste (D-WIN-17), and a burst of changes needs one capture.
    unsafe { SetTimer(hwnd, TIMER_REFRESH, REFRESH_DELAY_MS, None) };
}

fn refresh_snapshot(hwnd: HWND) {
    // Never open the clipboard while a paste session is in flight: the target
    // may be opening it for its Ctrl+V right now (D-WIN-17).
    if SHARED.lock().session.is_some() {
        return;
    }
    let opened = ClipboardGuard::open(Some(hwnd), 5, 20).and_then(|guard| {
        let in_open = unsafe { GetClipboardSequenceNumber() };
        capture(&guard).map(|data| (in_open, data))
    });
    // The guard is closed here. The snapshot's number is read in a bracket with
    // no clipboard open and accepted only if it did not move (a foreign write in
    // the gap would make the number newer than the data: reject).
    let snap = opened.and_then(|(in_open, data)| {
        let seq = stable_seq()?;
        log_capture_bump(in_open, seq);
        Some(Snap { seq, data })
    });
    let mut shared = SHARED.lock();
    if shared.session.is_none() && ENABLED.load(Ordering::SeqCst) {
        shared.snapshot = snap;
    }
}

/// Settle timer: restore only when the last paste was sent long enough ago; a
/// paste that raced the timer re-arms it instead (the target may not have read
/// yet). A restore that cannot finish is requeued, never dropped.
fn restore_if_due(hwnd: HWND) {
    // Age check and take happen in ONE locked section, so a paste cannot slip
    // in between and be restored from under the target.
    let due = {
        let mut shared = SHARED.lock();
        let owner = hwnd_of(&HOOK_OWNER);
        let due_now = match shared.session.as_ref() {
            Some(session) => {
                let elapsed = session.last_write.elapsed().as_millis() as u64;
                if restore_due(elapsed) {
                    true
                } else {
                    let wait = settle_remaining_ms(elapsed);
                    unsafe { SetTimer(hwnd, TIMER_SETTLE, wait as u32, None) };
                    false
                }
            }
            // A parked session gets another go.
            None => !shared.stranded.is_empty(),
        };
        if due_now {
            take_newest(&mut shared).map(|s| (s, owner))
        } else {
            None
        }
    };
    if let Some((session, owner)) = due {
        run_restore(hwnd, session, owner, false);
    }
}

/// One restore attempt for `session`; an unfinished one is requeued (or, at the
/// final quit flush, logged), a finished one retires every older parked session.
fn run_restore(hwnd: HWND, session: Session, owner: Option<HWND>, last: bool) {
    let generation = session.generation;
    match restore_session(session, owner) {
        RestoreOutcome::Done => {
            let mut shared = SHARED.lock();
            retire_stranded_older(&mut shared, generation);
        }
        RestoreOutcome::Retry(back) => {
            if last {
                tracing::error!("clipboard could not be restored at shutdown");
            } else {
                requeue(hwnd, back);
            }
        }
    }
}

/// Put an unfinished restore back and schedule the next round; out of rounds it
/// is parked for the flush. The user's data is never dropped here.
fn requeue(hwnd: HWND, mut session: Session) {
    session.rounds += 1;
    let mut shared = SHARED.lock();
    match restore_retry(session.rounds) {
        Some(delay) if shared.session.is_none() => {
            session.last_write =
                Instant::now().checked_sub(Duration::from_millis(SETTLE_MS)).unwrap_or_else(Instant::now);
            shared.session = Some(session);
            unsafe { SetTimer(hwnd, TIMER_SETTLE, delay, None) };
        }
        _ => {
            tracing::error!(
                rounds = session.rounds,
                "clipboard restore parked; it is retried on the next settle / mode-off / quit"
            );
            shared.stranded.push(session);
            keep_newest_stranded(&mut shared);
            // Still try again later: parked sessions are picked up by the timer.
            unsafe { SetTimer(hwnd, TIMER_SETTLE, 5000, None) };
        }
    }
}

/// Mode off / quit: take everything (session, parked sessions) and clear the
/// snapshot state in ONE locked section, then restore. `last` = quitting: a
/// restore that still fails is logged and given up (the process ends); else it
/// is requeued.
fn restore_all_now(hwnd: HWND, last: bool) {
    let (newest, owner) = {
        let mut shared = SHARED.lock();
        // Only the newest session is restored; older ones are retired.
        let newest = take_newest(&mut shared);
        shared.snapshot = None;
        shared.restored_seq = None;
        (newest, hwnd_of(&HOOK_OWNER))
    };
    if let Some(session) = newest {
        run_restore(hwnd, session, owner, last);
    }
}

/// Worker: put back what a failed write on the hook thread left half-written.
fn restore_urgent(hwnd: HWND) {
    let (jobs, owner) = {
        let mut shared = SHARED.lock();
        (std::mem::take(&mut shared.urgent), hwnd_of(&HOOK_OWNER))
    };
    for (data, own) in jobs {
        let generation = {
            let mut shared = SHARED.lock();
            shared.next_generation += 1;
            shared.next_generation
        };
        let session = Session {
            data,
            own,
            last_write: Instant::now(),
            text_fp: 0,
            rounds: 0,
            urgent: true,
            generation,
        };
        run_restore(hwnd, session, owner, false);
    }
}

/// Worker: zero and free what the hook retired.
fn collect_garbage() {
    let old = std::mem::take(&mut SHARED.lock().garbage);
    drop(old);
}

enum RestoreOutcome {
    /// Restored, or provably superseded by newer foreign data.
    Done,
    /// Busy or incomplete: the session comes back untouched for another round.
    Retry(Session),
}

/// Retried with backoff while the clipboard is busy; only if still ours.
///
/// On success the restored data is ADOPTED as the next snapshot (its sequence
/// number read under the open clipboard), so the mode keeps working without a
/// re-capture: a re-capture would see the privacy flags we just added and, by
/// design, refuse to snapshot. The restored data keeps those privacy flags on
/// purpose: the user's original copy is already in Win+V / cloud history, and
/// re-adding it would duplicate it.
fn restore_session(done: Session, owner: Option<HWND>) -> RestoreOutcome {
    for attempt in 0..5u32 {
        // Our pumped owner first; if that handle went stale (hook thread gone)
        // the last attempts use the NULL owner.
        // `open_owner` only decides which handle OpenClipboard gets; the proof
        // checks (GetClipboardOwner / marker) keep using the hook owner, they do
        // not depend on the handle that opened the clipboard.
        let open_owner = if uses_hook_owner(attempt) { owner } else { None };
        if let Some(guard) = ClipboardGuard::open(open_owner, 1, 0) {
            // Checked while we hold the clipboard: nobody can write in between.
            let now = unsafe { GetClipboardSequenceNumber() };
            let proof = owner_marker_ours(owner);
            let ours = if done.urgent {
                // A failed write: our owner window still owns it, or the number
                // is one recorded at the failure. The NULL owner is no proof.
                let owner_is_ours = owner
                    .is_some_and(|ours| unsafe { GetClipboardOwner() }.is_ok_and(|now| now == ours));
                urgent_proof(owner_is_ours, done.own.matches(now))
            } else {
                let base = restore_action(done.own, proof, now);
                // A manager may have re-set exactly our text without the marker.
                // Only for a plain-text clipboard and the first round; logged.
                let text_equal = base == RestoreAction::Abandon
                    && done.text_fp != 0
                    && list_formats().is_some_and(|ids| {
                        text_rescue_allowed(
                            done.rounds,
                            plain_text_only(ids.iter().map(|(id, n)| (*id, n.as_deref()))),
                        )
                    })
                    && clipboard_text_fp() == Some(done.text_fp);
                if text_equal {
                    tracing::debug!("clipboard text equals ours without the marker; restoring");
                    crate::debuglog::note("clipboard-text-rescue fired");
                }
                restore_action_text(base, text_equal) == RestoreAction::Restore
            };
            if !ours {
                tracing::debug!("clipboard changed after our paste; not restoring");
                drop(guard);
                // The content is foreign: take a fresh snapshot of it later.
                if let Some(worker) = hwnd_of(&WORKER_HWND) {
                    unsafe { SetTimer(worker, TIMER_REFRESH, REFRESH_DELAY_MS, None) };
                }
                let mut shared = SHARED.lock();
                retire(&mut shared, done.data);
                return RestoreOutcome::Done;
            }
            let complete = restore_locked(&done.data);
            let in_open = unsafe { GetClipboardSequenceNumber() };
            drop(guard);
            if !complete {
                tracing::warn!("clipboard restore incomplete; will retry");
                return RestoreOutcome::Retry(done);
            }
            // An empty user clipboard stays empty: no marker, nothing to adopt.
            if done.data.is_empty() {
                return RestoreOutcome::Done;
            }
            // Adopt only what owner + marker + a stable bracket prove is our
            // restored data (never trust a bare number, D-WIN-16).
            // What we just wrote was written through `open_owner`.
            let verdict = verify_write(open_owner, in_open);
            if let (WriteVerdict::Ours(after) | WriteVerdict::Unstable(after), true) =
                (verdict, ENABLED.load(Ordering::SeqCst))
            {
                let mut shared = SHARED.lock();
                shared.restored_seq = Some(OwnSeqs { in_open, after_close: after });
                shared.snapshot = Some(Snap { seq: after, data: done.data });
            }
            return RestoreOutcome::Done;
        }
        thread::sleep(Duration::from_millis(20 << attempt));
    }
    tracing::warn!("clipboard busy; restore will be retried");
    RestoreOutcome::Retry(done)
}

// ---------------------------------------------------------------------------
// Win32 clipboard helpers
// ---------------------------------------------------------------------------

/// Clipboard must be open. Empties it and writes the snapshot back.
fn restore_locked(snapshot: &[FormatData]) -> bool {
    if unsafe { EmptyClipboard() }.is_err() {
        return false;
    }
    let mut complete = true;
    for entry in snapshot {
        if !set_bytes(entry.format, &entry.bytes) {
            tracing::warn!(format = entry.format, "clipboard format not restored");
            complete = false;
        }
    }
    // The restored copy is not a new user action: keep it out of history/cloud
    // (an empty clipboard has nothing to flag).
    if !snapshot.is_empty() {
        set_privacy_flags();
    }
    // Ownership marker: lets the bracket after the close prove this state is
    // ours. Not for an empty snapshot: the user's empty clipboard stays empty.
    if !snapshot.is_empty() && !set_marker() {
        complete = false;
    }
    complete
}

/// Proof that the clipboard is open; closes it on drop (also on unwind).
struct ClipboardGuard;

impl ClipboardGuard {
    fn open(owner: Option<HWND>, tries: u32, wait_ms: u64) -> Option<Self> {
        for attempt in 0..tries.max(1) {
            if unsafe { OpenClipboard(owner.unwrap_or_default()) }.is_ok() {
                return Some(Self);
            }
            if attempt + 1 < tries {
                thread::sleep(Duration::from_millis(wait_ms));
            }
        }
        None
    }
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        let _ = unsafe { CloseClipboard() };
    }
}

fn format_name(id: u32) -> Option<String> {
    if id < REGISTERED_FIRST {
        return None;
    }
    let mut buf = [0u16; 128];
    let n = unsafe { GetClipboardFormatNameW(id, &mut buf) };
    if n <= 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&buf[..n as usize]))
}

/// Every format id with its registered name. `None` when the enumeration
/// failed or there are too many formats (not safely restorable).
fn list_formats() -> Option<Vec<(u32, Option<String>)>> {
    let mut ids = Vec::new();
    let mut current = 0u32;
    loop {
        // SAFETY: plain calls; the clipboard is open (caller holds the guard).
        unsafe { SetLastError(WIN32_ERROR(0)) };
        current = unsafe { EnumClipboardFormats(current) };
        if current == 0 {
            // 0 is both "done" and "failed": only ERROR_SUCCESS means done.
            return (unsafe { GetLastError() } == WIN32_ERROR(0)).then_some(ids);
        }
        if !format_count_ok(ids.len() + 1) {
            return None;
        }
        ids.push((current, format_name(current)));
    }
}

/// Snapshot the open clipboard, or `None` when a faithful round trip is not
/// possible (unknown/handle-only formats, unreadable data, too large, hung
/// owner). `None` means: never paste over it.
fn capture(_open: &ClipboardGuard) -> Option<Vec<FormatData>> {
    // A delayed-render owner that does not answer would block `GetClipboardData`
    // (it has no timeout): skip the snapshot unless the owner answers in 50 ms.
    // Fail closed: an elevated (higher-integrity) owner we may not message (UIPI)
    // answers 0 too, so a clipboard owned by an elevated program is never
    // snapshotted and the hook types instead.
    if !clipboard_owner_responsive(50) {
        return None;
    }
    // Pass 1: ids only. No data is read (or rendered) before the verdict.
    let ids = list_formats()?;
    let verdict = snapshot_verdict(ids.iter().map(|(id, name)| (*id, name.as_deref())));
    if verdict == Verdict::Lossy {
        tracing::debug!("clipboard holds formats that cannot be restored; no snapshot");
        return None;
    }
    // Privacy: the data's owner asked to be kept out of history/cloud/monitors.
    // Honour it before reading any real data: no snapshot, the hook types.
    for (id, name) in &ids {
        if classify_format(*id, name.as_deref()) != FormatClass::Flag {
            continue;
        }
        let name = name.as_deref().unwrap_or_default();
        let value = if name.eq_ignore_ascii_case(PRIVACY_FORMATS[2]) { None } else { read_dword(*id) };
        if flag_blocks_snapshot(name, value) {
            tracing::debug!("clipboard asks to be excluded from monitoring; no snapshot");
            return None;
        }
    }
    // Pass 2: copy the HGLOBAL formats.
    let mut formats = Vec::new();
    let mut total = 0usize;
    for (id, name) in &ids {
        if classify_format(*id, name.as_deref()) != FormatClass::Copy {
            continue;
        }
        let handle = unsafe { GetClipboardData(*id) }.ok().filter(|h| !h.is_invalid())?;
        let global = HGLOBAL(handle.0);
        let size = unsafe { GlobalSize(global) };
        if size == 0 {
            continue;
        }
        if !within_budget(total, size) {
            return None;
        }
        let ptr = unsafe { GlobalLock(global) } as *const u8;
        if ptr.is_null() {
            return None;
        }
        // SAFETY: `ptr` is a locked allocation of `size` bytes owned by the clipboard.
        let bytes = unsafe { std::slice::from_raw_parts(ptr, size) }.to_vec();
        let _ = unsafe { GlobalUnlock(global) };
        total += size;
        formats.push(FormatData { format: *id, bytes });
    }
    Some(formats)
}

/// Clipboard must be open. Fingerprint of the current `CF_UNICODETEXT`.
fn clipboard_text_fp() -> Option<u64> {
    let handle = unsafe { GetClipboardData(CF_UNICODETEXT) }.ok().filter(|h| !h.is_invalid())?;
    let global = HGLOBAL(handle.0);
    let size = unsafe { GlobalSize(global) };
    let ptr = unsafe { GlobalLock(global) } as *const u16;
    if ptr.is_null() {
        return None;
    }
    // SAFETY: bounded by the allocation size.
    let units = unsafe { std::slice::from_raw_parts(ptr, size / 2) };
    let end = units.iter().position(|u| *u == 0).unwrap_or(units.len());
    let fp = text_fingerprint(&String::from_utf16_lossy(&units[..end]));
    let _ = unsafe { GlobalUnlock(global) };
    Some(fp)
}

/// Clipboard must be open. First `DWORD` of a (tiny) flag format.
fn read_dword(id: u32) -> Option<u32> {
    let handle = unsafe { GetClipboardData(id) }.ok().filter(|h| !h.is_invalid())?;
    let global = HGLOBAL(handle.0);
    if unsafe { GlobalSize(global) } < 4 {
        return None;
    }
    let ptr = unsafe { GlobalLock(global) } as *const u32;
    if ptr.is_null() {
        return None;
    }
    // SAFETY: at least 4 bytes, possibly unaligned.
    let value = unsafe { ptr.read_unaligned() };
    let _ = unsafe { GlobalUnlock(global) };
    Some(value)
}

/// Clipboard must be open. Replaces it with `text` (opt-out flags included) and
/// returns the sequence number right after the write, `None` on any failure.
fn write_temp_text(text: &str) -> Option<u32> {
    let mut units: Vec<u16> = text.encode_utf16().collect();
    units.push(0);
    let bytes: Vec<u8> = units.iter().flat_map(|u| u.to_le_bytes()).collect();
    unsafe { EmptyClipboard() }.ok()?;
    if !set_bytes(CF_UNICODETEXT, &bytes) {
        return None;
    }
    set_privacy_flags();
    // Without the marker the bracket could never recognise this write: treat a
    // failure like any other failed write.
    if !set_marker() {
        return None;
    }
    let seq = unsafe { GetClipboardSequenceNumber() };
    (seq != 0).then_some(seq)
}

fn marker_id() -> u32 {
    static ID: OnceCell<u32> = OnceCell::new();
    *ID.get_or_init(|| {
        let wide: Vec<u16> = OWNED_MARKER.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe { RegisterClipboardFormatW(PCWSTR(wide.as_ptr())) }
    })
}

/// Clipboard must be open. One tiny `HGLOBAL`, no nonce: ownership is proved by
/// owner window + marker together.
fn set_marker() -> bool {
    let id = marker_id();
    id != 0 && set_bytes(id, &[1u8])
}

/// One-off `VKB_DEBUG_KEYS` line: does a read-only open/close move the sequence
/// number? (`after_close - in_open` of the first capture; QA reads keys.log.)
fn log_capture_bump(in_open: u32, after_close: u32) {
    static LOGGED: AtomicBool = AtomicBool::new(false);
    if crate::debuglog::enabled() && !LOGGED.swap(true, Ordering::SeqCst) {
        crate::debuglog::note(&format!(
            "clipboard-capture-seq in_open={in_open} after_close={after_close} delta={}",
            after_close.wrapping_sub(in_open)
        ));
    }
}

/// Owner window (or the NULL-owner fallback) AND marker are ours right now,
/// no sequence comparison, no clipboard open needed.
fn owner_marker_ours(expected_owner: Option<HWND>) -> bool {
    let marker = marker_id();
    let owner_ok = match expected_owner {
        Some(ours) => unsafe { GetClipboardOwner() }.is_ok_and(|now| now == ours),
        // NULL owner: the marker alone proves nothing, the sequence must match.
        None => false,
    };
    owner_ok && marker != 0 && unsafe { IsClipboardFormatAvailable(marker) }.is_ok()
}

/// Two reads of the sequence number with no clipboard open; `Some` only when it
/// did not move. Retried a few times (the close bump may land late).
fn stable_seq() -> Option<u32> {
    for _ in 0..3 {
        let s1 = unsafe { GetClipboardSequenceNumber() };
        let s2 = unsafe { GetClipboardSequenceNumber() };
        if let Some(seq) = snapshot_stable(s1, s2) {
            return Some(seq);
        }
        thread::sleep(Duration::from_millis(1));
    }
    None
}

/// After one of OUR writes, with the clipboard closed: owner window + marker +
/// stable sequence number, bracketed. `expected_owner == None` (NULL-owner
/// fallback) additionally requires the number to equal `in_open`.
fn verify_write(expected_owner: Option<HWND>, in_open: u32) -> WriteVerdict {
    let marker = marker_id();
    let mut last = WriteVerdict::Unstable(0);
    for attempt in 0..3 {
        let s1 = unsafe { GetClipboardSequenceNumber() };
        let owner_ok = match expected_owner {
            Some(ours) => unsafe { GetClipboardOwner() }.is_ok_and(|now| now == ours),
            // NULL owner: no window to compare, the number must match too.
            None => null_owner_ok(in_open, s1),
        };
        let marker_present = marker != 0 && unsafe { IsClipboardFormatAvailable(marker) }.is_ok();
        let s2 = unsafe { GetClipboardSequenceNumber() };
        last = classify_write(owner_ok, marker_present, s1, s2);
        if !matches!(last, WriteVerdict::Unstable(_)) {
            break;
        }
        if attempt < 2 {
            // Let a late close bump land; microseconds, never a sleep on the hook.
            std::thread::yield_now();
        }
    }
    last
}

fn privacy_ids() -> &'static [u32; 3] {
    static IDS: OnceCell<[u32; 3]> = OnceCell::new();
    IDS.get_or_init(|| {
        PRIVACY_FORMATS.map(|name| {
            let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
            unsafe { RegisterClipboardFormatW(PCWSTR(wide.as_ptr())) }
        })
    })
}

/// Clipboard must be open. A failure here only costs privacy, never the paste.
fn set_privacy_flags() {
    for id in privacy_ids() {
        if *id != 0 && !set_bytes(*id, &0u32.to_le_bytes()) {
            tracing::debug!(format = *id, "privacy format not set");
        }
    }
}

/// Clipboard must be open. Frees the allocation when the clipboard refused it.
fn set_bytes(format: u32, bytes: &[u8]) -> bool {
    // SAFETY: the allocation is locked, filled within its size, unlocked, and
    // either owned by the clipboard (success) or freed here (failure).
    unsafe {
        let Ok(handle) = GlobalAlloc(GMEM_MOVEABLE, bytes.len().max(1)) else {
            return false;
        };
        let ptr = GlobalLock(handle);
        if ptr.is_null() {
            let _ = GlobalFree(handle);
            return false;
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr.cast::<u8>(), bytes.len());
        let _ = GlobalUnlock(handle);
        if SetClipboardData(format, HANDLE(handle.0)).is_ok() {
            true
        } else {
            let _ = GlobalFree(handle);
            false
        }
    }
}

/// Read the current `CF_UNICODETEXT` (test/diagnostic helper).
pub fn read_text() -> Option<String> {
    let _guard = ClipboardGuard::open(None, 10, 20)?;
    let handle = unsafe { GetClipboardData(CF_UNICODETEXT) }.ok()?;
    if handle.is_invalid() {
        return None;
    }
    let global = HGLOBAL(handle.0);
    let size = unsafe { GlobalSize(global) };
    let ptr = unsafe { GlobalLock(global) } as *const u16;
    if ptr.is_null() {
        return None;
    }
    // SAFETY: bounded by the allocation size.
    let units = unsafe { std::slice::from_raw_parts(ptr, size / 2) };
    let end = units.iter().position(|u| *u == 0).unwrap_or(units.len());
    let text = String::from_utf16_lossy(&units[..end]);
    let _ = unsafe { GlobalUnlock(global) };
    Some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Touches the real clipboard: run explicitly.
    #[test]
    #[ignore = "clobbers the clipboard; run with --ignored on an idle desktop"]
    fn text_round_trip_restores_previous_contents() {
        let previous = read_text();
        let guard = ClipboardGuard::open(None, 10, 20).expect("open");
        write_temp_text("tạm thời").expect("write");
        drop(guard);
        assert_eq!(read_text().as_deref(), Some("tạm thời"));
        let guard = ClipboardGuard::open(None, 10, 20).expect("open");
        let backup = capture(&guard).expect("capture");
        let seq = write_temp_text("khác").expect("write");
        assert_eq!(
            restore_action(OwnSeqs { in_open: seq, after_close: seq }, false, unsafe {
                GetClipboardSequenceNumber()
            }),
            RestoreAction::Restore
        );
        assert!(restore_locked(&backup));
        drop(guard);
        assert_eq!(read_text().as_deref(), Some("tạm thời"));
        // Put the user's clipboard back.
        let guard = ClipboardGuard::open(None, 10, 20).expect("open");
        match previous {
            Some(text) => {
                write_temp_text(&text).expect("restore original");
            }
            None => {
                let _ = unsafe { EmptyClipboard() };
            }
        }
        drop(guard);
    }
}
