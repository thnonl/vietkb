//! Where things live on disk.

use std::path::PathBuf;

pub const APP_DIR: &str = "VietKB";
pub const CONFIG_NAME: &str = "config.toml";

/// `%APPDATA%\VietKB` on Windows, `~/Library/Application Support/VietKB` on macOS.
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_DIR)
}

pub fn config_file() -> PathBuf {
    config_dir().join(CONFIG_NAME)
}

pub fn logs_dir() -> PathBuf {
    config_dir().join("logs")
}

/// Where a legacy EVKey/UniKey `settings.ini` might be found next to the exe.
pub fn legacy_ini_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            out.push(dir.join("settings.ini"));
            out.push(dir.join("EVKey.ini"));
        }
    }
    out.push(config_dir().join("settings.ini"));
    out
}
