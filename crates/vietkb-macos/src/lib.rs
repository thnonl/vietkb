use vietkb_core::EngineAction;
use vietkb_platform::{AppInfo, EngineSink, PlatformBackend};

#[derive(Debug, Default)]
pub struct MacosBackend {
    active_app: Option<AppInfo>,
    emitted: Vec<EngineAction>,
    running: bool,
}

impl MacosBackend {
    pub fn emitted(&self) -> &[EngineAction] {
        &self.emitted
    }
}

impl PlatformBackend for MacosBackend {
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
