//! `PlatformBackend` / `EngineSink` implementations for the Tauri layer.
//!
//! Two injection owners are possible:
//! * the hook injects by itself (default, no sink supplied), or
//! * the app hands in an [`Injector`] through [`WindowsBackend::start`], in which
//!   case every engine action the hook decides is routed to that sink.

use anyhow::Result;
use vkb_core::EngineAction;
use vkb_platform::plan::{decide, Injection};
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
    fn emit(&mut self, action: EngineAction) -> Result<()> {
        // The action is already final: `ch` is irrelevant here.
        let decision = decide(&action, None);
        match decision.injection {
            Injection::None => {}
            Injection::Beep => sound::play_toggle(true),
            Injection::Type(text) => self.type_text(&text, 0),
            Injection::Replace { deletes, insert } => self.type_text(&insert, deletes),
        }
        Ok(())
    }

    fn active_app(&self) -> Option<AppInfo> {
        foreground::active_app()
    }
}

impl Injector {
    fn type_text(&self, text: &str, deletes: u32) {
        if self.clipboard {
            if deletes > 0 {
                let inputs = input::build_segment(deletes, "", self.mode);
                input::send(&inputs);
            }
            if !text.is_empty() {
                crate::clipboard::paste_async(text.to_string());
            }
            return;
        }
        let inputs = input::build_segment(deletes, text, self.mode);
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
        assert!(injector.emit(EngineAction::Noop).is_ok());
        assert!(injector.emit(EngineAction::Swallow).is_ok());
    }
}
