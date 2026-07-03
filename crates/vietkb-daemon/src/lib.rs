use vietkb_core::{KeyCode, KeyInput, SimpleVietnameseEngine, VietnameseEngine};
use vietkb_platform::{EngineSink, PlatformBackend, PlatformEvent};

pub struct Runtime<B> {
    backend: B,
}

impl<B> Runtime<B>
where
    B: PlatformBackend,
{
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        let mut engine = SimpleVietnameseEngine::new();
        let sink: EngineSink = Box::new(move |event| match event {
            PlatformEvent::KeyText(ch) => engine.process_key(KeyInput::from_char(ch)),
            PlatformEvent::Backspace => engine.process_key(KeyInput::plain(KeyCode::Backspace)),
            PlatformEvent::Space => engine.process_key(KeyInput::plain(KeyCode::Space)),
            PlatformEvent::Enter => engine.process_key(KeyInput::plain(KeyCode::Enter)),
            PlatformEvent::FocusChanged(_) => {
                engine.reset();
                vietkb_core::EngineAction::Noop
            }
        });
        self.backend.start(sink)
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        self.backend.stop()
    }
}
