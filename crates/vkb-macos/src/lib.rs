//! macOS backend: `CGEventTap` keyboard hook, text injection, hotkeys, autostart.
//!
//! Everything here translates between the macOS key space and
//! `vkb_platform::KeyInput` / `EngineAction`; there is no Vietnamese logic in
//! this crate (charter §4). The structure mirrors `vkb-windows`.
//!
//! The pure logic (keycode mapping, event translation, hotkeys, the runtime
//! state machine, injection planning, LaunchAgent plist) compiles and is tested
//! on every platform. The Apple FFI layer and the public backend functions
//! exist only on macOS (decision M1 in `27-mac-decisions.md`).

#![cfg_attr(not(target_os = "macos"), allow(dead_code))]

pub mod autostart;
mod globals;
pub mod hotkeys;
pub mod inject;
pub mod keys;
pub mod sound;
pub mod state;
#[cfg(target_os = "macos")]
mod sys;

/// De-duplicate `(bundle id, name)` pairs by bundle id (the first non-empty
/// name wins) and sort by bundle id.
pub(crate) fn tidy_running(mut apps: Vec<(String, String)>) -> Vec<(String, String)> {
    apps.retain(|(bundle, _)| !bundle.is_empty());
    apps.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.len().cmp(&a.1.len())));
    apps.dedup_by(|a, b| a.0 == b.0);
    apps
}

/// Whether a running app belongs in the exclusion picker. Regular (0) and
/// Accessory (1: menu-bar apps, Spotlight-like panels, Raycast, Alfred) qualify
/// when they have a bundle id and a name; Prohibited (2) background daemons
/// and VietKB itself do not.
pub(crate) fn wants_running_app(policy: isize, bundle: &str, name: &str, is_self: bool) -> bool {
    !is_self && matches!(policy, 0 | 1) && !bundle.is_empty() && !name.trim().is_empty()
}

#[cfg(test)]
mod tidy_tests {
    use super::{tidy_running, wants_running_app};

    #[test]
    fn picker_takes_regular_and_named_accessory_apps_only() {
        assert!(wants_running_app(0, "com.apple.textedit", "TextEdit", false));
        assert!(wants_running_app(1, "com.raycast.macos", "Raycast", false));
        assert!(!wants_running_app(2, "com.apple.daemon", "Daemon", false));
        assert!(!wants_running_app(1, "com.vendor.agent", "  ", false));
        assert!(!wants_running_app(1, "", "Nameless", false));
        assert!(!wants_running_app(0, "vn.vietkb.app", "VietKB", true));
        assert!(!wants_running_app(-1, "x.y", "Y", false));
    }

    #[test]
    fn dedups_sorts_and_drops_empty_bundles() {
        let out = tidy_running(vec![
            ("com.zed".into(), "Zed".into()),
            ("com.apple.safari".into(), String::new()),
            ("".into(), "Ghost".into()),
            ("com.apple.safari".into(), "Safari".into()),
        ]);
        assert_eq!(
            out,
            vec![
                ("com.apple.safari".to_string(), "Safari".to_string()),
                ("com.zed".to_string(), "Zed".to_string()),
            ]
        );
    }
}

/// True when the current OS can host a real backend.
pub const IMPLEMENTED: bool = cfg!(target_os = "macos");

pub use globals::secure_input_blocking;
pub use state::{is_injected, MARKER};

#[cfg(target_os = "macos")]
pub use imp::*;

#[cfg(target_os = "macos")]
mod imp {
    use std::path::PathBuf;

    use anyhow::Result;
    use vkb_core::EngineAction;
    use vkb_platform::plan::{decide, HeldKeys, Injection};
    use vkb_platform::{AppInfo, BackendCommand, EngineSink, PlatformBackend, SharedConfig};

    use crate::{autostart, globals, sys};

    /// Start the `CGEventTap` thread. Fails (never panics) when Accessibility is
    /// not granted or the tap cannot be created. Idempotent when already running.
    pub fn install() -> Result<()> {
        sys::install()
    }

    /// Stop the tap thread. Safe to call when nothing is installed.
    pub fn uninstall_hook() -> Result<()> {
        sys::uninstall();
        Ok(())
    }

    /// `true` while the tap exists and is enabled.
    pub fn is_hook_ready() -> bool {
        sys::is_ready()
    }

    /// Publish the config the tap consults. The tap thread picks it up on its
    /// next event; the keystroke path never touches the lock.
    pub fn set_config(config: SharedConfig) {
        globals::publish_config(config);
    }

    /// Flip Vietnamese on/off. Picked up by the tap on the next keystroke.
    pub fn set_enabled(enabled: bool) {
        globals::store_enabled(enabled);
    }

    pub fn is_enabled() -> bool {
        globals::enabled()
    }

    /// Called on the tap thread when a hotkey toggles the state.
    pub fn set_enabled_callback(cb: impl Fn(bool) + Send + 'static) {
        globals::set_enabled_callback(cb);
    }

    pub fn set_command_callback(cb: impl Fn(BackendCommand) + Send + 'static) {
        globals::set_command_callback(cb);
    }

