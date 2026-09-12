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

static LIVE_OBJECTS: AtomicUsize = AtomicUsize::new(0);
#[uniffi::export]
pub fn live_objects() -> u64 {
    LIVE_OBJECTS.load(Ordering::Relaxed) as u64
}

#[derive(uniffi::Object)]
pub struct Tracked;
impl Drop for Tracked {
    fn drop(&mut self) {
        LIVE_OBJECTS.fetch_sub(1, Ordering::Relaxed);
    }
}

#[uniffi::export]
impl Tracked {
    #[uniffi::constructor]
    pub fn new(payload: String, small: u8) -> Arc<Self> {
        let _ = (payload, small);
        LIVE_OBJECTS.fetch_add(1, Ordering::Relaxed);
        Arc::new(Self)
    }
    #[uniffi::constructor]
    pub async fn create(payload: String, small: u8) -> Arc<Self> {
        Self::new(payload, small)
    }
    pub fn accept_two(&self, first: String, second: String) -> u64 {
        (first.len() + second.len()) as u64
    }
    pub fn references(self: Arc<Self>) -> u64 {
        Arc::strong_count(&self) as u64
    }
    pub fn accept(&self, payload: String, small: u8) -> u64 {
        take(payload, small)
    }
    pub async fn accept_async(&self, payload: String, small: u8) -> u64 {
        take(payload, small)
    }
    pub fn as_rust_trait(self: Arc<Self>) -> Arc<dyn RustConsumer> {
        self
    }
    pub fn as_foreign_trait(self: Arc<Self>) -> Arc<dyn Consumer> {
        self
    }
}

#[derive(uniffi::Record)]
pub struct Packet {
    pub objects: Vec<Arc<Tracked>>,
    pub small: u8,
}

#[uniffi::export]
pub fn take(payload: String, small: u8) -> u64 {
    payload.len() as u64 + small as u64
}
#[uniffi::export]
pub fn take_void(payload: String, small: u8) {
    let _ = take(payload, small);
}
#[uniffi::export]
pub async fn take_async(payload: String, small: u8) -> u64 {
    take(payload, small)
}
#[uniffi::export]
pub fn take_packet(packet: Packet, small: u8) -> u64 {
    packet.objects.len() as u64 + small as u64
}
#[uniffi::export]
pub fn take_object(object: Arc<Tracked>, small: u8) -> u64 {
    let _ = object;
    small as u64
}

#[uniffi::export]
pub trait RustConsumer: Send + Sync {
    fn accept(&self, payload: String, small: u8) -> u64;
}
impl RustConsumer for Tracked {
    fn accept(&self, payload: String, small: u8) -> u64 {
        take(payload, small)
    }
}
#[uniffi::export(with_foreign)]
pub trait Consumer: Send + Sync {
    fn accept(&self, payload: String, small: u8) -> u64;
}
impl Consumer for Tracked {
    fn accept(&self, payload: String, small: u8) -> u64 {
        take(payload, small)
    }
}
#[uniffi::export]
pub fn take_callback(consumer: Arc<dyn Consumer>, small: u8) -> u64 {
    consumer.accept(String::new(), small)
}
#[uniffi::export]
pub fn colliding_names(uniffi_arguments: String, uniffi_arg0: u8, uniffi_arg01: u8) -> u64 {
    take(uniffi_arguments, uniffi_arg0) + uniffi_arg01 as u64
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum Rejection {
    #[error("rejected")]
    Rejected,
}
#[uniffi::export]
pub fn reject(object: Arc<Tracked>, payload: String) -> Result<u64, Rejection> {
    let _ = (object, payload);
    Err(Rejection::Rejected)
}
#[uniffi::export]
pub async fn reject_async(object: Arc<Tracked>, payload: String) -> Result<u64, Rejection> {
    reject(object, payload)
}

uniffi::include_scaffolding!("api");
