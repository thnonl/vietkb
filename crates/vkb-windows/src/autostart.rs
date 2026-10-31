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

fn write_value(subkey: &str, value_name: &str, data: &str) -> Result<()> {
    let subkey = wide(subkey);
    let name = wide(value_name);
    let data = wide(data);
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
    Ok(())
}

fn delete_value(subkey: &str, value_name: &str) -> Result<()> {
    let subkey = wide(subkey);
    let name = wide(value_name);
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

fn read_value(subkey: &str, value_name: &str) -> Option<String> {
    let subkey = wide(subkey);
    let name = wide(value_name);
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

/// `true` for a cargo test/bench binary (`...\target\<profile>\deps\x-<hash>.exe`).
/// Such a file is rebuilt, moved or left stale, and Windows would launch it at
/// every login: no `Run` value may ever point at one (D-AUTO-01, Win QA 0.2.7).
pub fn is_cargo_deps_binary(exe: &std::path::Path) -> bool {
    let parent = exe.parent().and_then(|dir| dir.file_name());
    parent.is_some_and(|name| name.eq_ignore_ascii_case("deps"))
        && exe
            .components()
            .any(|part| part.as_os_str().eq_ignore_ascii_case("target"))
}

/// `set_autostart` against an arbitrary key/value; the public one pins both.
fn set_in(subkey: &str, value_name: &str, enabled: bool) -> Result<()> {
    if enabled {
        let exe = std::env::current_exe().context("locating the current executable")?;
        return write_value(subkey, value_name, &autostart_command(&exe));
    }
    delete_value(subkey, value_name)
}

/// Create or remove the `Run` entry. Idempotent.
pub fn set_autostart(enabled: bool) -> Result<()> {
    // Enabling would register a throw-away exe; disabling would delete the real
    // value. A cargo test binary does neither.
    let exe = std::env::current_exe().context("locating the current executable")?;
    if is_cargo_deps_binary(&exe) {
        bail!("refusing to change the Run key from a cargo test binary: {}", exe.display());
    }
    set_in(RUN_KEY, VALUE_NAME, enabled)
}

/// The command line currently registered, if any.
pub fn autostart_entry() -> Option<String> {
    read_value(RUN_KEY, VALUE_NAME)
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
    use windows::Win32::System::Registry::RegDeleteKeyW;

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
    fn cargo_test_binaries_are_never_registered() {
        assert!(is_cargo_deps_binary(Path::new(
            r"E:\vietkb\target\debug\deps\vkb_windows-0123abcd.exe"
        )));
        assert!(!is_cargo_deps_binary(Path::new(r"E:\vietkb\target\debug\vkb-app.exe")));
        assert!(!is_cargo_deps_binary(Path::new(
            r"C:\Users\a\AppData\Local\VietKB\vkb-app.exe"
        )));
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
            RUN_KEY, r"Software\Microsoft\Windows\CurrentVersion\Run",
            "contract: HKCU Run, nothing else"
        );
        assert_eq!(VALUE_NAME, "VietKB");
    }

    /// Test-only key: not under `Run`, so nothing here can ever be launched
    /// by Windows, even if the test is killed midway.
    fn test_subkey() -> String {
        format!(r"Software\VietKB\test-autostart-{}", std::process::id())
    }

    const TEST_VALUE: &str = "autostart";

    /// Removes the value, then the leaf test key, even if an assert panics.
    struct KeyGuard(String);

    impl Drop for KeyGuard {
        fn drop(&mut self) {
            if let Err(error) = delete_value(&self.0, TEST_VALUE) {
                eprintln!("cleanup: deleting test value failed: {error:#}");
            }
            let subkey = wide(&self.0);
            let status = unsafe { RegDeleteKeyW(HKEY_CURRENT_USER, PCWSTR(subkey.as_ptr())) };
            // 2 = ERROR_FILE_NOT_FOUND: never created, nothing to remove.
            if !status.is_ok() && status.0 != 2 {
                eprintln!("cleanup: RegDeleteKeyW({}) failed: {:?}", self.0, status);
            }
        }
    }

    #[test]
    fn registry_write_round_trip() {
        // Never touches `Run`: the real `VietKB` value belongs to the user.
        let key = test_subkey();
        assert_ne!(key, RUN_KEY);
        assert!(!key.starts_with(RUN_KEY));
        let _guard = KeyGuard(key.clone());
        let exe = std::env::current_exe().expect("current exe");
        let expected = autostart_command(&exe);

        // enable: quoted exe plus the flag, read back unchanged.
        set_in(&key, TEST_VALUE, true).expect("write entry");
        let written = read_value(&key, TEST_VALUE).expect("entry present");
        assert!(written.starts_with('"'));
        assert!(
            written.ends_with(" --autostart"),
            "the app needs the flag to know it was started by Windows: {written}"
        );
        assert_eq!(
            written, expected,
            "the value must be the quoted exe plus the flag, nothing else"
        );
        assert!(entry_enables_autostart(&written));

        // disable removes it; a second disable is still Ok (ERROR_FILE_NOT_FOUND).
        set_in(&key, TEST_VALUE, false).expect("remove entry");
        assert_eq!(read_value(&key, TEST_VALUE), None);
        set_in(&key, TEST_VALUE, false).expect("second disable is idempotent");
        assert_eq!(read_value(&key, TEST_VALUE), None);
    }
}
