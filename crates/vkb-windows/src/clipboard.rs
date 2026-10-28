//! Opt-in clipboard fallback (config `typing.use_clipboard`).
//!
//! **Never** on the default path: charter C4 forbids it, and the fast
//! `SendInput` path covers every app we ship for. This exists only for the
//! stubborn targets where Unicode injection is ignored.
//!
//! Restoring the user's clipboard is mandatory. `EnumClipboardFormats` +
//! `GetClipboardData` copies every HGLOBAL-backed format byte-for-byte
//! (CF_UNICODETEXT, CF_TEXT, CF_OEMTEXT, CF_HTML, CF_LOCALE, custom text
//! formats) before we touch it, and puts them all back afterwards.
//!
//! Known limitation (documented in `70-impl-notes-windows.md`): formats backed
//! by GDI handles (CF_BITMAP, CF_METAFILEPICT, CF_ENHMETAFILE) or owner-rendered
//! private formats cannot be copied this way and are dropped.

use std::thread;
use std::time::Duration;

use anyhow::{bail, Result};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData, OpenClipboard,
    SetClipboardData,
};
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
};

use windows::Win32::Foundation::HGLOBAL;

/// `CF_UNICODETEXT`.
pub const CF_UNICODETEXT: u32 = 13;

#[derive(Debug, PartialEq, Eq)]
struct FormatData {
    format: u32,
    bytes: Vec<u8>,
}

/// Paste `text` into whatever has focus, then put the clipboard back.
///
/// Runs on its own thread: the 120 ms hand-off delay must never stall the hook.
pub fn paste_async(text: String) {
    let _ = thread::Builder::new()
        .name("vkb-clipboard".to_string())
        .spawn(move || {
            if let Err(err) = paste_blocking(&text) {
                tracing::warn!(error = %err, "clipboard fallback failed");
            }
        });
}

fn paste_blocking(text: &str) -> Result<()> {
    let backup = capture();
    write_text(text)?;
    let inputs = crate::input::ctrl_v_sequence();
    crate::input::send(&inputs);
    // Give the target time to service WM_PASTE before the clipboard changes back.
    thread::sleep(Duration::from_millis(120));
    if let Some(backup) = backup {
        restore(backup);
    }
    Ok(())
}

fn open_with_retry() -> bool {
    for _ in 0..10 {
        if unsafe { OpenClipboard(None) }.is_ok() {
            return true;
        }
        thread::sleep(Duration::from_millis(20));
    }
    false
}

fn capture() -> Option<Vec<FormatData>> {
    unsafe {
        if !open_with_retry() {
            return None;
        }
        let mut formats = Vec::new();
        let mut current = 0u32;
        loop {
            current = EnumClipboardFormats(current);
            if current == 0 {
                break;
            }
            let Ok(handle) = GetClipboardData(current) else {
                continue;
            };
            if handle.is_invalid() {
                continue;
            }
            let global = HGLOBAL(handle.0);
            let size = GlobalSize(global);
            if size == 0 {
                continue;
            }
            let ptr = GlobalLock(global) as *const u8;
            if ptr.is_null() {
                continue;
            }
            let bytes = std::slice::from_raw_parts(ptr, size).to_vec();
            let _ = GlobalUnlock(global);
            formats.push(FormatData {
                format: current,
                bytes,
            });
        }
        let _ = CloseClipboard();
        Some(formats)
    }
}

fn restore(formats: Vec<FormatData>) {
    unsafe {
        if !open_with_retry() {
            tracing::warn!("clipboard busy; previous contents could not be restored");
            return;
        }
        let _ = EmptyClipboard();
        for entry in formats {
            if !set_bytes(entry.format, &entry.bytes) {
                tracing::warn!(format = entry.format, "clipboard format not restored");
            }
        }
        let _ = CloseClipboard();
    }
}

fn write_text(text: &str) -> Result<()> {
    let mut units: Vec<u16> = text.encode_utf16().collect();
    units.push(0);
    unsafe {
        if !open_with_retry() {
            bail!("clipboard busy");
        }
        let _ = EmptyClipboard();
        let bytes = std::slice::from_raw_parts(units.as_ptr().cast::<u8>(), units.len() * 2);
        let written = set_bytes(CF_UNICODETEXT, bytes);
        let _ = CloseClipboard();
        if !written {
            bail!("SetClipboardData(CF_UNICODETEXT) failed");
        }
    }
    Ok(())
}

unsafe fn set_bytes(format: u32, bytes: &[u8]) -> bool {
    let Ok(handle) = GlobalAlloc(GMEM_MOVEABLE, bytes.len()) else {
        return false;
    };
    let ptr = GlobalLock(handle);
    if ptr.is_null() {
        return false;
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr.cast::<u8>(), bytes.len());
    let _ = GlobalUnlock(handle);
    SetClipboardData(format, HANDLE(handle.0)).is_ok()
}

/// Read the current `CF_UNICODETEXT` (test/diagnostic helper).
pub fn read_text() -> Option<String> {
    unsafe {
        if !open_with_retry() {
            return None;
        }
        let out = GetClipboardData(CF_UNICODETEXT).ok().and_then(|handle| {
            if handle.is_invalid() {
                return None;
            }
            let global = HGLOBAL(handle.0);
            let ptr = GlobalLock(global) as *const u16;
            if ptr.is_null() {
                return None;
            }
            let mut length = 0usize;
            while *ptr.add(length) != 0 {
                length += 1;
                if length > 1 << 20 {
                    break;
                }
            }
            let text = String::from_utf16_lossy(std::slice::from_raw_parts(ptr, length));
            let _ = GlobalUnlock(global);
            Some(text)
        });
        let _ = CloseClipboard();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Touches the real clipboard: run explicitly.
    #[test]
    #[ignore = "clobbers the clipboard; run with --ignored on an idle desktop"]
    fn text_round_trip_restores_previous_contents() {
        let previous = read_text();
        write_text("tạm thời").expect("write");
        assert_eq!(read_text().as_deref(), Some("tạm thời"));
        let backup = capture().expect("capture");
        write_text("khác").expect("write");
        restore(backup);
        assert_eq!(read_text().as_deref(), Some("tạm thời"));
        // Put the user's clipboard back.
        match previous {
            Some(text) => write_text(&text).expect("restore original"),
            None => {
                unsafe {
                    if open_with_retry() {
                        let _ = EmptyClipboard();
                        let _ = CloseClipboard();
                    }
                }
            }
        }
    }
}
