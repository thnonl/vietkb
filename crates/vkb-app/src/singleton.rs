//! macOS launch guard that runs before the single-instance plugin.
//!
//! `tauri-plugin-single-instance` decides "am I first?" by connecting to a Unix
//! socket in `/tmp` and, when nobody answers, removing the file and binding it.
//! Two launches in the same few milliseconds (`open -n` twice, a LaunchAgent at
//! login racing a Finder double-click) can both find no socket, and the second
//! `remove_file` + `bind` then steals the path: two menu-bar icons, two event
//! taps. An advisory `flock` on a file in the config directory closes that
//! window: the loser of the lock waits a moment so the winner has bound its
//! socket, then continues into the plugin, which hands over and exits.
//!
//! The lock lives as long as the process (the descriptor is leaked on purpose)
//! and the kernel drops it on any exit, crash included, so it can never go stale.

use std::fs::{File, OpenOptions};
use std::os::fd::AsRawFd;
use std::path::Path;
use std::time::Duration;

extern "C" {
    fn flock(fd: i32, operation: i32) -> i32;
}
const LOCK_EX: i32 = 2;
const LOCK_NB: i32 = 4;

/// How long the loser waits for the winner to bind its socket. Plugin setup
/// takes a few milliseconds; this is generous and only paid by a second launch.
const WINNER_SETTLE: Duration = Duration::from_millis(500);

pub enum Claim {
    /// This process holds the lock (keep the file alive).
    First(File),
    /// Another live process holds it.
    Taken,
    /// The lock file could not be used; fall back to the plugin alone.
    Unavailable,
}

pub fn try_claim(path: &Path) -> Claim {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let file = match OpenOptions::new().create(true).write(true).truncate(false).open(path) {
        Ok(file) => file,
        Err(_) => return Claim::Unavailable,
    };
    // SAFETY: `flock` on a descriptor this function owns; no memory is passed.
    let rc = unsafe { flock(file.as_raw_fd(), LOCK_EX | LOCK_NB) };
    if rc == 0 {
        Claim::First(file)
    } else {
        Claim::Taken
    }
}

/// Call first thing in `main`, before the plugin and before any UI. The winner
/// returns at once; a loser sleeps `WINNER_SETTLE` and returns, and the
/// single-instance plugin then connects to the winner's socket and exits.
pub fn guard() {
    match try_claim(&vkb_config::paths::config_dir().join("instance.lock")) {
        Claim::First(file) => std::mem::forget(file),
        Claim::Taken => {
            tracing::info!("another VietKB is running; handing over");
            std::thread::sleep(WINNER_SETTLE);
        }
        Claim::Unavailable => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_second_claim_on_one_file_is_taken_until_the_first_is_dropped() {
        let dir = std::env::temp_dir().join(format!("vkb-singleton-{}", std::process::id()));
        let path = dir.join("instance.lock");
        let first = try_claim(&path);
        assert!(matches!(first, Claim::First(_)));
        assert!(matches!(try_claim(&path), Claim::Taken));
        drop(first);
        assert!(matches!(try_claim(&path), Claim::First(_)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_unusable_path_falls_back_to_the_plugin() {
        assert!(matches!(
            try_claim(Path::new("/dev/null/nope/instance.lock")),
            Claim::Unavailable
        ));
    }
}
