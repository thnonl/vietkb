//! Foreground window probe with a short-lived cache.
//!
//! A `QueryFullProcessImageNameW` per keystroke is a performance trap (P2), so
//! the hook asks this module only when the foreground window actually changes,
//! plus a TTL re-check for HWND reuse. The cache is a plain field of the hook
//! thread's runtime — no lock, nothing to block on.

use std::path::PathBuf;

use vkb_platform::AppInfo;
use windows::Win32::Foundation::{CloseHandle, HWND};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
};
use windows::core::PWSTR;

/// Identity of the window owning the caret right now.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AppKey {
    pub hwnd: isize,
    pub pid: u32,
}

impl AppKey {
    pub fn is_valid(&self) -> bool {
        self.hwnd != 0 && self.pid != 0
    }
}

fn hwnd_from(raw: isize) -> HWND {
    HWND(raw as _)
}

/// Raw `HWND` of the foreground window (0 when none).
pub fn foreground_hwnd() -> isize {
    unsafe { GetForegroundWindow() }.0 as isize
}

/// `true` when the foreground window belongs to this process (a VietKB window
/// has the focus). Always `true` in unit tests and the E2E harness build, which
/// have no real foreground window.
#[cfg(not(any(test, feature = "e2e-harness")))]
pub fn own_process_in_front() -> bool {
    let hwnd = foreground_hwnd();
    if hwnd == 0 {
        return false;
    }
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd_from(hwnd), Some(&mut pid)) };
    pid == std::process::id()
}

#[cfg(any(test, feature = "e2e-harness"))]
pub fn own_process_in_front() -> bool {
    true
}

/// Lowercased executable name + pid + path + title for one window.
///
/// Allocates; call only on window/process change, never per keystroke.
pub fn probe(hwnd: isize) -> Option<(AppKey, AppInfo)> {
    if hwnd == 0 {
        return None;
    }
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd_from(hwnd), Some(&mut pid)) };
    if pid == 0 {
        return None;
    }

    let handle = unsafe {
        let inherit: windows::Win32::Foundation::BOOL = false.into();
        OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, inherit, pid)
    }
    .ok()?;
    let mut buffer = [0u16; 1024];
    let mut size = buffer.len() as u32;
    let ok = unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_FORMAT(0),
            PWSTR(buffer.as_mut_ptr()),
            &mut size,
        )
        .is_ok()
    };
    let _ = unsafe { CloseHandle(handle) };
    if !ok || size == 0 {
        return None;
    }

    let path = String::from_utf16_lossy(&buffer[..size as usize]);
    let process_name = vkb_platform::rules::normalize_process_name(&path);
    if process_name.is_empty() {
        return None;
    }

    Some((
        AppKey { hwnd, pid },
        AppInfo {
            process_name,
            exe_path: Some(PathBuf::from(path)),
            window_title: window_title(hwnd),
        },
    ))
}

/// Process name only — the common case for `active_app_name()`.
pub fn process_name(hwnd: isize) -> Option<String> {
    probe(hwnd).map(|(_, app)| app.process_name)
}

/// Title of the window, trimmed to something a settings row can show.
pub fn window_title(hwnd: isize) -> Option<String> {
    let mut buffer = [0u16; 256];
    let len = unsafe { GetWindowTextW(hwnd_from(hwnd), &mut buffer) };
    if len <= 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&buffer[..len as usize]))
}

/// Uncached probe for API callers (`active_app_name`, `list_running_apps`).
pub fn active_app() -> Option<AppInfo> {
    probe(foreground_hwnd()).map(|(_, app)| app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_key_detection() {
        assert!(!AppKey::default().is_valid());
        assert!(AppKey { hwnd: 1, pid: 2 }.is_valid());
    }

    #[test]
    fn foreground_probe_returns_a_process_on_a_live_session() {
        // Works in any interactive session; in a headless CI service it may
        // legitimately be `None`, so only assert the shape when present.
        if let Some(app) = active_app() {
            assert!(!app.process_name.is_empty());
            assert!(app.process_name.ends_with(".exe"));
            assert_eq!(app.process_name, app.process_name.to_ascii_lowercase());
        }
    }
}
