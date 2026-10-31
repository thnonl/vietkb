//! Native window / webview background colour from the effective theme.
//!
//! A new window shows its native background until the page has painted: white
//! on both platforms, so a dark theme flashed white for 60-120 ms (D-WIN-04).
//! The colour below is the page's own `html` background (`--plane` in
//! `ui/src/styles/tokens.css`), so the first frame already matches.

use tauri::window::Color;
use tauri::Manager;
use vkb_config::Theme;

/// `--plane` of the light palette (`#f2f0eb`).
const LIGHT: Color = Color(0xf2, 0xf0, 0xeb, 0xff);
/// `--plane` of the dark palette (`#202020`).
const DARK: Color = Color(0x20, 0x20, 0x20, 0xff);

/// The palette the page will paint: an explicit choice wins, `System` follows
/// the OS (`None` = unknown, treated as light like the page's own default).
pub fn is_dark(theme: Theme, os: Option<tauri::Theme>) -> bool {
    match theme {
        Theme::Light => false,
        Theme::Dark => true,
        Theme::System => os == Some(tauri::Theme::Dark),
    }
}

pub fn color(dark: bool) -> Color {
    if dark {
        DARK
    } else {
        LIGHT
    }
}

fn configured_theme() -> Theme {
    crate::commands::config_cell().read().system.theme
}

/// The window-level theme for a new window: an explicit choice pins it (so
/// WebView2's `prefers-color-scheme` and the DWM frame cannot follow a dark OS
/// under `theme = light`, D-WIN-08); `System` leaves it to the OS.
pub fn window_theme(theme: Theme) -> Option<tauri::Theme> {
    match theme {
        Theme::Light => Some(tauri::Theme::Light),
        Theme::Dark => Some(tauri::Theme::Dark),
        Theme::System => None,
    }
}

pub fn configured_window_theme() -> Option<tauri::Theme> {
    window_theme(configured_theme())
}

/// Script run before the page's own: hands the saved theme to `main.ts`, which
/// otherwise guesses from a localStorage mirror that can lag the real setting
/// (first run, config edited by hand, other profile) and paints the OS theme.
pub fn init_script() -> String {
    let name = match configured_theme() {
        Theme::Light => "light",
        Theme::Dark => "dark",
        Theme::System => "system",
    };
    format!("window.__VKB_THEME__ = \"{name}\";")
}

/// Best guess before the window exists: the configured theme, or for `System`
/// the OS theme as reported by a window that is already open.
pub fn initial(app: &tauri::AppHandle) -> Color {
    let os = app
        .webview_windows()
        .values()
        .find_map(|window| window.theme().ok());
    color(is_dark(configured_theme(), os))
}

/// Exact colour once the (still hidden) window can tell the OS theme. Call
/// before `show()`.
pub fn apply(window: &tauri::WebviewWindow) {
    // Windows has no app-wide theme: pin (or release) each window itself.
    #[cfg(not(target_os = "macos"))]
    if let Err(error) = window.set_theme(configured_window_theme()) {
        tracing::warn!(%error, "could not set the window theme");
    }
    let dark = is_dark(configured_theme(), window.theme().ok());
    if let Err(error) = window.set_background_color(Some(color(dark))) {
        tracing::warn!(%error, "could not set the window background");
    }
}

/// The OS appearance changed under an open window.
pub fn on_os_theme(window: &tauri::Window, os: tauri::Theme) {
    let dark = is_dark(configured_theme(), Some(os));
    if let Err(error) = window.set_background_color(Some(color(dark))) {
        tracing::warn!(%error, "could not set the window background");
    }
}

/// The theme setting changed: repaint the native background of every window.
pub fn apply_all(app: &tauri::AppHandle) {
    for window in app.webview_windows().values() {
        apply(window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_theme_ignores_the_os() {
        assert!(!is_dark(Theme::Light, Some(tauri::Theme::Dark)));
        assert!(is_dark(Theme::Dark, Some(tauri::Theme::Light)));
        assert!(is_dark(Theme::Dark, None));
    }

    #[test]
    fn system_theme_follows_the_os() {
        assert!(is_dark(Theme::System, Some(tauri::Theme::Dark)));
        assert!(!is_dark(Theme::System, Some(tauri::Theme::Light)));
        assert!(!is_dark(Theme::System, None));
    }

    #[test]
    fn explicit_theme_pins_the_window_theme() {
        assert_eq!(window_theme(Theme::Light), Some(tauri::Theme::Light));
        assert_eq!(window_theme(Theme::Dark), Some(tauri::Theme::Dark));
        assert_eq!(window_theme(Theme::System), None);
    }

    #[test]
    fn colours_match_the_page_planes() {
        assert_eq!(color(true), Color(0x20, 0x20, 0x20, 0xff));
        assert_eq!(color(false), Color(0xf2, 0xf0, 0xeb, 0xff));
    }
}
