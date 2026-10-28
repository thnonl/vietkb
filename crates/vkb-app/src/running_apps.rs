//! Enumerate visible top-level windows so the app-exclusion picker can offer
//! "chrome.exe — Google Chrome" instead of asking the user to type an exe name.
//!
//! Kept in `vkb-app` because it is UI plumbing, not input handling; the OS
//! backend crate owns the hook. Returns `(process, title)` pairs.

/// Visible windows with a title, de-duplicated by process name (the first
/// window of a process wins). Sorted by process name.
#[cfg(windows)]
pub fn list() -> Vec<(String, String)> {
    use windows::core::{PWSTR, BOOL};
    use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM};
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
        IsWindowVisible,
    };

    unsafe extern "system" fn collect(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let out = &mut *(lparam.0 as *mut Vec<(String, String)>);
        if IsWindowVisible(hwnd).0 == 0 {
            return BOOL(1);
        }
        let length = GetWindowTextLengthW(hwnd);
        if length <= 0 {
            return BOOL(1);
        }
        let mut title_buf = vec![0u16; length as usize + 1];
        let copied = GetWindowTextW(hwnd, &mut title_buf);
        if copied <= 0 {
            return BOOL(1);
        }
        let title = String::from_utf16_lossy(&title_buf[..copied as usize]).trim().to_string();
        if title.is_empty() {
            return BOOL(1);
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid as *mut u32));
        if pid == 0 {
            return BOOL(1);
        }
        let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return BOOL(1);
        };

        let mut path_buf = vec![0u16; 1024];
        let mut size = path_buf.len() as u32;
        let mut process = String::new();
        if QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(path_buf.as_mut_ptr()),
            &mut size,
        )
        .is_ok()
        {
            let full = String::from_utf16_lossy(&path_buf[..size as usize]);
            process = full
                .rsplit(['\\', '/'])
                .next()
                .unwrap_or(&full)
                .to_ascii_lowercase();
        }
        let _ = CloseHandle(handle);

        if !process.is_empty() {
            out.push((process, title));
        }
        BOOL(1)
    }

    let mut found: Vec<(String, String)> = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(collect), LPARAM(&mut found as *mut _ as isize));
    }

    found.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.len().cmp(&a.1.len())));
    found.dedup_by(|a, b| a.0 == b.0);
    found
}

#[cfg(not(windows))]
pub fn list() -> Vec<(String, String)> {
    Vec::new()
}
