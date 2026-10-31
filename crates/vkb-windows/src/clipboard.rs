//! Opt-in clipboard send mode (config `typing.use_clipboard`).
//!
//! **Never** on the default path: charter C4 forbids it, and the fast
//! `SendInput` path covers every app we ship for. This exists only for the
//! stubborn targets where Unicode injection is ignored.
//!
//! Design (D-WIN-12..15, `70-impl-notes-windows.md` section 9):
//!
//! - **One worker thread, one ordered queue.** The hook only enqueues a [`Job`];
//!   every clipboard call (which can retry or block on a delayed-render owner)
//!   runs on `vkb-clipboard`. Consecutive expansions share one snapshot.
//! - **Ordering.** Backspaces now go out from the worker, so a physical key
//!   could overtake them. [`submit`] raises [`PENDING`] and the worker lowers it
//!   (drop guard, every path) once the batch is sent; the hook calls
//!   [`wait_pending`] before processing a physical key (bounded by
//!   [`WAIT_CAP`]). Opening the clipboard fails fast (3 x 10 ms) so the worker
//!   normally finishes well inside the cap; on timeout the key is processed
//!   anyway and the miss is logged.
//! - **Snapshot first, and only if it is faithful** (`vkb_platform::clipboard_plan`).
//!   Capture, sequence check and our write happen under ONE `OpenClipboard`.
//!   GDI handle formats are never dereferenced; a clipboard that cannot be
//!   restored makes us type the text with Unicode `SendInput` instead.
//! - **No paste unless our text is on the clipboard.** One `SendInput`: the
//!   Backspaces and the `Ctrl+V` chord, Shift re-read from the live key state.
//! - **Restore late and only if still ours:** after `SETTLE_MS` of queue
//!   idleness (fixed on purpose: there is no consumed-paste signal, and a
//!   longer wait only delays the restore), only if the sequence number is
//!   unchanged, retried with backoff while the clipboard is busy.
//! - **Shutdown.** [`shutdown`] restores immediately (bounded wait).
//! - **Privacy.** Our temporary text carries the history/cloud/monitor opt-out
//!   formats.
//!
//! The release profile uses `panic = "abort"`: `catch_unwind` below protects
//! dev/test builds, the release guarantee comes from checking every call.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

use once_cell::sync::OnceCell;
use vkb_platform::clipboard_plan::{
    classify_format, restore_action, snapshot_verdict, within_budget, FormatClass, PendingGate,
    PendingToken, RestoreAction, Verdict, MAX_FORMATS, PRIVACY_FORMATS, REGISTERED_FIRST,
    SETTLE_MS,
};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{
    GetLastError, GlobalFree, SetLastError, HANDLE, HGLOBAL, HWND, WIN32_ERROR,
};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData,
    GetClipboardFormatNameW, GetClipboardOwner, GetClipboardSequenceNumber, OpenClipboard,
    RegisterClipboardFormatW, SetClipboardData,
};
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
};
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, IsHungAppWindow, HWND_MESSAGE, WINDOW_EX_STYLE, WINDOW_STYLE,
};

use crate::input::{self, Held, SendMode};
use crate::keys;

/// `CF_UNICODETEXT`.
pub const CF_UNICODETEXT: u32 = 13;

/// How long a physical key waits for a queued clipboard job.
const WAIT_CAP: Duration = Duration::from_millis(150);

/// Jobs queued or being sent (not counting the settle wait before the restore).
static PENDING: PendingGate = PendingGate::new();

#[derive(Debug, PartialEq, Eq)]
struct FormatData {
    format: u32,
    bytes: Vec<u8>,
}

/// One replacement: `deletes` Backspaces, then `text`.
struct Job {
    deletes: u32,
    text: String,
    mode: SendMode,
    held: Held,
    /// Released (dropped) as soon as the input is sent, or with the job.
    token: Option<PendingToken<'static>>,
}

enum Msg {
    Job(Job),
    /// Restore now and acknowledge (quit).
    Flush(Sender<()>),
}

/// The user's clipboard while our temporary text may be on it.
struct Session {
    snapshot: Vec<FormatData>,
    /// Sequence number right after our last write.
    expected_seq: u32,
}

static QUEUE: OnceCell<Sender<Msg>> = OnceCell::new();

