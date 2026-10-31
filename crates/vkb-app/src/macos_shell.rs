//! macOS app activation. Tauri's `set_focus` only orders a window to the front
//! inside its own app; an Accessory app that is not active still sits behind
//! the frontmost app, so the app itself has to be activated.

use objc2::MainThreadMarker;
use objc2_app_kit::NSApplication;

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
