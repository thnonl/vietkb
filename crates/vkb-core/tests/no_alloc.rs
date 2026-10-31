//! Charter P1: `process_key` must not allocate on the `Noop` path.
//!
//! A counting allocator wraps the system one. The counter is per thread, so
//! tests running in parallel threads of this binary cannot count each other's
//! allocations (D-TEST-01). The check still takes the *minimum* over several
//! attempts as a belt-and-braces guard; a path that really allocates allocates
//! on every attempt, so the minimum still catches it.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use vkb_core::{Engine, EngineOptions, InputMethod, KeyInput, VietnameseEngine};

struct Counting;

thread_local! {
    // `const` init and no destructor: first access never allocates or registers
    // a TLS destructor, so touching it from inside the allocator is safe.
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

fn bump() {
    // `try_with`: tolerate access during thread teardown.
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

fn allocations<F: FnMut()>(mut body: F) -> usize {
    let before = count();
    body();
    count() - before
}

fn minimum_over<F: FnMut()>(attempts: usize, mut body: F) -> usize {
    let mut worst = usize::MAX;
    for _ in 0..attempts {
        worst = worst.min(allocations(&mut body));
    }
    worst
}

#[test]
fn ignored_keys_do_not_allocate() {
    let mut engine = Engine::new(EngineOptions::default());
    // Warm up: nothing one-time may land inside the measurement.
    for _ in 0..4 {
        engine.process_key(KeyInput::char('a'));
        engine.reset();
    }

    let lowest = minimum_over(8, || {
        for _ in 0..64 {
            let mut key_up = KeyInput::char('a');
            key_up.down = false;
            assert!(engine.process_key(key_up).is_noop());

            let mut chord = KeyInput::char('s');
            chord.modifiers.ctrl = true;
            assert!(engine.process_key(chord).is_noop());

            let mut alt = KeyInput::char('a');
            alt.modifiers.alt = true;
            assert!(engine.process_key(alt).is_noop());

            // A non-text key (arrow up) carries no character.
            assert!(engine.process_key(KeyInput::key(0x26, None)).is_noop());
        }
    });
    assert_eq!(lowest, 0, "process_key allocated on the Noop path");
}

#[test]
fn plain_append_and_boundary_do_not_allocate() {
    let mut engine = Engine::new(EngineOptions::default());
    let lowest = minimum_over(8, || {
        engine.reset();
        // `t`, `a`, `n` only append; the space then ends the word.
        assert!(engine.process_key(KeyInput::char('t')).is_noop());
        assert!(engine.process_key(KeyInput::char('a')).is_noop());
        assert!(engine.process_key(KeyInput::char('n')).is_noop());
        assert!(engine.process_key(KeyInput::space()).is_noop());
    });
    assert_eq!(lowest, 0, "the steady path allocated");
}

#[test]
fn vni_strip_that_changes_nothing_does_not_allocate() {
    let mut engine = Engine::new(EngineOptions {
        method: InputMethod::Vni,
        ..EngineOptions::default()
    });
    // Only the `0` key is measured; the setup keys carry text and allocate.
    let mut lowest = usize::MAX;
    for _ in 0..8 {
        engine.reset();
        engine.process_key(KeyInput::char('a'));
        engine.process_key(KeyInput::char('6'));
        // `0` after a mark but with no tone to remove renders the same word
        // (a bare `a0` is a literal digit now, D-TN-03).
        let before = count();
        let action = engine.process_key(KeyInput::char('0'));
        let used = count() - before;
        assert!(matches!(action, vkb_core::EngineAction::Swallow));
        lowest = lowest.min(used);
    }
    assert_eq!(lowest, 0, "the Swallow path allocated");
}

#[test]
fn committing_does_allocate_only_on_the_action_string() {
    // Sanity check that the counter is live: an action that carries text must
    // register exactly one allocation (the `String` it hands to the backend),
    // and no more. `minimum_over` keeps the harness's own threads out of it.
    let mut engine = Engine::new(EngineOptions::default());
    let lowest = minimum_over(5, || {
        engine.reset();
        engine.process_key(KeyInput::char('a'));
        let action = engine.process_key(KeyInput::char('a'));
        std::hint::black_box(action);
    });
    assert_eq!(lowest, 1, "Replace must allocate the insert string once");
}