/// Replace `deletes` characters with `text` through the clipboard.
///
/// Never blocks: the hook thread only enqueues. If the worker cannot be started
/// the text is typed directly.
pub fn submit(deletes: u32, text: String, mode: SendMode, held: Held) {
    let job = Job { deletes, text, mode, held, token: Some(PENDING.enter()) };
    let sender = QUEUE.get_or_try_init(|| {
        let (tx, rx) = channel::<Msg>();
        thread::Builder::new()
            .name("vkb-clipboard".to_string())
            .spawn(move || worker(rx))
            .map(|_| tx)
    });
    match sender {
        Ok(tx) => {
            if let Err(err) = tx.send(Msg::Job(job)) {
                if let Msg::Job(job) = err.0 {
                    type_directly(&job);
                }
            }
        }
        Err(_) => type_directly(&job),
    }
}

/// Top of every hook event: a physical key waits (bounded) while a clipboard
/// job has not sent its input yet, so it cannot overtake the replacement.
#[inline]
pub fn wait_pending(extra_info: usize) {
    if PENDING.pending() == 0 || extra_info == input::injected_tag() {
        return;
    }
    if !PENDING.wait(WAIT_CAP, Duration::from_micros(500)) {
        tracing::warn!("clipboard job still pending after the wait cap; processing the key anyway");
        crate::debuglog::note("clipboard-wait TIMEOUT; processing the key anyway");
    }
}

/// Put the user's clipboard back now (app quit, hook uninstall). Bounded.
pub fn shutdown() {
    let Some(tx) = QUEUE.get() else { return };
    let (ack_tx, ack_rx) = channel();
    if tx.send(Msg::Flush(ack_tx)).is_ok() {
        let _ = ack_rx.recv_timeout(Duration::from_millis(800));
    }
}

/// Owner window and friends, used from the worker only.
struct Ctx {
    /// Message-only window: a real clipboard owner (`OpenClipboard(NULL)` makes
    /// `SetClipboardData` unreliable). `None` falls back to the NULL owner.
    owner: Option<HWND>,
}

fn worker(rx: Receiver<Msg>) {
    // SAFETY: a message-only window of the system "STATIC" class, owned and
    // destroyed by this thread.
    let owner = unsafe {
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
    }
    .ok();
    let ctx = Ctx { owner };
    let mut session: Option<Session> = None;
    loop {
        let next = if session.is_some() {
            rx.recv_timeout(Duration::from_millis(SETTLE_MS))
        } else {
            rx.recv().map_err(|_| RecvTimeoutError::Disconnected)
        };
        match next {
            Ok(Msg::Job(mut job)) => {
                let outcome =
                    catch_unwind(AssertUnwindSafe(|| run_job(&mut job, &mut session, &ctx)));
                if outcome.is_err() {
                    tracing::error!("clipboard job panicked; dropping snapshot, typing directly");
                    session = None;
                    type_directly(&job);
                }
                // `job` (and its token) drops here on every path.
            }
            Ok(Msg::Flush(ack)) => {
                flush(&mut session, &ctx);
                let _ = ack.send(());
            }
            Err(RecvTimeoutError::Timeout) => flush(&mut session, &ctx),
            Err(RecvTimeoutError::Disconnected) => {
                flush(&mut session, &ctx);
                break;
            }
        }
    }
    if let Some(window) = owner {
        // SAFETY: window created above on this thread.
        let _ = unsafe { DestroyWindow(window) };
    }
}

fn flush(session: &mut Option<Session>, ctx: &Ctx) {
    if let Some(done) = session.take() {
        let _ = catch_unwind(AssertUnwindSafe(|| restore_session(&done, ctx)));
    }
}

/// Unicode `SendInput` delivery (the safe fallback).
fn type_directly(job: &Job) {
    let inputs = input::build_segment_held(job.deletes, &job.text, job.mode, job.held);
    input::send_split(&inputs, job.deletes, job.mode);
}

/// The Shift keys that are down right now (the worker runs a few ms after the
/// hook saw the key, so the hook-time model can be stale).
fn live_shift() -> u8 {
    let down = |vk: u16| unsafe { GetAsyncKeyState(i32::from(vk)) } < 0;
    let mut mask = 0;
    if down(keys::VK_LSHIFT) {
        mask |= keys::SHIFT_L;
    }
    if down(keys::VK_RSHIFT) {
        mask |= keys::SHIFT_R;
    }
    mask
}

