//! Shared OS-backend contract. No OS code lives here.

#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::sync::Arc;

pub use vkb_core::{EngineAction, KeyInput, Modifiers};

// Pure helpers shared by every backend. Additive: the frozen items above are
// unchanged (contract §4).
pub mod clipboard_plan;
pub mod hotkeys;
pub mod inject_strategy;
pub mod plan;
pub mod rules;

pub type SharedConfig = Arc<parking_lot::RwLock<vkb_config::AppConfig>>;

/// Which program owns the caret right now.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AppInfo {
    /// Lowercased executable file name, e.g. `chrome.exe`.
    pub process_name: String,
    pub exe_path: Option<PathBuf>,
    pub window_title: Option<String>,
}

impl AppInfo {
    pub fn matches(&self, rule: &vkb_config::AppRule) -> bool {
        self.process_name.eq_ignore_ascii_case(&rule.process)
    }
}

/// The backend calls this for every engine decision.
pub trait EngineSink: Send {
    /// Apply an action to the live target window, with the Shift keys and Caps
    /// Lock the user holds at this event so the injection leaves them exactly as
    /// it found them. There is deliberately no `held`-less variant: a caller
    /// cannot drop the held keys without a compile error. A sink that does not
    /// care ignores `held`.
    fn emit_with(&mut self, action: EngineAction, held: plan::HeldKeys) -> anyhow::Result<()>;
    fn active_app(&self) -> Option<AppInfo>;
}

/// A platform input backend (Windows hook, macOS event tap, ...).
pub trait PlatformBackend: Send {
    fn start(&mut self, sink: Box<dyn EngineSink>) -> anyhow::Result<()>;
    fn stop(&mut self) -> anyhow::Result<()>;
    /// Replace the config the hook consults on every keystroke.
    fn set_config(&mut self, config: SharedConfig);
    fn is_running(&self) -> bool;
}

/// Called on the hotkey thread; must not block.
pub type HotkeyCallback = Box<dyn Fn(Hotkey) + Send + 'static>;

/// Hotkeys a backend can fire.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hotkey {
    Enable,
    Disable,
    Toggle,
    RestoreWord,
    AppExclude,
    MacroPanel,
    MacroToggle,
    OpenSettings,
    /// `Đổi kiểu gõ` — cycle the active input method.
    CycleMethod,
}

/// Everything the app must be told about, pulled from the backend by callback.
pub type EnabledCallback = Box<dyn Fn(bool) + Send + 'static>;
pub type CommandCallback = Box<dyn Fn(BackendCommand) + Send + 'static>;

/// Commands the hook raises that only the app can service (windows, tray, files).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackendCommand {
    Enable(bool),
    RememberApp { process: String, enabled: bool },
    SetMethod(vkb_core::InputMethod),
    RestoreWord,
    ToggleAppExclude,
    OpenMacroPanel,
    ToggleMacro,
    OpenSettings,
}
