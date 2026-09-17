//! Local experiment: keep generated Dart callbacks on their owner thread.
//! The relay is fixture-specific and is not enabled in the generator.
use std::sync::Arc;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};

mod owner_queue;
mod relay;

static SAVED: Mutex<Option<Arc<dyn Sink>>> = Mutex::new(None);
static BACKGROUND: AtomicU64 = AtomicU64::new(0);

#[uniffi::export]
pub fn call_cross_nested(sink: Arc<dyn Sink>) -> u64 {
    *SAVED.lock().unwrap() = Some(sink.clone());
    let result = sink.bytes(vec![42]);
    SAVED.lock().unwrap().take();
    result
}

#[uniffi::export]
pub fn call_saved_thread(sink: Arc<dyn Sink>) -> u64 {
    let saved = SAVED.lock().unwrap().as_ref().unwrap().clone();
    let result = std::thread::spawn(move || saved.bytes(vec![42])).join().unwrap();
    drop(sink);
    result
}

#[uniffi::export]
pub fn save_sink(sink: Arc<dyn Sink>) -> bool {
    assert!(SAVED.lock().unwrap().replace(sink).is_none());
    true
}

#[uniffi::export]
pub fn fire_saved_background() {
    BACKGROUND.store(0, Ordering::Release);
    let saved = SAVED.lock().unwrap().take().unwrap();
    std::thread::spawn(move || {
        let result = saved.bytes(vec![42]);
        drop(saved);
        BACKGROUND.store(result, Ordering::Release);
    });
}

#[uniffi::export]
pub fn fire_saved_background_clone() {
    BACKGROUND.store(0, Ordering::Release);
    let saved = SAVED.lock().unwrap().take().unwrap();
    std::thread::spawn(move || {
        let result = call_clone_thread(saved);
        BACKGROUND.store(result, Ordering::Release);
    });
}

#[no_mangle]
pub extern "C" fn relay_background_result() -> u64 {
    BACKGROUND.load(Ordering::Acquire)
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum CallbackError {
    #[error("Rejected input")]
    Rejected,
}

#[uniffi::export(with_foreign)]
pub trait Sink: Send + Sync {
    fn bytes(&self, value: Vec<u8>) -> u64;
    fn checked(&self, value: u32) -> Result<u32, CallbackError>;
}

#[uniffi::export]
pub fn call_bytes_direct(sink: Arc<dyn Sink>) -> u64 {
    sink.bytes(vec![42])
}

// The original failure shape: Rust spawns a worker and joins it while that worker
// calls the Dart object. The relay must not bypass this function or its vtable.
#[uniffi::export]
pub fn call_bytes_thread(sink: Arc<dyn Sink>) -> u64 {
    std::thread::spawn(move || sink.bytes(vec![42])).join().unwrap()
}

#[uniffi::export]
pub fn call_bytes_parallel(sink: Arc<dyn Sink>) -> u64 {
    let workers: Vec<_> = (0..8)
        .map(|_| {
            let sink = sink.clone();
            std::thread::spawn(move || (0..10).map(|_| sink.bytes(vec![42])).sum::<u64>())
        })
        .collect();
    workers.into_iter().map(|worker| worker.join().unwrap()).sum()
}

#[uniffi::export]
pub fn call_checked_thread(sink: Arc<dyn Sink>, value: u32) -> Result<u32, CallbackError> {
    std::thread::spawn(move || sink.checked(value)).join().unwrap()
}

#[uniffi::export]
pub fn call_clone_thread(sink: Arc<dyn Sink>) -> u64 {
    std::thread::spawn(move || {
        // Unlike Arc::clone, this exercises the generated foreign handle-clone
        // callback, then re-lifts that handle into a separately owned wrapper.
        let handle = <Arc<dyn Sink> as uniffi::Lower<UniFfiTag>>::lower(sink.clone());
        let clone = <Arc<dyn Sink> as uniffi::Lift<UniFfiTag>>::try_lift(handle).unwrap();
        let result = clone.bytes(vec![42]);
        drop(clone);
        drop(sink);
        result
    })
    .join()
    .unwrap()
}

uniffi::setup_scaffolding!();

#[derive(uniffi::Enum)]
pub enum ArgumentPayload {
    Code { value: u32 },
}
#[uniffi::export]
pub fn call_with_payload(sink: Arc<dyn Sink>, _payload: ArgumentPayload) -> u64 {
    sink.bytes(vec![42])
}
