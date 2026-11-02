//! macOS app activation. Tauri's `set_focus` only orders a window to the front
//! inside its own app; an Accessory app that is not active still sits behind
//! the frontmost app, so the app itself has to be activated.

use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSWindow, NSWindowOcclusionState};

/// Whether the window server reports any part of the window visible
/// (`NSWindowOcclusionStateVisible`). WebKit throttles the page of a window that
/// is not: no animation frames, no painting. Main thread only; `false` off it.
pub fn occlusion_visible(window: &tauri::WebviewWindow) -> bool {
    if MainThreadMarker::new().is_none() {
        return false;
    }
    let Ok(ptr) = window.ns_window() else { return false };
    // SAFETY: the live NSWindow of this Tauri window, on the main thread.
    unsafe { (ptr as *const NSWindow).as_ref() }
        .is_some_and(|ns_window| ns_window.occlusionState().contains(NSWindowOcclusionState::Visible))
}

/// Run `f` on the window's `NSWindow`, on the main thread (AppKit). Every
/// window build and `reveal` already runs there (`show_settings` / `show_aux`
/// hop to it first), so `f` runs in the caller's turn. A caller off the main
/// thread still gets `f` applied, one turn later, never skipped: skipping the
/// alpha-1 step would leave an invisible key window.
fn with_ns_window(window: &tauri::WebviewWindow, f: impl FnOnce(&NSWindow) + Send + 'static) {
    if MainThreadMarker::new().is_some() {
        apply_on_main(window, f);
        return;
    }
    tracing::warn!("NSWindow access off the main thread; dispatched to it");
    let target = window.clone();
    if let Err(error) = window.run_on_main_thread(move || apply_on_main(&target, f)) {
        tracing::warn!(%error, "could not reach the main thread for an NSWindow change");
    }
}

/// `f` on the window's `NSWindow`. Main thread only.
fn apply_on_main(window: &tauri::WebviewWindow, f: impl FnOnce(&NSWindow)) {
    let Ok(ptr) = window.ns_window() else { return };
    // SAFETY: `ns_window()` is the live NSWindow behind this Tauri window, and
    // this only runs on the main thread, where AppKit objects may be used.
    if let Some(ns_window) = unsafe { (ptr as *const NSWindow).as_ref() } {
        f(ns_window);
    }
}

/// Whether the caller is on the main (AppKit) thread.
pub fn on_main_thread() -> bool {
    MainThreadMarker::new().is_some()
}

/// Put a freshly built (hidden) window on screen fully transparent, without
/// making it key: WebKit only loads and renders a page whose window is ordered
/// in (a hidden window runs no animation frames, and a cold one may not even
/// start its navigation), so the page paints its real content behind alpha 0
/// and `reveal_staged` makes that frame visible. While transparent it lets
/// clicks through to whatever is behind it.
///
/// A child window (a modal with a parent) is already ordered in by
/// `addChildWindow:` during the build; this makes it transparent before the
/// run loop gets to composite a frame of it.
pub fn stage_transparent(window: &tauri::WebviewWindow) {
    with_ns_window(window, |ns_window| {
        ns_window.setAlphaValue(0.0);
        ns_window.setIgnoresMouseEvents(true);
        ns_window.orderFront(None);
    });
}

/// Make a staged window opaque and clickable again. The caller then shows and
/// focuses it as before; both happen in the same main-thread turn, so the first
/// composited frame at alpha 1 is the page the window already painted.
pub fn reveal_staged(window: &tauri::WebviewWindow) {
    with_ns_window(window, |ns_window| {
        ns_window.setIgnoresMouseEvents(false);
        ns_window.setAlphaValue(1.0);
    });
}

/// Activate VietKB so its windows come in front of every other app. Runs on the
/// main thread, queued after the activation-policy change that precedes it.
pub fn activate(app: &tauri::AppHandle) {
    let result = app.run_on_main_thread(|| {
        if let Some(mtm) = MainThreadMarker::new() {
            #[allow(deprecated)]
            NSApplication::sharedApplication(mtm).activateIgnoringOtherApps(true);
        }
    });
    if let Err(error) = result {
        tracing::warn!(%error, "could not activate the app");
    }
}

/// Apply `system.theme` to the whole app: the native title bar, menus and the
/// WebView's `prefers-color-scheme` follow it. `System` hands control back to
/// System Settings > Appearance (27-mac-decisions M6).
pub fn apply_theme(app: &tauri::AppHandle, theme: &vkb_config::Theme) {
    app.set_theme(match theme {
        vkb_config::Theme::System => None,
        vkb_config::Theme::Light => Some(tauri::Theme::Light),
        vkb_config::Theme::Dark => Some(tauri::Theme::Dark),
    });
}
