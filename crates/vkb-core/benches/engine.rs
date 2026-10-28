//! Engine latency bench (charter P1: p99 ≤ 50 µs, no allocation on the Noop path).
//! Filled in with the state machine in sprint 1; the harness is wired now so the
//! budget is measured from the first commit, not retrofitted.

use criterion::{criterion_group, criterion_main, Criterion};
use vkb_core::{Engine, EngineOptions, KeyInput, VietnameseEngine};

fn keystrokes(c: &mut Criterion) {
    let mut engine = Engine::new(EngineOptions::default());
    let keys: Vec<char> = "tieengs vieetj".chars().collect();

    c.bench_function("process_key/telex_word", |b| {
        b.iter(|| {
            for ch in &keys {
                std::hint::black_box(engine.process_key(KeyInput::char(*ch)));
            }
        })
    });
}

criterion_group!(benches, keystrokes);
criterion_main!(benches);
