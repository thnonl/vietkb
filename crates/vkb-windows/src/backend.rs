//! `PlatformBackend` / `EngineSink` implementations for the Tauri layer.
//!
//! Two injection owners are possible:
//! * the hook injects by itself (default, no sink supplied), or
//! * the app hands in an [`Injector`] through [`WindowsBackend::start`], in which
//!   case every engine action the hook decides is routed to that sink.

use anyhow::Result;
use vkb_core::EngineAction;
use vkb_platform::plan::{decide, HeldKeys, Injection};
use vkb_platform::{AppInfo, EngineSink, PlatformBackend, SharedConfig};

use crate::foreground;
use crate::globals;
use crate::hook;
use crate::input::{self, SendMode};
use crate::sound;

/// Ready-made sink: `SendInput` (optionally the clipboard fallback).
#[derive(Clone, Copy, Debug)]
pub struct Injector {
    mode: SendMode,
    clipboard: bool,
}

impl Default for Injector {
    fn default() -> Self {
        Self {
            mode: SendMode::Unicode,
            clipboard: false,
        }
    }
}

impl Injector {
    pub fn new(mode: SendMode) -> Self {
        Self {
            mode,
            clipboard: false,
        }
    }

    /// Opt-in clipboard fallback; off by default (charter C4).
    pub fn with_clipboard(mut self, clipboard: bool) -> Self {
        self.clipboard = clipboard;
        self
    }
}

impl EngineSink for Injector {
    fn emit_with(&mut self, action: EngineAction, held: HeldKeys) -> Result<()> {
        // The action is already final: `ch` is irrelevant here.
        let decision = decide(&action, None);
        match decision.injection {
            Injection::None => {}
            // Not a toggle: `Beep` means "invalid keystroke", so it keeps the OS
            // error alias and never the Vietnamese-on cue (`22-sound-spec.md`
            // §5; the ON/OFF symmetry covers the three toggle routes).
            Injection::Beep => sound::play_error(),
            Injection::Type(text) => self.type_text(&text, 0, held),
            Injection::Replace { deletes, insert } => self.type_text(&insert, deletes, held),
        }
        Ok(())
    }

    fn active_app(&self) -> Option<AppInfo> {
        foreground::active_app()
    }
}

impl Injector {
    fn type_text(&self, text: &str, deletes: u32, held: HeldKeys) {
        if self.clipboard {
            if text.is_empty() {
                if deletes > 0 {
                    let inputs = input::build_segment(deletes, "", self.mode);
                    input::send(&inputs);
                }
            } else {
                crate::clipboard::replace(deletes, text, self.mode, vkb_platform::Modifiers::NONE, held);
            }
            return;
        }
        let inputs = input::build_segment_held(deletes, text, self.mode, held);
        // Deliberately one `SendInput` call (no `input::send_split`): this sink
        // may run off the hook thread, where a physical key could land between
        // a delete call and a text call. The hook path (`state.rs`) splits.
        input::send(&inputs);
    }
}

/// `PlatformBackend` for Windows.
#[derive(Default)]
pub struct WindowsBackend {
    config: Option<SharedConfig>,
}

impl WindowsBackend {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PlatformBackend for WindowsBackend {
    fn start(&mut self, sink: Box<dyn EngineSink>) -> Result<()> {
        if let Some(config) = self.config.clone() {
            *globals::CONFIG.lock() = Some(config);
        }
        *globals::SINK.lock() = Some(sink);
        hook::install()
    }

    fn stop(&mut self) -> Result<()> {
        hook::uninstall();
        Ok(())
    }

    fn set_config(&mut self, config: SharedConfig) {
        self.config = Some(config.clone());
        *globals::CONFIG.lock() = Some(config);
        hook::notify_config_changed();
    }

    fn is_running(&self) -> bool {
        hook::is_ready()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vkb_platform::PlatformBackend;

    #[test]
    fn injector_defaults_are_the_safe_path() {
        let injector = Injector::default();
        assert_eq!(injector.mode, SendMode::Unicode);
        assert!(!injector.clipboard, "clipboard must be opt-in");
    }

    #[test]
    fn backend_reports_not_running_before_start() {
        let backend = WindowsBackend::new();
        assert!(!backend.is_running());
    }

    #[test]
    fn injector_ignores_noop_actions() {
        let mut injector = Injector::default();
        assert!(injector.emit_with(EngineAction::Noop, HeldKeys::default()).is_ok());
        assert!(injector.emit_with(EngineAction::Swallow, HeldKeys::default()).is_ok());
    }
}
