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
//!   -> single-try `OpenClipboard`, write our text (privacy opt-out formats and
//!   the ownership marker included), close, then ONE `SendInput` (Backspaces +
//!   `Ctrl+V` chord). Anything else -> the text is typed with Unicode
//!   `SendInput`, synchronously. The clipboard owner window is created on the
//!   hook thread, which pumps messages.
//! - **Worker thread (pumped).** Keeps the fresh snapshot (clipboard format
//!   listener) and executes the restores.
//! - **All session bookkeeping lives in the pure state machine
//!   `vkb_platform::clipboard_book::SessionBook`** (generations, newest-wins,
//!   retry/park, mode-off and quit deadlines, snapshot gating), unit tested on
//!   every host. This file only executes the [`Action`]s it returns.
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
use vkb_platform::clipboard_book::{
    Action, RestoreJob, RestoreResult, SessionBook, QUIT_RETRY_MS,
};
use vkb_platform::clipboard_plan::{
    classify_format, decide_paste, flag_blocks_snapshot, format_count_ok,
    garbage_admit, notification_is_ours, open_retry_delay, ownership,
    our_text_bytes, plain_text_only, restore_action, restore_decision, should_clear,
    text_fingerprint, write_verdict, OwnerCheck, Rescue,
    snapshot_stable, snapshot_verdict, uses_hook_owner, within_budget, FormatClass,
    OwnSeqs, Ownership, PasteDecision, RestoreAction, Verdict, WriteVerdict, OWNED_MARKER,
    PRIVACY_FORMATS, REFRESH_DELAY_MS, REGISTERED_FIRST,
};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{
    GetLastError, GlobalFree, SetLastError, HANDLE, HGLOBAL, HWND, LPARAM, LRESULT, WIN32_ERROR,
    WPARAM,
};
use windows::Win32::System::DataExchange::{
    AddClipboardFormatListener, CloseClipboard, EmptyClipboard, EnumClipboardFormats,
    GetClipboardData, GetClipboardFormatNameW, GetClipboardOwner, GetClipboardSequenceNumber,
    IsClipboardFormatAvailable, OpenClipboard, RegisterClipboardFormatW,
    RemoveClipboardFormatListener, SetClipboardData,
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

use crate::globals;
use crate::input::{self, Held, SendMode};
use vkb_platform::Modifiers;

/// `CF_UNICODETEXT`.
pub const CF_UNICODETEXT: u32 = 13;

const WM_NULL: u32 = 0;
const WM_TIMER: u32 = 0x0113;
const WM_CLIPBOARDUPDATE: u32 = 0x031D;
const WM_APP_PASTED: u32 = 0x8000 + 0x21;
const WM_APP_MODE: u32 = 0x8000 + 0x22;
const WM_APP_FLUSH: u32 = 0x8000 + 0x23;
const WM_APP_GC: u32 = 0x8000 + 0x24;
/// Run the restore timer logic now (urgent data is waiting).
const WM_APP_KICK: u32 = 0x8000 + 0x25;
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

/// Identity of the text we wrote: fingerprint and length in UTF-16 units (the
/// length is compared first, so a huge foreign text is never decoded).
#[derive(Clone, Copy)]
struct OurText {
    fp: u64,
    units: usize,
}

impl OurText {
    fn of(text: &str) -> Self {
        Self { fp: text_fingerprint(text), units: text.encode_utf16().count() }
    }
}

type Book = SessionBook<Vec<FormatData>>;
type Act = Action<Vec<FormatData>>;

struct Shared {
    snapshot: Option<Snap>,
    /// Sequence numbers of the write that put the user's data back (the adopted
    /// snapshot): that notification is ours, no re-capture.
    restored_seq: Option<OwnSeqs>,
    /// Session / generation bookkeeping (pure, see `clipboard_book`).
    book: Book,
    /// The text we wrote last (the narrow text rescue and the post-give-up clear
    /// compare the clipboard with it; only the newest session is ever restored).
    text: OurText,
    /// Old user data the hook retired; the worker zeroes and frees it so the
    /// hook thread never pays for large buffers (bounded by bytes).
    garbage: Vec<Vec<FormatData>>,
    garbage_bytes: usize,
}

static SHARED: Mutex<Shared> = Mutex::new(Shared {
    snapshot: None,
    restored_seq: None,
    book: SessionBook::new(),
    text: OurText { fp: 0, units: 0 },
    garbage: Vec::new(),
    garbage_bytes: 0,
});

/// Milliseconds on a process-wide monotonic clock (the book is clock-free).
fn now_ms() -> u64 {
    static START: OnceCell<Instant> = OnceCell::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as u64
}

fn data_bytes(data: &[FormatData]) -> usize {
    data.iter().map(|f| f.bytes.len()).sum()
}

/// Hand old user data to the worker for zeroing and freeing. Above the byte cap
/// (worker far behind, rare) it is zeroed and freed right here. Worst case on
/// the hook thread: `start_session` / a failed write retire up to two entries
/// at once, each at most `MAX_TOTAL_BYTES` (64 MiB), so up to 2 x 64 MiB of
/// `memset` (tens of milliseconds), and only when more than 128 MiB is already
/// queued.
fn retire(shared: &mut Shared, data: Vec<FormatData>) {
    let bytes = data_bytes(&data);
    if bytes == 0 {
        return;
    }
    if !garbage_admit(shared.garbage_bytes, bytes) {
        tracing::warn!(bytes, "clipboard garbage queue full; zeroing on this thread");
        drop(data);
        return;
    }
    shared.garbage_bytes += bytes;
    shared.garbage.push(data);
    if let Some(worker) = hwnd_of(&WORKER_HWND) {
        // SAFETY: posting to a window we created.
        let _ = unsafe { PostMessageW(worker, WM_APP_GC, WPARAM(0), LPARAM(0)) };
    }
    // No worker: the data stays queued until one exists (or the process ends).
}

/// What the executor still has to do outside the lock.
#[derive(Default)]
struct Todo {
    arm_ms: Option<u64>,
    stop_timer: bool,
    snapshot: bool,
    /// A deadline ran out: empty the clipboard if our text is still on it.
    clear_ours: bool,
    restore: Option<RestoreJob<Vec<FormatData>>>,
}

/// Execute the state-machine actions that only touch `Shared`; collect the ones
/// that need Win32 calls or the worker's window into a [`Todo`].
fn exec_locked(shared: &mut Shared, actions: Vec<Act>) -> Todo {
    let mut todo = Todo::default();
    for action in actions {
        match action {
            Action::Restore(job) => todo.restore = Some(job),
            Action::Retire(data) => retire(shared, data),
            Action::Finished { payload, restored, adopt } => {
                match (restored, adopt) {
                    (true, Some(own)) if ENABLED.load(Ordering::SeqCst) => {
                        // Keep what we put back as the next snapshot (a
                        // re-capture would see our own privacy flags).
                        shared.restored_seq = Some(own);
                        if let Some(old) =
                            shared.snapshot.replace(Snap { seq: own.after_close, data: payload })
                        {
                            retire(shared, old.data);
                        }
                    }
                    (restored, _) => {
                        retire(shared, payload);
                        if !restored {
                            // The content is foreign: snapshot it later.
                            todo.snapshot = true;
                        }
                    }
                }
            }
            Action::ArmTimer(ms) => todo.arm_ms = Some(ms),
            Action::StopTimer => {
                todo.stop_timer = true;
                todo.arm_ms = None;
            }
            Action::Snapshot => todo.snapshot = true,
            Action::SkipSnapshot | Action::DropSnapshot => {
                if let Some(old) = shared.snapshot.take() {
                    retire(shared, old.data);
                }
                shared.restored_seq = None;
            }
            Action::GiveUp(generation) => {
                tracing::error!(generation, "clipboard restore gave up at its deadline");
                crate::debuglog::note(&format!("clipboard-restore GIVE UP gen={generation}"));
            }
            Action::ClearIfOurs => todo.clear_ours = true,
        }
    }
    todo
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

fn post_worker(message: u32) {
    if let Some(worker) = hwnd_of(&WORKER_HWND) {
        // SAFETY: posting to a window we created.
        let _ = unsafe { PostMessageW(worker, message, WPARAM(0), LPARAM(0)) };
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
/// clipboard holds what it held (or its restore is queued): the caller types.
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
    if let Some(own) = shared.book.current_own() {
        if !own.matches(current) {
            if let Ownership::OursAtNewSeq(now) =
                ownership(Some(own), owner_marker_ours(Some(owner)), current)
            {
                shared.book.set_current_own(OwnSeqs { after_close: now, ..own });
            }
        }
    }
    let decision = decide_paste(
        true,
        shared.book.current_own(),
        shared.snapshot.as_ref().map(|s| s.seq),
        current,
    );
    if decision == PasteDecision::TypeDirectly {
        // A session whose number moved is NOT dropped here: the restore decides.
        return false;
    }
    // A restore is running (its payload is out of the book): never start or
    // continue a session meanwhile, type.
    if shared.book.inflight() {
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
    // Take the snapshot under the lock we hold (it cannot vanish meanwhile).
    let mut started = if decision == PasteDecision::StartSession {
        match shared.snapshot.take() {
            Some(snap) => Some(snap),
            None => return false,
        }
    } else {
        None
    };
    let Some(in_open) = write_temp_text(text) else {
        // Possibly emptied half way. The clipboard is still open: record its
        // sequence number (and the one after the close) as the proof the
        // worker restores against, close, and queue the restore (up to 64 MB,
        // never restored on the hook thread). Then type instead.
        let failed_open = unsafe { GetClipboardSequenceNumber() };
        drop(guard);
        let failed_own =
            OwnSeqs { in_open: failed_open, after_close: unsafe { GetClipboardSequenceNumber() } };
        let actions = match decision {
            PasteDecision::ContinueSession => shared.book.current_to_urgent(failed_own, now_ms()),
            _ => match started.take() {
                Some(snap) => shared.book.urgent_failed_write(snap.data, failed_own, now_ms()).1,
                None => Vec::new(),
            },
        };
        exec_locked(&mut shared, actions);
        drop(shared);
        post_worker(WM_APP_KICK);
        return false;
    };
    // CloseClipboard advances the sequence number (D-WIN-16), so the number that
    // identifies OUR write is only known after the close, and only trusted when
    // owner + marker + a stable bracket say the clipboard is ours.
    drop(guard);
    let post_close = unsafe { GetClipboardSequenceNumber() };
    let own = match verify_write(Some(owner), in_open, post_close) {
        // Unstable keeps the session: user data is never discarded on
        // uncertainty.
        WriteVerdict::Ours(after) | WriteVerdict::Unstable(after) => {
            OwnSeqs { in_open, after_close: after }
        }
        WriteVerdict::Foreign => {
            // Owner/marker prove somebody else wrote: the clipboard is theirs
            // now. Forget the old copy and type.
            let actions = match decision {
                PasteDecision::ContinueSession => shared.book.discard_current(),
                _ => started.take().map(|s| Action::Retire(s.data)).into_iter().collect(),
            };
            exec_locked(&mut shared, actions);
            return false;
        }
    };
    match decision {
        PasteDecision::StartSession => {
            let Some(snap) = started.take() else {
                // Unreachable: the snapshot was taken under the lock we still
                // hold, and a missing one returned before the write.
                debug_assert!(false, "clipboard snapshot vanished after the write");
                tracing::error!("clipboard snapshot vanished after the write");
                return false;
            };
            // A new session supersedes everything older (the book retires it).
            let (_, actions) = shared.book.start_session(snap.data, own, now_ms());
            exec_locked(&mut shared, actions);
        }
        _ => shared.book.continue_session(own, now_ms()),
    }
    shared.text = OurText::of(text);
    drop(shared);

    let inputs = input::build_paste(deletes, mode, chord, held);
    let sent = input::send(&inputs);
    // The settle wait starts when the batch has been SENT (the worker stamps the
    // time on receipt): (re)arm it whatever happened to the batch.
    post_worker(WM_APP_PASTED);
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
    if unsafe { PostMessageW(worker, WM_APP_FLUSH, WPARAM(0), LPARAM(0)) }.is_ok()
        && ack_rx.recv_timeout(Duration::from_millis(800)).is_err()
    {
        tracing::error!("clipboard flush did not finish within 800 ms");
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
            // The batch has been sent: the book restarts the settle wait now.
            let _ = catch_unwind(|| {
                let actions = SHARED.lock().book.paste_sent(now_ms());
                drive(hwnd, actions);
            });
            LRESULT(0)
        }
        WM_APP_KICK => {
            let _ = catch_unwind(|| run_timer(hwnd));
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
            let _ = catch_unwind(|| run_timer(hwnd));
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
            let _ = catch_unwind(|| quit_flush(hwnd));
            if let Some(ack) = FLUSH_ACK.lock().take() {
                let _ = ack.send(());
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

/// Worker: zero and free what the hook retired.
fn collect_garbage() {
    let old = {
        let mut shared = SHARED.lock();
        shared.garbage_bytes = 0;
        std::mem::take(&mut shared.garbage)
    };
    drop(old);
}

/// The restore timer fired (or urgent data is waiting).
fn run_timer(hwnd: HWND) {
    let actions = SHARED.lock().book.timer(now_ms());
    drive(hwnd, actions);
}

/// Execute actions; run the restore attempt they ask for and feed its result
/// back into the book until nothing more is asked.
fn drive(hwnd: HWND, mut actions: Vec<Act>) {
    loop {
        let (todo, owner) = {
            let mut shared = SHARED.lock();
            (exec_locked(&mut shared, actions), hwnd_of(&HOOK_OWNER))
        };
        if todo.stop_timer {
            let _ = unsafe { KillTimer(hwnd, TIMER_SETTLE) };
        }
        if let Some(ms) = todo.arm_ms {
            unsafe { SetTimer(hwnd, TIMER_SETTLE, ms.clamp(1, u64::from(u32::MAX)) as u32, None) };
        }
        if todo.snapshot && ENABLED.load(Ordering::SeqCst) {
            unsafe { SetTimer(hwnd, TIMER_REFRESH, REFRESH_DELAY_MS, None) };
        }
        let text = SHARED.lock().text;
        if todo.clear_ours {
            clear_if_ours(owner, text);
        }
        let Some(job) = todo.restore else { return };
        let result = restore_session(&job, owner, text);
        actions = SHARED.lock().book.restore_result(job, result, now_ms());
    }
}

/// A restore gave up and its user data was zeroed: empty the clipboard when our
/// temporary text is still on it ([`should_clear`]: owner ours AND marker or
/// equal text; or a manager re-set exactly our plain text), so a later Ctrl+V
/// cannot paste it. Bounded: one open try, never retried; a foreign owner is
/// probed BEFORE the clipboard is opened and its text is only read after a size
/// check.
fn clear_if_ours(owner: Option<HWND>, text: OurText) {
    let Some(ours) = owner else { return };
    // May a foreign text be read at all? Decided before the clipboard is opened.
    let may_read_text = text.fp != 0 && clipboard_owner_responsive(50);
    let Some(guard) = ClipboardGuard::open(Some(ours), 1, 0) else { return };
    let owner_is_ours = unsafe { GetClipboardOwner() }.is_ok_and(|now| now == ours);
    let marker = marker_id() != 0 && unsafe { IsClipboardFormatAvailable(marker_id()) }.is_ok();
    // The text is only needed when owner + marker alone do not decide.
    let rescue = (may_read_text && !(owner_is_ours && marker)).then(|| rescue_view(text));
    if should_clear(owner_is_ours, marker, rescue) && unsafe { EmptyClipboard() }.is_ok() {
        tracing::warn!("clipboard emptied: our temporary text was left after a give-up");
    }
    drop(guard);
}

/// Clipboard must be open and the owner probe must have passed. What the
/// (possibly foreign) clipboard holds relative to our text: plain text only
/// (formats listed, no data read) and equal text (size compared before any
/// decoding).
fn rescue_view(text: OurText) -> Rescue {
    let plain_only = list_formats()
        .is_some_and(|ids| plain_text_only(ids.iter().map(|(id, n)| (*id, n.as_deref()))));
    let text_equal = plain_only && clipboard_text_fp(text) == Some(text.fp);
    Rescue { plain_only, text_equal }
}

/// Mode flipped (or the worker just started): listen + first snapshot, or
/// restore once, drop every snapshot and stop.
fn apply_mode(hwnd: HWND) {
    if ENABLED.load(Ordering::SeqCst) {
        SHARED.lock().book.mode_on();
        // SAFETY: our own window.
        let _ = unsafe { AddClipboardFormatListener(hwnd) };
        refresh_snapshot(hwnd);
    } else {
        let _ = unsafe { KillTimer(hwnd, TIMER_REFRESH) };
        let _ = unsafe { RemoveClipboardFormatListener(hwnd) };
        // Restore once (retried for at most MODE_OFF_BUDGET_MS by the book's
        // timer, then zeroed), drop every snapshot, stop the timers.
        let actions = SHARED.lock().book.mode_off(now_ms());
        drive(hwnd, actions);
    }
}

/// Quit: restore against the fixed deadline (urgent data included), polling
/// the book; whatever is still pending when it gives up is zeroed and logged.
fn quit_flush(hwnd: HWND) {
    let _ = unsafe { KillTimer(hwnd, TIMER_REFRESH) };
    let _ = unsafe { RemoveClipboardFormatListener(hwnd) };
    let actions = SHARED.lock().book.quit(now_ms());
    drive(hwnd, actions);
    // The book's deadline bounds this loop (700 ms); the counter is a seatbelt.
    for _ in 0..200 {
        if !SHARED.lock().book.pending() {
            break;
        }
        thread::sleep(Duration::from_millis(QUIT_RETRY_MS));
        let actions = SHARED.lock().book.timer(now_ms());
        drive(hwnd, actions);
    }
    collect_garbage();
    let _ = unsafe { KillTimer(hwnd, TIMER_SETTLE) };
}

/// `WM_CLIPBOARDUPDATE`: the clipboard changed (our own writes included).
fn on_update(hwnd: HWND) {
    if !ENABLED.load(Ordering::SeqCst) {
        let mut shared = SHARED.lock();
        if let Some(old) = shared.snapshot.take() {
            retire(&mut shared, old.data);
        }
        shared.restored_seq = None;
        return;
    }
    let todo = {
        let mut shared = SHARED.lock();
        // Read AFTER taking the lock: the hook records its write's sequence
        // number under this lock, so a paste in flight cannot look foreign.
        let current = unsafe { GetClipboardSequenceNumber() };
        let session_own = shared.book.current_own();
        if notification_is_ours(session_own, shared.restored_seq, current) {
            return; // our temporary text, or the user's data we put back
        }
        // The number is not one we recorded: before dropping anything, ask
        // owner + marker whether it is still our content (late bump).
        if session_own.is_some() || shared.restored_seq.is_some() {
            let proof = owner_marker_ours(hwnd_of(&HOOK_OWNER));
            if let Ownership::OursAtNewSeq(now) = ownership(session_own, proof, current) {
                if let Some(own) = session_own {
                    shared.book.set_current_own(OwnSeqs { after_close: now, ..own });
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
        // While a restore is pending the book answers SkipSnapshot (the restore
        // decides what the new content is); otherwise: drop and re-snapshot.
        let actions = shared.book.notification(false);
        exec_locked(&mut shared, actions)
    };
    // Never open the clipboard right away: a target may be about to open it for
    // a paste (D-WIN-17), and a burst of changes needs one capture.
    if todo.snapshot {
        unsafe { SetTimer(hwnd, TIMER_REFRESH, REFRESH_DELAY_MS, None) };
    }
}

fn refresh_snapshot(hwnd: HWND) {
    // Never open the clipboard while any restore is pending: the target may be
    // opening it for its Ctrl+V right now (D-WIN-17).
    if !SHARED.lock().book.refresh_allowed() {
        return;
    }
    // Probe BEFORE opening: a delayed-render owner that does not answer would
    // block `GetClipboardData` (no timeout), with the clipboard held open.
    // Fail closed: an elevated (higher-integrity) owner we may not message
    // (UIPI) answers 0 too, so a clipboard owned by an elevated program is never
    // snapshotted and the hook types instead.
    let opened = if clipboard_owner_responsive(50) {
        ClipboardGuard::open(Some(hwnd), 5, 20).and_then(|guard| {
            let in_open = unsafe { GetClipboardSequenceNumber() };
            capture(&guard).map(|data| (in_open, data))
        })
    } else {
        None
    };
    // The guard is closed here. The snapshot's number is read in a bracket with
    // no clipboard open and accepted only if it did not move (a foreign write in
    // the gap would make the number newer than the data: reject).
    let snap = opened.and_then(|(in_open, data)| {
        let seq = stable_seq()?;
        log_capture_bump(in_open, seq);
        Some(Snap { seq, data })
    });
    let mut shared = SHARED.lock();
    if shared.book.refresh_allowed() && ENABLED.load(Ordering::SeqCst) {
        if let Some(old) = std::mem::replace(&mut shared.snapshot, snap) {
            retire(&mut shared, old.data);
        }
    }
}

/// One restore attempt, retried while the clipboard is busy. Proof of ownership
/// (sequence / owner / marker) is checked while the clipboard is open. The
/// restored data keeps the privacy flags on purpose: the user's original copy is
/// already in Win+V / cloud history, re-adding it would duplicate it.
///
/// Narrow text rescue: a clipboard manager / cloud sync / RDP that re-sets our
/// text without the marker would make the content look foreign and lose the
/// user's original clipboard (possibly an image). If the clipboard is plain text
/// only, this is round 0, and the text equals what we wrote, the restore goes
/// ahead (`keys.log`: `clipboard-text-rescue fired`). The price: a user copy of
/// exactly the same plain text within the settle window gets their previous
/// clipboard back instead.
fn restore_session(
    job: &RestoreJob<Vec<FormatData>>,
    owner: Option<HWND>,
    text: OurText,
) -> RestoreResult {
    // May a FOREIGN-owned clipboard's text be read for the rescue? The owner
    // probe runs BEFORE the clipboard is opened (a hung delayed-render owner
    // would block `GetClipboardData` with the clipboard locked system-wide).
    let may_read_text =
        !job.urgent && job.rounds == 0 && text.fp != 0 && clipboard_owner_responsive(50);
    let started = Instant::now();
    let mut attempt = 0u32;
    loop {
        // `open_owner` only decides which handle OpenClipboard gets; the proof
        // checks (GetClipboardOwner / marker) keep using the hook owner, they do
        // not depend on the handle that opened the clipboard.
        let open_owner = if uses_hook_owner(attempt) { owner } else { None };
        if let Some(guard) = ClipboardGuard::open(open_owner, 1, 0) {
            // Checked while we hold the clipboard: nobody can write in between.
            let now = unsafe { GetClipboardSequenceNumber() };
            let owner_is_ours = owner
                .is_some_and(|ours| unsafe { GetClipboardOwner() }.is_ok_and(|now| now == ours));
            let owner_marker = owner_marker_ours(owner);
            // The rescue view is only built when the base proof fails and the
            // probe allowed reading the text.
            let rescue = (may_read_text
                && restore_action(job.own, owner_marker, now) == RestoreAction::Abandon)
                .then(|| rescue_view(text));
            let ours = restore_decision(
                job.urgent,
                job.own,
                job.rounds,
                owner_marker,
                owner_is_ours,
                now,
                rescue,
            ) == RestoreAction::Restore;
            if ours && !job.urgent && rescue.is_some_and(|r| r.text_equal) {
                tracing::debug!("clipboard text equals ours without the marker; restoring");
                crate::debuglog::note("clipboard-text-rescue fired");
            }
            if !ours {
                tracing::debug!("clipboard changed after our paste; not restoring");
                return RestoreResult::Foreign;
            }
            let complete = restore_locked(&job.payload);
            let in_open = unsafe { GetClipboardSequenceNumber() };
            drop(guard);
            // The number AFTER the close (CloseClipboard advances it): every
            // later comparison, including the NULL-owner one, uses this one.
            let post_close = unsafe { GetClipboardSequenceNumber() };
            if !complete {
                tracing::warn!("clipboard restore incomplete; will retry");
                // Record the post-write numbers: the next round proves ownership
                // against them (also on the NULL-owner path, which has no window).
                return RestoreResult::Incomplete {
                    own: Some(OwnSeqs { in_open, after_close: post_close }),
                };
            }
            // An empty user clipboard stays empty: no marker, nothing to adopt.
            if job.payload.is_empty() {
                return RestoreResult::Done { adopt: None };
            }
            // Adopt only what owner + marker + a stable bracket prove is our
            // restored data (never trust a bare number, D-WIN-16).
            let adopt = match verify_write(open_owner, in_open, post_close) {
                WriteVerdict::Ours(after) | WriteVerdict::Unstable(after) => {
                    Some(OwnSeqs { in_open, after_close: after })
                }
                WriteVerdict::Foreign => None,
            };
            return RestoreResult::Done { adopt };
        }
        // Normal restores back off; a flush job never sleeps past its budget
        // (deadline - now) and makes a single try when the budget is tiny.
        let elapsed = started.elapsed().as_millis() as u64;
        match open_retry_delay(attempt, elapsed, job.max_wait_ms) {
            Some(delay) => {
                thread::sleep(Duration::from_millis(delay));
                attempt += 1;
            }
            None => break,
        }
    }
    tracing::warn!("clipboard busy; restore will be retried");
    RestoreResult::Busy
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
    // (The owner-responsiveness probe ran BEFORE the clipboard was opened, see
    // `refresh_snapshot`: a delayed-render owner that does not answer would
    // block `GetClipboardData`, which has no timeout.)
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

/// Clipboard must be open and the owner probe passed. Fingerprint of the
/// current `CF_UNICODETEXT`, but only when its size is exactly that of our text:
/// anything else cannot be ours and is never decoded.
fn clipboard_text_fp(text: OurText) -> Option<u64> {
    let handle = unsafe { GetClipboardData(CF_UNICODETEXT) }.ok().filter(|h| !h.is_invalid())?;
    let global = HGLOBAL(handle.0);
    let size = unsafe { GlobalSize(global) };
    if size != our_text_bytes(text.units) {
        return None;
    }
    let ptr = unsafe { GlobalLock(global) } as *const u16;
    if ptr.is_null() {
        return None;
    }
    // SAFETY: `size` bytes, checked against the length of our own text.
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
/// fallback) additionally requires the number to equal `post_close`, the one
/// read right after the close (NOT the in-open one: the close advances it).
fn verify_write(expected_owner: Option<HWND>, in_open: u32, post_close: u32) -> WriteVerdict {
    let marker = marker_id();
    let mut last = WriteVerdict::Unstable(0);
    for attempt in 0..3 {
        let s1 = unsafe { GetClipboardSequenceNumber() };
        let owner_check = match expected_owner {
            Some(ours) => OwnerCheck::Window {
                matches: unsafe { GetClipboardOwner() }.is_ok_and(|now| now == ours),
            },
            // NULL owner: no window to compare, the number must match too.
            None => OwnerCheck::Null,
        };
        let marker_present = marker != 0 && unsafe { IsClipboardFormatAvailable(marker) }.is_ok();
        let s2 = unsafe { GetClipboardSequenceNumber() };
        last = write_verdict(owner_check, marker_present, in_open, post_close, s1, s2);
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
