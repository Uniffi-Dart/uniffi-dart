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

#[derive(uniffi::Record)]
pub struct Packet {
    pub bytes: Vec<u8>,
    pub groups: Vec<Vec<u8>>,
    pub optional: Option<Vec<u8>>,
}

#[uniffi::export]
pub fn make_packet() -> Packet {
    Packet {
        bytes: vec![42; 64],
        groups: vec![vec![11; 8], vec![22; 16]],
        optional: Some(vec![33; 4]),
    }
}

#[uniffi::export]
pub fn make_bytes() -> Vec<u8> {
    vec![42; 64]
}

#[uniffi::export]
pub fn make_string() -> String {
    "x".repeat(64)
}

#[uniffi::export]
pub async fn make_packet_async() -> Packet {
    make_packet()
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum OwnershipError {
    #[error("rejected")]
    Rejected { payload: Vec<u8> },
}

#[uniffi::export]
pub fn fail_result() -> Result<String, OwnershipError> {
    Err(OwnershipError::Rejected { payload: vec![7; 32] })
}

#[uniffi::export]
pub async fn fail_async() -> Result<String, OwnershipError> {
    fail_result()
}

#[uniffi::export]
pub fn panic_result() -> String {
    panic!("ownership probe");
}

#[uniffi::export(with_foreign)]
pub trait BufferConsumer: Send + Sync {
    fn consume(&self, first: Vec<u8>, second: Vec<u8>) -> u64;
}

#[uniffi::export]
pub fn send_buffers(consumer: Arc<dyn BufferConsumer>) -> u64 {
    consumer.consume(vec![4; 64], vec![5; 32])
}

#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait AsyncBufferConsumer: Send + Sync {
    async fn consume(&self, first: Vec<u8>, second: Vec<u8>) -> u64;
}

#[uniffi::export]
pub async fn send_buffers_async(consumer: Arc<dyn AsyncBufferConsumer>) -> u64 {
    consumer.consume(vec![4; 64], vec![5; 32]).await
}

uniffi::include_scaffolding!("api");
