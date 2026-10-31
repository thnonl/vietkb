//! Where things live on disk.

use std::path::PathBuf;

/// Directory under the platform config root. The `dev-channel` build gets its
/// own, so a test install never shares config, logs or perf files with the
/// production app.
#[cfg(feature = "dev-channel")]
pub const APP_DIR: &str = "VietKB Dev";
#[cfg(not(feature = "dev-channel"))]
pub const APP_DIR: &str = "VietKB";
pub const CONFIG_NAME: &str = "config.toml";

/// `%APPDATA%\VietKB` on Windows, `~/Library/Application Support/VietKB` on macOS
/// (`VietKB Dev` with the `dev-channel` feature).
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

/// `true` for a cargo test/bench binary (`.../target/<profile>/deps/x-<hash>`).
/// Such a file is rebuilt, moved or left stale, so no autostart entry may ever
/// point at it, and a test run must not clear the user's real entry either.
pub fn is_cargo_deps_binary(exe: &std::path::Path) -> bool {
    let in_deps = exe
        .parent()
        .and_then(|dir| dir.file_name())
        .is_some_and(|name| name.eq_ignore_ascii_case("deps"));
    in_deps
        && exe
            .components()
            .any(|part| part.as_os_str().eq_ignore_ascii_case("target"))
}

/// The running process is a cargo test binary. `false` when the exe path is
/// unknown (so disabling autostart is never blocked by a failing lookup).
pub fn running_from_cargo_deps() -> bool {
    std::env::current_exe().is_ok_and(|exe| is_cargo_deps_binary(&exe))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_cargo_deps_binaries_are_recognised() {
        let deps = PathBuf::from("repo").join("target").join("debug").join("deps").join("x-1.exe");
        assert!(is_cargo_deps_binary(&deps));
        let app = PathBuf::from("repo").join("target").join("debug").join("vkb-app.exe");
        assert!(!is_cargo_deps_binary(&app));
        let installed = PathBuf::from("Programs").join("VietKB").join("vkb-app.exe");
        assert!(!is_cargo_deps_binary(&installed));
    }
}
