//! The `Run` entry this build owns.
//!
//! Production delegates to `vkb-windows` (`HKCU\...\Run\VietKB`). The
//! `dev-channel` build must not share that value, so it writes the same shape
//! under `VietKB Dev` itself: quoted absolute exe plus `--autostart`, HKCU only.
//!
//! It cannot reuse the Tauri autostart plugin for this: `auto-launch` writes an
//! unquoted command line and picks `HKLM` first when the process may write
//! there. The channel value has to stay exactly `"<exe>" --autostart`.

#[cfg(any(windows, target_os = "macos"))]
pub use imp::{is_enabled, set};

/// macOS: the LaunchAgent `~/Library/LaunchAgents/vn.vietkb.app.plist` owned by
/// `vkb-macos` (decision M1).
#[cfg(target_os = "macos")]
mod imp {
    pub fn set(enabled: bool) -> Result<(), String> {
        vkb_macos::set_autostart(enabled).map_err(|error| error.to_string())
    }

    pub fn is_enabled() -> bool {
        vkb_macos::is_autostart_enabled()
    }
}

#[cfg(all(windows, not(feature = "dev-channel")))]
mod imp {
    pub fn set(enabled: bool) -> Result<(), String> {
        vkb_windows::set_autostart(enabled).map_err(|error| error.to_string())
    }

    pub fn is_enabled() -> bool {
        vkb_windows::is_autostart_enabled()
    }
}

#[cfg(all(windows, feature = "dev-channel"))]
mod imp {
    use windows::core::PCWSTR;
    use windows::Win32::System::Registry::{
        RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_SZ,
        RRF_RT_REG_SZ,
    };

    pub const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    pub const VALUE_NAME: &str = "VietKB Dev";
    pub const AUTOSTART_FLAG: &str = "--autostart";

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// The exact command line stored in the `Run` value.
    pub fn command(exe: &std::path::Path) -> String {
        format!("\"{}\" {AUTOSTART_FLAG}", exe.display())
    }

    pub fn set(enabled: bool) -> Result<(), String> {
        let subkey = wide(RUN_KEY);
        let name = wide(VALUE_NAME);

        if enabled {
            let exe = std::env::current_exe().map_err(|error| error.to_string())?;
            let data = wide(&command(&exe));
            let status = unsafe {
                RegSetKeyValueW(
                    HKEY_CURRENT_USER,
                    PCWSTR(subkey.as_ptr()),
                    PCWSTR(name.as_ptr()),
                    REG_SZ.0,
                    Some(data.as_ptr().cast()),
                    (data.len() * std::mem::size_of::<u16>()) as u32,
                )
            };
            if status.0 != 0 {
                return Err(format!("RegSetKeyValueW failed: {status:?}"));
            }
            return Ok(());
        }

        let status = unsafe {
            RegDeleteKeyValueW(
                HKEY_CURRENT_USER,
                PCWSTR(subkey.as_ptr()),
                PCWSTR(name.as_ptr()),
            )
        };
        // 2 = ERROR_FILE_NOT_FOUND: nothing to remove, which is success here.
        if status.0 != 0 && status.0 != 2 {
            return Err(format!("RegDeleteKeyValueW failed: {status:?}"));
        }
        Ok(())
    }

    /// The command line currently registered under `VietKB Dev`, if any.
    pub fn entry() -> Option<String> {
        let subkey = wide(RUN_KEY);
        let name = wide(VALUE_NAME);
        let mut buffer = [0u16; 1024];
        let mut bytes = (buffer.len() * std::mem::size_of::<u16>()) as u32;
        let status = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                PCWSTR(subkey.as_ptr()),
                PCWSTR(name.as_ptr()),
                RRF_RT_REG_SZ,
                None,
                Some(buffer.as_mut_ptr().cast()),
                Some(&mut bytes),
            )
        };
        if status.0 != 0 || bytes < 2 {
            return None;
        }
        let units = (bytes as usize / std::mem::size_of::<u16>()).saturating_sub(1);
        Some(String::from_utf16_lossy(&buffer[..units]))
    }

    pub fn is_enabled() -> bool {
        entry().is_some_and(|value| !value.trim().is_empty())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn command_is_the_quoted_exe_plus_the_flag() {
            let cmd = command(std::path::Path::new(r"C:\Users\test\VietKB Dev\vietkb.exe"));
            assert_eq!(
                cmd,
                "\"C:\\Users\\test\\VietKB Dev\\vietkb.exe\" --autostart"
            );
        }

        #[test]
        fn value_name_is_the_dev_one() {
            assert_eq!(VALUE_NAME, "VietKB Dev");
            assert_ne!(VALUE_NAME, "VietKB");
        }
    }
}
