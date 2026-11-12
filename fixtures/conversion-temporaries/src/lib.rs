use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// Track only this fixture's Rust heap, separately from Dart's native scratch.
struct CountingAllocator;
static LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = System.alloc(layout);
        if !pointer.is_null() {
            LIVE_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        System.dealloc(pointer, layout);
        LIVE_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

#[uniffi::export]
pub fn live_bytes() -> u64 {
    LIVE_BYTES.load(Ordering::Relaxed) as u64
}

#[derive(uniffi::Enum)]
pub enum Shade {
    Light,
    Dark,
}

#[derive(uniffi::Enum)]
pub enum Colored {
    Paint { shade: Shade, number: u32 },
}

#[uniffi::export]
pub fn check_colored(value: Colored) -> bool {
    matches!(value, Colored::Paint { shade: Shade::Dark, number: 42 })
}

#[uniffi::export(with_foreign)]
pub trait OptionalText: Send + Sync {
    fn text(&self) -> Option<String>;
}

#[uniffi::export]
pub fn check_optional_text(callback: Arc<dyn OptionalText>, present: bool) -> bool {
    match callback.text() {
        Some(value) => present && value.len() == 64 && value.bytes().all(|b| b == b'x'),
        None => !present,
    }
}

uniffi::include_scaffolding!("api");
