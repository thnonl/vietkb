use vietkb_core::EngineAction;

pub type EngineSink = Box<dyn FnMut(PlatformEvent) -> EngineAction + Send>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformEvent {
    KeyText(char),
    Backspace,
    Space,
    Enter,
    FocusChanged(Option<AppInfo>),
}

pub trait PlatformBackend {
    fn start(&mut self, sink: EngineSink) -> anyhow::Result<()>;
    fn stop(&mut self) -> anyhow::Result<()>;
    fn active_app(&self) -> Option<AppInfo>;
    fn emit(&mut self, action: EngineAction) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RecordedBackend {
    active_app: Option<AppInfo>,
    emitted: Vec<EngineAction>,
    started: bool,
}

impl RecordedBackend {
    pub fn emitted(&self) -> &[EngineAction] {
        &self.emitted
    }

    pub fn is_started(&self) -> bool {
        self.started
    }
}

impl PlatformBackend for RecordedBackend {
    fn start(&mut self, mut sink: EngineSink) -> anyhow::Result<()> {
        self.started = true;
        let action = sink(PlatformEvent::FocusChanged(self.active_app.clone()));
        self.emit(action)
    }

    fn stop(&mut self) -> anyhow::Result<()> {
        self.started = false;
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
