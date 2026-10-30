//! The per-key plan step must not touch the heap.
//!
//! `keep_unchanged_head` runs for every key the hook decides. The engine has
//! already built the `Replace` text; trimming it must shorten that string in
//! place, and the snapshot of the on-screen word must fit the inline buffer.
//! Same counting-allocator technique as `vkb-core/tests/no_alloc.rs`.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use vkb_core::charset::TextBuf;
use vkb_core::EngineAction;
use vkb_platform::plan::keep_unchanged_head;

struct Counting;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        System.realloc(ptr, layout, new_size)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        System.alloc_zeroed(layout)
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

fn replace(delete_units: u32, insert: &str) -> EngineAction {
    EngineAction::Replace {
        delete_units,
        insert: insert.to_string(),
    }
}

#[test]
fn the_plan_step_does_not_allocate() {
    let screen: Vec<char> = "Việt".chars().collect();
    // Built before the measurement: the engine owns these strings already.
    let mut batch: Vec<EngineAction> = (0..256)
        .map(|n| match n % 3 {
            0 => replace(4, "Việt!"),
            1 => replace(4, "Vie\u{323}t"),
            _ => EngineAction::Noop,
        })
        .collect();
    batch.reverse();

    // The harness runs tests on parallel threads and may allocate meanwhile, so
    // take the minimum over several attempts (a real allocation repeats).
    let mut lowest = usize::MAX;
    for _ in 0..8 {
        let mut again = batch.clone();
        let before = ALLOCATIONS.load(Ordering::Relaxed);
        while let Some(action) = again.pop() {
            let shown: TextBuf = screen.iter().copied().collect();
            let planned = keep_unchanged_head(&shown, action);
            std::hint::black_box(&planned);
            std::mem::forget(planned);
        }
        lowest = lowest.min(ALLOCATIONS.load(Ordering::Relaxed) - before);
    }
    assert_eq!(lowest, 0, "the per-key plan step allocated");
}
