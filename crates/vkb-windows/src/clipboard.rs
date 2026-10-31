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
    classify_format, decide_paste, format_count_ok, restore_action, settle_remaining_ms, snapshot_verdict,
    within_budget, FormatClass, PasteDecision, RestoreAction, Verdict, PRIVACY_FORMATS,
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
    OpenClipboard, RegisterClipboardFormatW, RemoveClipboardFormatListener, SetClipboardData,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, IsHungAppWindow,
    KillTimer, PostMessageW, RegisterClassW, SetTimer, TranslateMessage, HWND_MESSAGE, MSG,
    WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSW,
};

use crate::input::{self, Held, SendMode};
use crate::{globals, keys};

/// `CF_UNICODETEXT`.
pub const CF_UNICODETEXT: u32 = 13;

const WM_TIMER: u32 = 0x0113;
const WM_CLIPBOARDUPDATE: u32 = 0x031D;
const WM_APP_PASTED: u32 = 0x8000 + 0x21;
const WM_APP_MODE: u32 = 0x8000 + 0x22;
const WM_APP_FLUSH: u32 = 0x8000 + 0x23;
const TIMER_SETTLE: usize = 1;

#[derive(Debug, PartialEq, Eq)]
struct FormatData {
    format: u32,
    bytes: Vec<u8>,
}

/// A faithful copy of the clipboard as of sequence number `seq`.
struct Snap {
    seq: u32,
    data: Vec<FormatData>,
}

/// The user's clipboard while our temporary text is on it.
struct Session {
    data: Vec<FormatData>,
    /// Sequence number right after our last write.
    expected_seq: u32,
    /// When we last wrote (the settle wait counts from here).
    last_write: Instant,
}

#[derive(Default)]
struct Shared {
    snapshot: Option<Snap>,
    session: Option<Session>,
}

static SHARED: Mutex<Shared> = Mutex::new(Shared { snapshot: None, session: None });
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
pub fn replace(deletes: u32, text: &str, mode: SendMode, held: Held) {
    let pasted = catch_unwind(AssertUnwindSafe(|| try_paste(deletes, text, mode, held)))
        .unwrap_or(false);
    if !pasted {
        type_directly(deletes, text, mode, held);
    }
}

/// Unicode `SendInput` delivery (the safe fallback).
fn type_directly(deletes: u32, text: &str, mode: SendMode, held: Held) {
    let inputs = input::build_segment_held(deletes, text, mode, held);
    input::send_split(&inputs, deletes, mode);
}

/// The Shift keys that are down right now.
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

