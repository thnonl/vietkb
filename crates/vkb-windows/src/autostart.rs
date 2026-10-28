//! Autostart through `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
//!
//! Per-user, no elevation, no Task Scheduler (charter: never require admin).
//! The value is a quoted absolute path to the current executable followed by
//! `--autostart`, so the app can tell a Windows launch from a manual one.

use anyhow::{bail, Context, Result};
use windows::core::PCWSTR;
use windows::Win32::System::Registry::{
    RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ,
};

pub const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
pub const VALUE_NAME: &str = "VietKB";

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Argument the app checks at startup to know Windows started it.
pub const AUTOSTART_FLAG: &str = "--autostart";

/// Command line written to the `Run` value for `exe`: the quoted path, then
/// the flag. Without the flag the app cannot honour `start_minimized`.
pub fn autostart_command(exe: &std::path::Path) -> String {
    format!("\"{}\" {AUTOSTART_FLAG}", exe.display())
}

/// Create or remove the `Run` entry. Idempotent.
pub fn set_autostart(enabled: bool) -> Result<()> {
    let subkey = wide(RUN_KEY);
    let name = wide(VALUE_NAME);

    if enabled {
        let exe = std::env::current_exe().context("locating the current executable")?;
        let command = autostart_command(&exe);
        let data = wide(&command);
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
        if !status.is_ok() {
            bail!("RegSetKeyValueW failed: {:?}", status);
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
    // 2 = ERROR_FILE_NOT_FOUND: nothing to remove, which is success for us.
    if !status.is_ok() && status.0 != 2 {
        bail!("RegDeleteKeyValueW failed: {:?}", status);
    }
    Ok(())
}

/// The command line currently registered, if any.
pub fn autostart_entry() -> Option<String> {
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
    if !status.is_ok() || bytes < 2 {
        return None;
    }
    let units = (bytes as usize / std::mem::size_of::<u16>()).saturating_sub(1);
    Some(String::from_utf16_lossy(&buffer[..units]))
}

/// A `Run` value means "autostart on" whether or not it carries the flag: an
/// entry written by an older build is the bare quoted path, and an empty value
/// is not an entry at all.
pub fn entry_enables_autostart(entry: &str) -> bool {
    !entry.trim().is_empty()
}

pub fn is_autostart_enabled() -> bool {
    autostart_entry().is_some_and(|value| entry_enables_autostart(&value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn command_is_quoted_for_spaces() {
        let cmd = autostart_command(Path::new(r"C:\Program Files\VietKB\vkb-app.exe"));
        assert_eq!(
            cmd,
            "\"C:\\Program Files\\VietKB\\vkb-app.exe\" --autostart"
        );
        assert!(cmd.starts_with('"'), "path must stay quoted: {cmd}");
        assert!(cmd.ends_with(" --autostart"), "flag must be there: {cmd}");
    }

    #[test]
    fn entry_counts_as_enabled_with_or_without_the_flag() {
        assert!(entry_enables_autostart(
            r#""C:\VietKB\vkb-app.exe" --autostart"#
        ));
        assert!(
            entry_enables_autostart(r#""C:\VietKB\vkb-app.exe""#),
            "an entry written by an older build is still enabled"
        );
        assert!(!entry_enables_autostart(""));
        assert!(!entry_enables_autostart("   "));
    }

    #[test]
    fn wide_strings_are_nul_terminated() {
        let w = wide("VietKB");
        assert_eq!(w.len(), 7);
        assert_eq!(w[6], 0);
    }

    #[test]
    fn registry_paths_match_the_documented_ones() {
        assert_eq!(
            RUN_KEY,
            r"Software\Microsoft\Windows\CurrentVersion\Run",
            "contract: HKCU Run, nothing else"
        );
        assert_eq!(VALUE_NAME, "VietKB");
    }

    #[test]
    fn registry_write_round_trip() {
        // Writes only under HKCU; restores the previous value so a developer
        // machine keeps its state.
        let previous = autostart_entry();
        set_autostart(true).expect("write Run entry");
        let written = autostart_entry().expect("entry present");
        assert!(written.contains("vkb"), "unexpected entry: {written}");
        assert!(written.starts_with('"'));
        assert!(
            written.ends_with(" --autostart"),
            "the app needs the flag to know it was started by Windows: {written}"
        );
        assert_eq!(
            Some(written.clone()),
            std::env::current_exe().ok().map(|exe| autostart_command(&exe)),
            "the value must be the quoted exe plus the flag, nothing else"
        );
        assert!(is_autostart_enabled());
        set_autostart(false).expect("remove Run entry");
        assert_eq!(autostart_entry(), None);
        assert!(!is_autostart_enabled());
        if let Some(previous) = previous {
            set_autostart(true).ok();
            let _ = previous;
        }
    }
}