    /// Create or remove `~/Library/LaunchAgents/vn.vietkb.app.plist`.
    pub fn set_autostart(enabled: bool) -> Result<()> {
        autostart::set_autostart(enabled)
    }

    pub fn is_autostart_enabled() -> bool {
        autostart::is_autostart_enabled()
    }

    /// The app's data directory (channel-aware, owned by the app).
    pub fn set_data_dir(dir: PathBuf) {
        *globals::DATA_DIR.lock() = Some(dir);
    }

    /// `CGPreflightListenEventAccess()`: Input Monitoring granted (no prompt).
    pub fn has_input_monitoring_permission() -> bool {
        sys::has_input_monitoring_permission()
    }

    /// `CGRequestListenEventAccess()`: shows the system prompt; returns the
    /// current state, the user answers asynchronously.
    pub fn request_input_monitoring_permission() -> bool {
        sys::request_input_monitoring_permission()
    }

    /// `AXIsProcessTrusted()`.
    pub fn has_accessibility_permission() -> bool {
        sys::has_accessibility_permission()
    }

    /// `AXIsProcessTrustedWithOptions` with the system prompt. Returns the
    /// current trust state; the user answers asynchronously.
    pub fn request_accessibility_permission() -> bool {
        sys::request_accessibility_permission()
    }

    /// Lowercased bundle identifier of the frontmost application.
    pub fn active_app_bundle() -> Option<String> {
        sys::active_app_bundle()
    }

    pub fn active_app() -> Option<AppInfo> {
        sys::active_app_bundle().map(|bundle| AppInfo {
            process_name: bundle,
            exe_path: None,
            window_title: None,
        })
    }

    /// Regular (Dock) and Accessory (menu-bar / panel) apps currently running: `process_name` is the lowercased
    /// bundle id (the key per-app rules use), `window_title` the localized name.
    /// De-duplicated by bundle id and sorted by it.
    pub fn running_apps() -> Vec<AppInfo> {
        crate::tidy_running(sys::running_apps())
            .into_iter()
            .map(|(bundle, name)| AppInfo {
                process_name: bundle,
                exe_path: None,
                window_title: Some(name).filter(|name| !name.is_empty()),
            })
            .collect()
    }

    /// `true` while secure event input is on (password fields): the tap passes
    /// every key through.
    pub fn secure_input_enabled() -> bool {
        sys::secure_input_enabled()
    }

    /// Ready-made sink: posts the engine's actions as tagged `CGEvent`s.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct Injector;

    impl EngineSink for Injector {
        fn emit_with(&mut self, action: EngineAction, _held: HeldKeys) -> Result<()> {
            // The action is already final: `ch` is irrelevant here.
            match decide(&action, None).injection {
                Injection::None => {}
                Injection::Beep => crate::sound::play_error(),
                Injection::Type(text) => sys::send_replace(0, &text),
                Injection::Replace { deletes, insert } => sys::send_replace(deletes, &insert),
            }
            Ok(())
        }

        fn active_app(&self) -> Option<AppInfo> {
            active_app()
        }
    }

    /// `PlatformBackend` for macOS.
    #[derive(Default)]
    pub struct MacBackend {
        config: Option<SharedConfig>,
    }

    impl MacBackend {
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl PlatformBackend for MacBackend {
        fn start(&mut self, sink: Box<dyn EngineSink>) -> Result<()> {
            if let Some(config) = self.config.clone() {
                globals::publish_config(config);
            }
            *globals::SINK.lock() = Some(sink);
            sys::install()
        }

        fn stop(&mut self) -> Result<()> {
            sys::uninstall();
            Ok(())
        }

        fn set_config(&mut self, config: SharedConfig) {
            self.config = Some(config.clone());
            globals::publish_config(config);
        }

        fn is_running(&self) -> bool {
            sys::is_ready()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn backend_reports_not_running_before_start() {
            assert!(!MacBackend::new().is_running());
            assert!(!is_hook_ready());
        }

        #[test]
        fn uninstall_without_install_is_a_no_op() {
            assert!(uninstall_hook().is_ok());
            assert!(uninstall_hook().is_ok());
        }

        #[test]
        fn enabled_flag_round_trips() {
            let _guard = crate::globals::TEST_LOCK.lock();
            set_enabled(false);
            assert!(!is_enabled());
            set_enabled(true);
            assert!(is_enabled());
        }

        #[test]
        fn read_only_probes_do_not_panic() {
            // Read-only OS queries: no prompt, no tap, no events posted.
            let _ = has_accessibility_permission();
            let _ = has_input_monitoring_permission();
            let _ = secure_input_enabled();
            let _ = active_app_bundle();
        }

        #[test]
        fn sink_ignores_noop_actions() {
            let mut injector = Injector;
            assert!(injector
                .emit_with(EngineAction::Noop, HeldKeys::default())
                .is_ok());
            assert!(injector
                .emit_with(EngineAction::Swallow, HeldKeys::default())
                .is_ok());
        }
    }
}
