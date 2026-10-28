mod hook;
mod system;

use vietkb_core::EngineAction;
use vietkb_platform::{AppInfo, EngineSink, PlatformBackend};

pub use hook::{
    is_keyboard_enabled, run_keyboard_loop, set_command_callback, set_enabled_changed_callback,
    set_keyboard_config, KeyboardCommand,
};
pub use system::{play_notification_sound, set_start_with_windows};

#[derive(Debug, Default)]
pub struct WindowsBackend {
    active_app: Option<AppInfo>,
    emitted: Vec<EngineAction>,
    running: bool,
}

impl WindowsBackend {
    pub fn emitted(&self) -> &[EngineAction] {
        &self.emitted
    }
}

impl PlatformBackend for WindowsBackend {
    fn start(&mut self, _sink: EngineSink) -> anyhow::Result<()> {
        self.running = true;
        Ok(())
    }

    fn stop(&mut self) -> anyhow::Result<()> {
        self.running = false;
        Ok(())
    }

    fn active_app(&self) -> Option<AppInfo> {
        self.active_app.clone()
    }

    fn emit(&mut self, action: EngineAction) -> anyhow::Result<()> {
        self.emitted.push(action);
        Ok(())
    }
}