fn run_job(job: &mut Job, session: &mut Option<Session>, ctx: &Ctx) {
    let prepared = match ClipboardGuard::open(ctx.owner, 3, 10) {
        Some(guard) => prepare(&guard, &job.text, session),
        None => false,
    };
    if !prepared {
        type_directly(job);
        job.token = None;
        return;
    }
    let held = Held { shift: live_shift(), caps: job.held.caps };
    let inputs = input::build_paste(job.deletes, job.mode, held);
    let sent = input::send(&inputs);
    // Input is in the queue of the target: later physical keys may proceed.
    job.token = None;
    if (sent as usize) < inputs.len() {
        tracing::warn!(sent, total = inputs.len(), "clipboard paste batch blocked");
    }
}

/// Under one open clipboard: keep or take the snapshot, then write our text.
/// `false` = the clipboard was not changed for good; type directly.
fn prepare(guard: &ClipboardGuard, text: &str, session: &mut Option<Session>) -> bool {
    // Someone else wrote since our last write: that content is the user's now.
    if let Some(current) = session.as_ref() {
        let now = unsafe { GetClipboardSequenceNumber() };
        if restore_action(current.expected_seq, now) == RestoreAction::Abandon {
            tracing::debug!("clipboard changed by another program; old snapshot dropped");
            *session = None;
        }
    }
    if session.is_none() {
        match capture(guard) {
            Some(snapshot) => *session = Some(Session { snapshot, expected_seq: 0 }),
            None => return false,
        }
    }
    let Some(active) = session.as_mut() else { return false };
    match write_temp_text(text) {
        Some(seq) => {
            active.expected_seq = seq;
            true
        }
        None => {
            // Possibly emptied half way: put the user's data back right now.
            if let Some(done) = session.take() {
                restore_locked(&done.snapshot);
            }
            false
        }
    }
}

/// Restore after the settle wait; retried with backoff while the clipboard is busy.
fn restore_session(done: &Session, ctx: &Ctx) {
    for attempt in 0..5u32 {
        if let Some(guard) = ClipboardGuard::open(ctx.owner, 1, 0) {
            // Checked while we hold the clipboard: nobody can write in between.
            let now = unsafe { GetClipboardSequenceNumber() };
            if restore_action(done.expected_seq, now) == RestoreAction::Abandon {
                tracing::debug!("clipboard changed after our paste; not restoring");
                return;
            }
            if !restore_locked(&done.snapshot) {
                tracing::warn!("clipboard restore incomplete");
            }
            drop(guard);
            return;
        }
        thread::sleep(Duration::from_millis(20 << attempt));
    }
    tracing::warn!("clipboard busy; previous contents could not be restored");
}

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
/// failed or there are more than [`MAX_FORMATS`] (not safely restorable).
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
        if ids.len() >= MAX_FORMATS {
            return None;
        }
        ids.push((current, format_name(current)));
    }
}

/// Snapshot the open clipboard, or `None` when a faithful round trip is not
/// possible (unknown/handle-only formats, unreadable data, too large, hung
/// owner). `None` means: do not touch the clipboard.
fn capture(_open: &ClipboardGuard) -> Option<Vec<FormatData>> {
    // A delayed-render owner that hangs would block `GetClipboardData`.
    if let Ok(owner) = unsafe { GetClipboardOwner() } {
        if !owner.is_invalid() && unsafe { IsHungAppWindow(owner) }.as_bool() {
            return None;
        }
    }
    // Pass 1: ids only. No data is read (or rendered) before the verdict.
    let ids = list_formats()?;
    let verdict = snapshot_verdict(ids.iter().map(|(id, name)| (*id, name.as_deref())));
    if verdict == Verdict::Lossy {
        tracing::debug!("clipboard holds formats that cannot be restored; typing instead");
        return None;
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
    let seq = unsafe { GetClipboardSequenceNumber() };
    (seq != 0).then_some(seq)
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
        assert_eq!(restore_action(seq, unsafe { GetClipboardSequenceNumber() }), RestoreAction::Restore);
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
