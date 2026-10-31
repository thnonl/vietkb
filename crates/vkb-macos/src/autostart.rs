//! Autostart through a per-user LaunchAgent:
//! `~/Library/LaunchAgents/vn.vietkb.app.plist` (`RunAtLoad`, no elevation).
//!
//! The plist generation and the file handling are pure and take the directory
//! and executable as parameters, so tests never touch the real `~/Library`.
//! `VKB_LAUNCH_AGENTS_DIR` overrides the directory for harnesses.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

/// LaunchAgent label and plist base name.
pub const LABEL: &str = "vn.vietkb.app";
/// File name inside the LaunchAgents directory.
pub const PLIST_NAME: &str = "vn.vietkb.app.plist";
/// Argument the app checks at startup to know the login item started it.
pub const AUTOSTART_FLAG: &str = "--autostart";
/// Environment override for the LaunchAgents directory (tests, harnesses).
pub const DIR_ENV: &str = "VKB_LAUNCH_AGENTS_DIR";

/// Escape the five XML special characters.
fn xml_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            other => out.push(other),
        }
    }
    out
}

/// The LaunchAgent plist for `exe`.
pub fn plist_contents(exe: &Path) -> String {
    format!(
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            "<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" ",
            "\"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n",
            "<plist version=\"1.0\">\n",
            "<dict>\n",
            "  <key>Label</key>\n",
            "  <string>{label}</string>\n",
            "  <key>ProgramArguments</key>\n",
            "  <array>\n",
            "    <string>{exe}</string>\n",
            "    <string>{flag}</string>\n",
            "  </array>\n",
            "  <key>RunAtLoad</key>\n",
            "  <true/>\n",
            "  <key>ProcessType</key>\n",
            "  <string>Interactive</string>\n",
            "</dict>\n",
            "</plist>\n",
        ),
        label = LABEL,
        exe = xml_escape(&exe.to_string_lossy()),
        flag = AUTOSTART_FLAG,
    )
}

/// `~/Library/LaunchAgents`, or the `VKB_LAUNCH_AGENTS_DIR` override.
pub fn launch_agents_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os(DIR_ENV).filter(|dir| !dir.is_empty()) {
        return Some(PathBuf::from(dir));
    }
    std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(|home| PathBuf::from(home).join("Library").join("LaunchAgents"))
}

fn plist_path(dir: &Path) -> PathBuf {
    dir.join(PLIST_NAME)
}

/// Create or remove the plist in `dir`. Idempotent.
pub fn set_autostart_in(dir: &Path, exe: &Path, enabled: bool) -> Result<()> {
    let path = plist_path(dir);
    if enabled {
        fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        let contents = plist_contents(exe);
        // Rewrite only on change so a no-op toggle does not touch the file.
        if fs::read_to_string(&path).ok().as_deref() != Some(contents.as_str()) {
            fs::write(&path, contents).with_context(|| format!("writing {}", path.display()))?;
        }
        return Ok(());
    }
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => bail!("removing {}: {err}", path.display()),
    }
}

/// `true` when `dir` holds a non-empty VietKB plist.
pub fn is_autostart_enabled_in(dir: &Path) -> bool {
    fs::read_to_string(plist_path(dir)).is_ok_and(|text| text.contains(LABEL))
}

/// Where the installed app lives; the stable path a LaunchAgent should name.
pub const INSTALLED_APP: &str = "/Applications/VietKB.app";

/// `true` for a path that will not exist at the next login: a mounted disk
/// image (`/Volumes/...`) or the randomized read-only copy Gatekeeper runs a
/// quarantined, never-moved app from (`.../AppTranslocation/...`).
pub fn is_transient_path(exe: &Path) -> bool {
    let text = exe.to_string_lossy();
    text.starts_with("/Volumes/") || text.contains("/AppTranslocation/")
}

/// Autostart refused: the running app is on a disk image or an App
/// Translocation path that will not exist at the next login. The app layer
/// shows its own localized text; `Display` is the English log line.
#[derive(Debug)]
pub struct TransientPath(pub PathBuf);

impl std::fmt::Display for TransientPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "running from a transient path ({}); move VietKB to /Applications", self.0.display())
    }
}

impl std::error::Error for TransientPath {}

/// The executable a LaunchAgent should start for the running `exe`.
///
/// A transient `exe` (see [`is_transient_path`]) is replaced by the same binary
/// inside `installed_app` when that copy exists; otherwise autostart is refused
/// so no plist is written that breaks at the next login. A stable path is used
/// as is (a dev build under `target/` still works, with a warning when it is
/// outside `Applications`).
pub fn resolve_exe(exe: &Path, installed_app: &Path) -> Result<PathBuf> {
    if !is_transient_path(exe) {
        if !exe.starts_with(installed_app) {
            tracing::warn!(exe = %exe.display(), "autostart points outside {INSTALLED_APP}");
        }
        return Ok(exe.to_path_buf());
    }
    if let Some(name) = exe.file_name() {
        let installed = installed_app.join("Contents").join("MacOS").join(name);
        if installed.is_file() {
            return Ok(installed);
        }
    }
    Err(TransientPath(exe.to_path_buf()).into())
}

