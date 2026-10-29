//! Where this crate's opt-in debug outputs land.
//!
//! The app owns the data directory: `vkb-app` calls [`set_data_dir`] once at
//! startup with `vkb_config::paths::config_dir()`, which is channel-aware
//! (`%APPDATA%\VietKB Dev` for the dev build). Resolving it here would duplicate
//! that rule and let a dev build write into the production profile.
//!
//! Standalone use (no app): the directory falls back to the historical
//! `%APPDATA%\VietKB` (or the temp dir when `APPDATA` is unset), so nothing
//! changes for a test harness or a benchmark binary that links this crate.
//!
//! Cost: the directory is resolved once, on first use, and never on the
//! keystroke path — [`data_dir`] only hands out a `&'static Path`, and the
//! modules build the file path only when a file is actually written.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Directory used when the app never called [`set_data_dir`].
const FALLBACK_APP_DIR: &str = "VietKB";

static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Point the debug outputs at the app's data directory. Call once, at startup,
/// before the hook is installed. Later calls (and a crate that already fell back
/// to `%APPDATA%\VietKB`) are ignored, so the first resolution wins.
pub fn set_data_dir(dir: PathBuf) {
    if DATA_DIR.set(dir).is_err() {
        tracing::debug!(
            target: "vkb_windows::paths",
            "data directory already resolved; ignoring set_data_dir"
        );
    }
}

/// The directory the debug outputs are written under. Resolved once.
pub(crate) fn data_dir() -> &'static Path {
    DATA_DIR.get_or_init(|| {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join(FALLBACK_APP_DIR)
    })
}
