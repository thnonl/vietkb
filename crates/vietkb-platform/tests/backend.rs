use vietkb_core::EngineAction;
use vietkb_platform::{PlatformBackend, RecordedBackend};

#[test]
fn recorded_backend_captures_emitted_actions() {
    let mut backend = RecordedBackend::default();
    backend
        .start(Box::new(|_| EngineAction::Noop))
        .unwrap_or_else(|err| panic!("start failed: {err}"));
    backend
        .emit(EngineAction::Commit("a".to_owned()))
        .unwrap_or_else(|err| panic!("emit failed: {err}"));

    assert!(backend.is_started());
    assert_eq!(
        backend.emitted(),
        &[EngineAction::Noop, EngineAction::Commit("a".to_owned())]
    );

    backend
        .stop()
        .unwrap_or_else(|err| panic!("stop failed: {err}"));
    assert!(!backend.is_started());
}
