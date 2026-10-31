//! Opt-in clipboard send mode (config `typing.use_clipboard`).
//!
//! **Never** on the default path: charter C4 forbids it, and the fast
//! `SendInput` path covers every app we ship for. This exists only for the
//! stubborn targets where Unicode injection is ignored.
//!
//! Design (D-WIN-12..15, `70-impl-notes-windows.md` "Clipboard send mode"):
//!
//! - **One worker thread, one ordered queue.** The hook only enqueues a [`Job`];
//!   every clipboard call (which can retry or block on a delayed-render owner)
//!   runs on `vkb-clipboard`. Two quick expansions can no longer snapshot each
//!   other's temporary text (the old "only `ê` left" loss).
//! - **Snapshot first, and only if it is faithful.** All formats are listed and
//!   classified before any data is read ([`vkb_platform::clipboard_plan`]).
//!   `HGLOBAL` formats are copied byte for byte; GDI handle formats are never
//!   dereferenced as memory (that was the bitmap crash) and are rebuilt by
//!   Windows from the DIB. If anything would be lost, the text is typed with
//!   Unicode `SendInput` instead and the clipboard is left alone.
//! - **No paste unless our text is on the clipboard.** `Ctrl+V` is sent only
//!   after `SetClipboardData` succeeded, in the same `SendInput` call as the
//!   Backspaces.
//! - **Restore late and only if still ours.** The snapshot goes back after the
//!   queue has been idle for [`SETTLE_MS`], and only when
//!   `GetClipboardSequenceNumber` still equals the value right after our write.
//! - **Privacy.** Our temporary text carries the history/cloud/monitor opt-out
//!   formats so it never reaches Win+V or cloud sync.
//!
//! The release profile uses `panic = "abort"`: `catch_unwind` below protects
//! dev/test builds, the release guarantee comes from checking every call.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

use once_cell::sync::OnceCell;
use vkb_platform::clipboard_plan::{
    classify_format, restore_action, snapshot_verdict, within_budget, FormatClass, RestoreAction,
    Verdict, PRIVACY_FORMATS, SETTLE_MS,
};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData,
    GetClipboardFormatNameW, GetClipboardOwner, GetClipboardSequenceNumber, OpenClipboard,
    RegisterClipboardFormatW, SetClipboardData,
};
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
};
use windows::Win32::UI::WindowsAndMessaging::IsHungAppWindow;

use crate::input::{self, Held, SendMode};

/// `CF_UNICODETEXT`.
pub const CF_UNICODETEXT: u32 = 13;

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
}

/// The user's clipboard while our temporary text may be on it.
struct Session {
    snapshot: Vec<FormatData>,
    /// Sequence number right after our last write.
    expected_seq: u32,
}

static QUEUE: OnceCell<Sender<Job>> = OnceCell::new();

/// Replace `deletes` characters with `text` through the clipboard.
///
/// Never blocks: the hook thread only enqueues. If the worker cannot be started
/// the text is typed directly.
pub fn submit(deletes: u32, text: String, mode: SendMode, held: Held) {
    let job = Job { deletes, text, mode, held };
    let sender = QUEUE.get_or_try_init(|| {
        let (tx, rx) = channel::<Job>();
        thread::Builder::new()
            .name("vkb-clipboard".to_string())
            .spawn(move || worker(rx))
            .map(|_| tx)
    });
    match sender {
        Ok(tx) => {
            if let Err(err) = tx.send(job) {
                type_directly(&err.0);
            }
        }
        Err(_) => type_directly(&job),
    }
}

