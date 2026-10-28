//! Charter P1 bench: `process_key` latency, criterion flavour.
//!
//! p99 and the allocation check live in `vkb bench` and `tests/no_alloc.rs`;
//! this harness gives the steady-state mean/median that criterion is good at.

use criterion::{criterion_group, criterion_main, Criterion};
use vkb_core::{Engine, EngineOptions, InputMethod, KeyInput, VietnameseEngine};

fn type_word(engine: &mut Engine, word: &str) {
    for ch in word.chars() {
        std::hint::black_box(engine.process_key(KeyInput::char(ch)));
    }
    engine.reset();
}

fn engine_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("process_key");

    group.bench_function("telex_word", |b| {
        let mut engine = Engine::new(EngineOptions::default());
        b.iter(|| type_word(&mut engine, "tieengs"));
    });

    group.bench_function("telex_long_word", |b| {
        let mut engine = Engine::new(EngineOptions::default());
        b.iter(|| type_word(&mut engine, "nghieengx"));
    });

    group.bench_function("telex_rewrite_every_key", |b| {
        // Every key rewrites the word (`nghiêng` keeps re-placing the tone).
        let mut engine = Engine::new(EngineOptions::default());
        b.iter(|| {
            for ch in "nghiengs".chars() {
                engine.process_key(KeyInput::char(ch));
                engine.reset();
            }
        });
    });

    group.bench_function("vni_word", |b| {
        let mut engine = Engine::new(EngineOptions {
            method: InputMethod::Vni,
            ..EngineOptions::default()
        });
        b.iter(|| type_word(&mut engine, "tie6ng1"));
    });

    group.bench_function("noop_chord", |b| {
        let mut engine = Engine::new(EngineOptions::default());
        let mut key = KeyInput::char('s');
        key.modifiers.ctrl = true;
        b.iter(|| {
            std::hint::black_box(engine.process_key(key));
        });
    });

    group.bench_function("boundary_commit", |b| {
        let mut engine = Engine::new(EngineOptions::default());
        b.iter(|| {
            type_word(&mut engine, "tieengs");
            std::hint::black_box(engine.process_key(KeyInput::space()));
        });
    });

    group.finish();
}

criterion_group!(benches, engine_latency);
criterion_main!(benches);