/// Create or remove the LaunchAgent for the running executable.
///
/// No `launchctl bootstrap` is needed: `launchd` loads every plist in
/// `~/Library/LaunchAgents` with `RunAtLoad` at the next login, and the app is
/// already running now. Disabling removes the plist and never needs the
/// executable path (so it works from any location, transient or not). A plist
/// from an app that was later Trashed stays behind; the user removes it with the
/// Mở cùng macOS switch while the app still exists.
pub fn set_autostart(enabled: bool) -> Result<()> {
    let dir = launch_agents_dir().context("cannot locate ~/Library/LaunchAgents")?;
    if !enabled {
        return set_autostart_in(&dir, Path::new(""), false);
    }
    let exe = std::env::current_exe().context("locating the current executable")?;
    let exe = resolve_exe(&exe, Path::new(INSTALLED_APP))?;
    set_autostart_in(&dir, &exe, true)
}

pub fn is_autostart_enabled() -> bool {
    launch_agents_dir().is_some_and(|dir| is_autostart_enabled_in(&dir))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "vkb-macos-autostart-{tag}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    const EXE: &str = "/Applications/VietKB.app/Contents/MacOS/vkb-app";

    #[test]
    fn plist_has_label_run_at_load_and_program_arguments() {
        let text = plist_contents(Path::new(EXE));
        assert!(text.starts_with("<?xml"));
        assert!(text.contains("<key>Label</key>\n  <string>vn.vietkb.app</string>"));
        assert!(text.contains("<key>RunAtLoad</key>\n  <true/>"));
        assert!(text.contains(&format!(
            "<array>\n    <string>{EXE}</string>\n    <string>--autostart</string>\n  </array>"
        )));
        assert!(text.trim_end().ends_with("</plist>"));
    }

    #[test]
    fn exe_path_is_xml_escaped() {
        let text = plist_contents(Path::new("/Apps/A&B <x>/vkb"));
        assert!(text.contains("/Apps/A&amp;B &lt;x&gt;/vkb"));
        assert!(!text.contains("A&B"));
    }

    #[test]
    fn enable_then_disable_round_trips_in_a_temp_dir() {
        let dir = temp_dir("roundtrip").join("LaunchAgents");
        assert!(!is_autostart_enabled_in(&dir));

        set_autostart_in(&dir, Path::new(EXE), true).unwrap();
        assert!(is_autostart_enabled_in(&dir));
        let written = fs::read_to_string(dir.join(PLIST_NAME)).unwrap();
        assert_eq!(written, plist_contents(Path::new(EXE)));

        // Idempotent enable.
        set_autostart_in(&dir, Path::new(EXE), true).unwrap();
        assert!(is_autostart_enabled_in(&dir));

        set_autostart_in(&dir, Path::new(EXE), false).unwrap();
        assert!(!is_autostart_enabled_in(&dir));
        // Idempotent disable (file already gone).
        set_autostart_in(&dir, Path::new(EXE), false).unwrap();
        let _ = fs::remove_dir_all(dir.parent().unwrap());
    }

    #[test]
    fn a_moved_app_rewrites_the_plist() {
        let dir = temp_dir("moved");
        set_autostart_in(&dir, Path::new("/old/vkb-app"), true).unwrap();
        set_autostart_in(&dir, Path::new(EXE), true).unwrap();
        let written = fs::read_to_string(dir.join(PLIST_NAME)).unwrap();
        assert!(written.contains(EXE) && !written.contains("/old/"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn dmg_and_translocated_paths_are_transient() {
        assert!(is_transient_path(Path::new("/Volumes/VietKB/VietKB.app/Contents/MacOS/vkb-app")));
        assert!(is_transient_path(Path::new(
            "/private/var/folders/ab/xyz/T/AppTranslocation/1234/d/VietKB.app/Contents/MacOS/vkb-app"
        )));
        assert!(!is_transient_path(Path::new(EXE)));
        assert!(!is_transient_path(Path::new("/Users/me/Volumes/x")));
    }

    #[test]
    fn transient_exe_is_redirected_to_the_installed_copy_or_refused() {
        let root = temp_dir("resolve");
        let app = root.join("VietKB.app");
        let transient = Path::new("/Volumes/VietKB/VietKB.app/Contents/MacOS/vkb-app");

        // Not installed: refused.
        assert!(resolve_exe(transient, &app).is_err());

        // Installed: the stable binary is used.
        let installed = app.join("Contents/MacOS/vkb-app");
        fs::create_dir_all(installed.parent().unwrap()).unwrap();
        fs::write(&installed, b"").unwrap();
        assert_eq!(resolve_exe(transient, &app).unwrap(), installed);

        // A stable path is kept as is, even outside the app folder.
        assert_eq!(resolve_exe(Path::new(EXE), &app).unwrap(), Path::new(EXE));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn disabling_removes_the_plist_without_needing_an_exe() {
        let dir = temp_dir("disable");
        set_autostart_in(&dir, Path::new(EXE), true).unwrap();
        set_autostart_in(&dir, Path::new(""), false).unwrap();
        assert!(!dir.join(PLIST_NAME).exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_empty_plist_file_does_not_count_as_enabled() {
        let dir = temp_dir("empty");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(PLIST_NAME), "").unwrap();
        assert!(!is_autostart_enabled_in(&dir));
        let _ = fs::remove_dir_all(&dir);
    }
}