/// `true` = the replacement was delivered. `false` = nothing was sent and the
/// clipboard holds what it held (or was restored): the caller types instead.
fn try_paste(deletes: u32, text: &str, mode: SendMode, held: Held) -> bool {
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
    let current = unsafe { GetClipboardSequenceNumber() };
    let decision = decide_paste(
        true,
        shared.session.as_ref().map(|s| s.expected_seq),
        shared.snapshot.as_ref().map(|s| s.seq),
        current,
    );
    if decision == PasteDecision::TypeDirectly {
        // A session whose clipboard moved on is the user's data now: forget it.
        if shared.session.as_ref().is_some_and(|s| s.expected_seq != current) {
            shared.session = None;
        }
        return false;
    }
    // A hung previous owner would block EmptyClipboard (WM_DESTROYCLIPBOARD).
    if let Ok(prev) = unsafe { GetClipboardOwner() } {
        if !prev.is_invalid() && unsafe { IsHungAppWindow(prev) }.as_bool() {
            return false;
        }
    }
    // Single try: the hook never retries.
    let Some(guard) = ClipboardGuard::open(Some(owner), 1, 0) else { return false };
    // Nobody can write while we hold it: the decision must still hold.
    if unsafe { GetClipboardSequenceNumber() } != current {
        return false;
    }
    let Some(seq) = write_temp_text(text) else {
        // Possibly emptied half way: put the user's data back right now.
        let data = match decision {
            PasteDecision::ContinueSession => shared.session.take().map(|s| s.data),
            _ => shared.snapshot.take().map(|s| s.data),
        };
        if let Some(data) = data {
            restore_locked(&data);
        }
        return false;
    };
    drop(guard);
    match decision {
        PasteDecision::StartSession => {
            if let Some(snap) = shared.snapshot.take() {
                shared.session = Some(Session { data: snap.data, expected_seq: seq, last_write: Instant::now() });
            }
        }
        _ => {
            if let Some(session) = shared.session.as_mut() {
                session.expected_seq = seq;
                session.last_write = Instant::now();
            }
        }
    }
    drop(shared);

    let live = Held { shift: live_shift(), caps: held.caps };
    let inputs = input::build_paste(deletes, mode, live);
    let sent = input::send(&inputs);
    if let Some(worker) = hwnd_of(&WORKER_HWND) {
        // (Re)arm the restore timer, whatever happened to the batch.
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
            // Restart the settle wait; the restore runs when it elapses.
            unsafe { SetTimer(hwnd, TIMER_SETTLE, SETTLE_MS as u32, None) };
            LRESULT(0)
        }
        WM_TIMER if wparam.0 == TIMER_SETTLE => {
            let _ = unsafe { KillTimer(hwnd, TIMER_SETTLE) };
            let _ = catch_unwind(|| restore_due(hwnd));
            LRESULT(0)
        }
        WM_APP_MODE => {
            let _ = catch_unwind(|| apply_mode(hwnd));
            LRESULT(0)
        }
        WM_APP_FLUSH => {
            let _ = unsafe { KillTimer(hwnd, TIMER_SETTLE) };
            let _ = catch_unwind(|| {
                restore_now();
                clear_snapshot();
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

fn clear_snapshot() {
    SHARED.lock().snapshot = None;
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
        let _ = unsafe { RemoveClipboardFormatListener(hwnd) };
        restore_now();
        clear_snapshot();
    }
}

/// `WM_CLIPBOARDUPDATE`: the clipboard changed (our own writes included).
fn on_update(hwnd: HWND) {
    if !ENABLED.load(Ordering::SeqCst) {
        clear_snapshot();
        return;
    }
    let current = unsafe { GetClipboardSequenceNumber() };
    {
        let mut shared = SHARED.lock();
        if let Some(session) = shared.session.as_ref() {
            if session.expected_seq == current {
                return; // our own temporary text
            }
            // Someone else copied: their data, never restored over.
            shared.session = None;
            let _ = unsafe { KillTimer(hwnd, TIMER_SETTLE) };
        }
        shared.snapshot = None; // stale until the new one is ready
    }
    refresh_snapshot(hwnd);
}

fn refresh_snapshot(hwnd: HWND) {
    let snap = ClipboardGuard::open(Some(hwnd), 5, 20).and_then(|guard| {
        let seq = unsafe { GetClipboardSequenceNumber() };
        let data = capture(&guard)?;
        Some(Snap { seq, data })
    });
    let mut shared = SHARED.lock();
    if shared.session.is_none() && ENABLED.load(Ordering::SeqCst) {
        shared.snapshot = snap;
    }
}

/// Settle timer: restore only when the last write is old enough; a paste that
/// raced the timer re-arms it instead (the target may not have read yet).
fn restore_due(hwnd: HWND) {
    let wait = {
        let shared = SHARED.lock();
        let Some(session) = shared.session.as_ref() else { return };
        settle_remaining_ms(session.last_write.elapsed().as_millis() as u64)
    };
    if wait > 0 {
        unsafe { SetTimer(hwnd, TIMER_SETTLE, wait as u32, None) };
    } else {
        restore_now();
    }
}

/// Flush / mode off / settled: put the user's data back.
fn restore_now() {
    let Some(done) = SHARED.lock().session.take() else { return };
    restore_session(&done);
}

/// Retried with backoff while the clipboard is busy; only if still ours.
fn restore_session(done: &Session) {
    let owner = hwnd_of(&HOOK_OWNER);
    for attempt in 0..5u32 {
        if let Some(guard) = ClipboardGuard::open(owner, 1, 0) {
            // Checked while we hold the clipboard: nobody can write in between.
            let now = unsafe { GetClipboardSequenceNumber() };
            if restore_action(done.expected_seq, now) == RestoreAction::Abandon {
                tracing::debug!("clipboard changed after our paste; not restoring");
                return;
            }
            if !restore_locked(&done.data) {
                tracing::warn!("clipboard restore incomplete");
            }
            drop(guard);
            return;
        }
        thread::sleep(Duration::from_millis(20 << attempt));
    }
    tracing::warn!("clipboard busy; previous contents could not be restored");
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
        tracing::debug!("clipboard holds formats that cannot be restored; no snapshot");
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
        assert_eq!(
            restore_action(seq, unsafe { GetClipboardSequenceNumber() }),
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