fn worker(rx: std::sync::mpsc::Receiver<Job>) {
    let mut session: Option<Session> = None;
    loop {
        let next = if session.is_some() {
            rx.recv_timeout(Duration::from_millis(SETTLE_MS))
        } else {
            rx.recv().map_err(|_| RecvTimeoutError::Disconnected)
        };
        match next {
            Ok(job) => {
                let outcome = catch_unwind(AssertUnwindSafe(|| run_job(&job, &mut session)));
                if outcome.is_err() {
                    tracing::error!("clipboard job panicked; dropping snapshot, typing directly");
                    session = None;
                    type_directly(&job);
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                if let Some(done) = session.take() {
                    let _ = catch_unwind(AssertUnwindSafe(|| finish(done)));
                }
            }
            Err(RecvTimeoutError::Disconnected) => {
                if let Some(done) = session.take() {
                    let _ = catch_unwind(AssertUnwindSafe(|| finish(done)));
                }
                return;
            }
        }
    }
}

/// Unicode `SendInput` delivery (the safe fallback).
fn type_directly(job: &Job) {
    let inputs = input::build_segment_held(job.deletes, &job.text, job.mode, job.held);
    input::send_split(&inputs, job.deletes, job.mode);
}

fn run_job(job: &Job, session: &mut Option<Session>) {
    // Someone else wrote to the clipboard since our last write: that content is
    // the user's now. Forget the old snapshot and start from what is there.
    if let Some(current) = session.as_ref() {
        let now = unsafe { GetClipboardSequenceNumber() };
        if restore_action(current.expected_seq, now) == RestoreAction::Abandon {
            tracing::debug!("clipboard changed by another program; old snapshot dropped");
            *session = None;
        }
    }
    let fresh = session.is_none();
    if fresh {
        match capture() {
            Some(snapshot) => *session = Some(Session { snapshot, expected_seq: 0 }),
            None => {
                type_directly(job);
                return;
            }
        }
    }
    let Some(active) = session.as_mut() else {
        type_directly(job);
        return;
    };
    match write_temp_text(&job.text) {
        Some(seq) => active.expected_seq = seq,
        None => {
            // The clipboard may have been emptied half way: put it back now.
            if let Some(done) = session.take() {
                restore_snapshot(&done.snapshot, None);
            }
            type_directly(job);
            return;
        }
    }
    let inputs = input::build_paste(job.deletes, job.mode, job.held);
    let sent = input::send(&inputs);
    if (sent as usize) < inputs.len() {
        tracing::warn!(sent, total = inputs.len(), "clipboard paste batch blocked");
    }
}

fn finish(done: Session) {
    restore_snapshot(&done.snapshot, Some(done.expected_seq));
}

/// Put `snapshot` back. With `Some(seq)` only when the clipboard is still the
/// state right after our write; `None` restores unconditionally (our own failed
/// write).
fn restore_snapshot(snapshot: &[FormatData], expected: Option<u32>) {
    let Some(guard) = ClipboardGuard::open() else {
        tracing::warn!("clipboard busy; previous contents could not be restored");
        return;
    };
    if let Some(expected) = expected {
        let now = unsafe { GetClipboardSequenceNumber() };
        if restore_action(expected, now) == RestoreAction::Abandon {
            tracing::debug!("clipboard changed after our paste; not restoring");
            return;
        }
    }
    if unsafe { EmptyClipboard() }.is_err() {
        tracing::warn!("EmptyClipboard failed during restore");
        return;
    }
    for entry in snapshot {
        if !set_bytes(entry.format, &entry.bytes) {
            tracing::warn!(format = entry.format, "clipboard format not restored");
        }
    }
    // The restored copy is not a new user action: keep it out of history/cloud.
    set_privacy_flags();
    drop(guard);
}

/// An open clipboard, closed on drop (also on unwind).
struct ClipboardGuard;

impl ClipboardGuard {
    fn open() -> Option<Self> {
        for _ in 0..10 {
            if unsafe { OpenClipboard(None) }.is_ok() {
                return Some(Self);
            }
            thread::sleep(Duration::from_millis(20));
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
    if id < vkb_platform::clipboard_plan::REGISTERED_FIRST {
        return None;
    }
    let mut buf = [0u16; 128];
    let n = unsafe { GetClipboardFormatNameW(id, &mut buf) };
    if n <= 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&buf[..n as usize]))
}

/// Snapshot the clipboard, or `None` when a faithful round trip is not
/// possible (busy, unknown/handle-only formats, unreadable data, too large,
/// hung owner). `None` means: do not touch the clipboard.
fn capture() -> Option<Vec<FormatData>> {
    let guard = ClipboardGuard::open()?;
    // A delayed-render owner that hangs would block `GetClipboardData`.
    if let Ok(owner) = unsafe { GetClipboardOwner() } {
        if !owner.is_invalid() && unsafe { IsHungAppWindow(owner) }.as_bool() {
            return None;
        }
    }
    // Pass 1: ids only. No data is read (or rendered) before the verdict.
    let mut ids = Vec::new();
    let mut current = 0u32;
    loop {
        current = unsafe { EnumClipboardFormats(current) };
        if current == 0 || ids.len() > 512 {
            break;
        }
        ids.push((current, format_name(current)));
    }
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
    drop(guard);
    Some(formats)
}

/// Replace the clipboard with `text` (history/cloud opt-out flags included).
/// Returns the sequence number right after the write, `None` on any failure.
fn write_temp_text(text: &str) -> Option<u32> {
    let mut units: Vec<u16> = text.encode_utf16().collect();
    units.push(0);
    let bytes: Vec<u8> = units.iter().flat_map(|u| u.to_le_bytes()).collect();
    {
        let _guard = ClipboardGuard::open()?;
        unsafe { EmptyClipboard() }.ok()?;
        if !set_bytes(CF_UNICODETEXT, &bytes) {
            return None;
        }
        set_privacy_flags();
    }
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
    let _guard = ClipboardGuard::open()?;
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
        write_temp_text("tạm thời").expect("write");
        assert_eq!(read_text().as_deref(), Some("tạm thời"));
        let backup = capture().expect("capture");
        let seq = write_temp_text("khác").expect("write");
        restore_snapshot(&backup, Some(seq));
        assert_eq!(read_text().as_deref(), Some("tạm thời"));
        // Put the user's clipboard back.
        match previous {
            Some(text) => {
                write_temp_text(&text).expect("restore original");
            }
            None => {
                if let Some(_guard) = ClipboardGuard::open() {
                    let _ = unsafe { EmptyClipboard() };
                }
            }
        }
    }
}
