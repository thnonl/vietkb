//! Soak test for the long technology passage: one engine, the whole passage
//! 1,000 times. Nothing may leak or drift.
//!
//! - allocation: a key whose action carries no text allocates nothing; a key
//!   whose action carries text allocates only that action's `String`;
//! - bounded state: the capacity of every buffer the engine owns after loop 1000
//!   equals the capacity after loop 1, and none has spilled to the heap;
//! - no drift: the text produced is identical on every loop.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use vkb_core::{apply_action, AppKind, Engine, EngineAction, EngineOptions, InputMethod, KeyInput, VietnameseEngine};

struct Counting;

thread_local! {
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

fn bump() {
    let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
}

fn count() -> usize {
    ALLOCATIONS.with(Cell::get)
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        bump();
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        bump();
        System.realloc(ptr, layout, new_size)
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        bump();
        System.alloc_zeroed(layout)
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

const LOOPS: usize = 1000;

struct Pass {
    /// Text on screen after every paragraph.
    output: Vec<String>,
    /// Keys whose action carried no text but allocated anyway.
    leaking_keys: usize,
    /// Keys whose action carried text and allocated more than its `String`.
    heavy_keys: usize,
    /// Every allocation the engine made during the pass.
    total: usize,
}

fn pass(engine: &mut Engine, keys: &str, screen: &mut String) -> Pass {
    let mut out = Pass { output: Vec::new(), leaking_keys: 0, heavy_keys: 0, total: 0 };
    for line in keys.lines() {
        screen.clear();
        for ch in line.chars().chain(std::iter::once('\r')) {
            let key = if ch == '\r' { KeyInput::enter() } else { KeyInput::char(ch) };
            let before = count();
            let action = engine.process_key(key);
            let used = count() - before;
            out.total += used;
            match &action {
                EngineAction::Noop | EngineAction::Swallow | EngineAction::Reject | EngineAction::ToggleEnabled => {
                    if used != 0 {
                        out.leaking_keys += 1;
                    }
                }
                _ => {
                    // The action's own `String` (a few growth steps for multi-byte text).
                    if used > 3 {
                        out.heavy_keys += 1;
                    }
                }
            }
            apply_action(screen, action, key);
        }
        out.output.push(screen.trim_end_matches(['\r', '\n']).to_string());
    }
    out
}

fn soak(method: InputMethod, app: AppKind, keys: &str) {
    let mut engine = Engine::new(EngineOptions { method, app_kind: app, ..EngineOptions::default() });
    let mut screen = String::with_capacity(4096);
    let first = pass(&mut engine, keys, &mut screen);
    let (caps_first, spilled_first) = engine.buffer_footprint();
    assert!(!spilled_first, "an engine buffer spilled to the heap in loop 1");
    assert_eq!(first.leaking_keys, 0, "a key without text allocated (loop 1)");
    assert_eq!(first.heavy_keys, 0, "a key allocated more than its action string (loop 1)");

    let mut leaking = 0;
    let mut heavy = 0;
    let mut second_total: Option<usize> = None;
    for round in 2..=LOOPS {
        let again = pass(&mut engine, keys, &mut screen);
        leaking += again.leaking_keys;
        heavy += again.heavy_keys;
        assert_eq!(again.output, first.output, "output drifted in loop {round}");
        assert_eq!(again.total, second_total.unwrap_or(again.total), "allocations per loop changed in loop {round}");
        second_total = Some(again.total);
    }
    let (caps_last, spilled_last) = engine.buffer_footprint();
    assert_eq!(leaking, 0, "keys without text allocated on the steady path");
    assert_eq!(heavy, 0, "keys allocated more than their action string");
    assert_eq!(caps_first, caps_last, "engine buffer capacities changed over {LOOPS} loops");
    assert!(!spilled_last, "an engine buffer spilled to the heap");
}

const TELEX: &str = include_str!("data/tech_mixed_vi.txt");
const VNI: &str = include_str!("data/tech_mixed_vi_vni.txt");

#[test]
fn soak_telex_neutral() {
    soak(InputMethod::Telex, AppKind::Neutral, TELEX);
}

#[test]
fn soak_telex_code_app() {
    soak(InputMethod::Telex, AppKind::Code, TELEX);
}

#[test]
fn soak_vni_neutral() {
    soak(InputMethod::Vni, AppKind::Neutral, VNI);
}

#[test]
fn soak_vni_code_app() {
    soak(InputMethod::Vni, AppKind::Code, VNI);
}
