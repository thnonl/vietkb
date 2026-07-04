use std::env;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::Media::Audio::{PlaySoundW, SND_ALIAS, SND_ASYNC};
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
    KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ,
};

const RUN_KEY: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const RUN_VALUE: PCWSTR = w!("VietKB");

/// Registers or unregisters VietKB in the current user's autostart list so it
/// launches with Windows when `enabled` is `true`.
pub fn set_start_with_windows(enabled: bool) -> anyhow::Result<()> {
    let mut key = HKEY::default();
    let status = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            RUN_KEY,
            0,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            None,
            &mut key,
            None,
        )
    };
    if status != ERROR_SUCCESS {
        anyhow::bail!("failed to open Run registry key: {status:?}");
    }

    let result = if enabled {
        write_autostart_value(key)
    } else {
        delete_autostart_value(key)
    };

    unsafe {
        let _ = RegCloseKey(key);
    }
    result
}

fn write_autostart_value(key: HKEY) -> anyhow::Result<()> {
    let exe = env::current_exe()?;
    let quoted = format!("\"{}\"", exe.display());
    let mut wide: Vec<u16> = quoted.encode_utf16().collect();
    wide.push(0);
    let bytes = unsafe { std::slice::from_raw_parts(wide.as_ptr().cast::<u8>(), wide.len() * 2) };

    let status = unsafe { RegSetValueExW(key, RUN_VALUE, 0, REG_SZ, Some(bytes)) };
    if status != ERROR_SUCCESS {
        anyhow::bail!("failed to write autostart value: {status:?}");
    }
    Ok(())
}

fn delete_autostart_value(key: HKEY) -> anyhow::Result<()> {
    // Deleting a value that does not exist is not an error for our purposes.
    unsafe {
        let _ = RegDeleteValueW(key, RUN_VALUE);
    }
    Ok(())
}

/// Plays the default system notification sound asynchronously.
pub fn play_notification_sound() {
    unsafe {
        let _ = PlaySoundW(w!("SystemAsterisk"), None, SND_ALIAS | SND_ASYNC);
    }
}
