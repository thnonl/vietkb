use vietkb_daemon::Runtime;
use vietkb_platform::RecordedBackend;

#[test]
fn runtime_starts_and_stops_backend() {
    let backend = RecordedBackend::default();
    let mut runtime = Runtime::new(backend);
    runtime
        .start()
        .unwrap_or_else(|err| panic!("start failed: {err}"));
    runtime
        .stop()
        .unwrap_or_else(|err| panic!("stop failed: {err}"));
}
