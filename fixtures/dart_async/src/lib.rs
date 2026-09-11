use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::Duration;

use futures::future::{AbortHandle, Abortable, Aborted};
use once_cell::sync::Lazy;

/// Non-blocking timer future.
pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState { completed: false, waker: None }));

        let thread_shared_state = shared_state.clone();
        // Let's mimic an event coming from somewhere else, like the system.
        thread::spawn(move || {
            thread::sleep(duration);
            let mut shared_state: MutexGuard<_> = thread_shared_state.lock().unwrap();
            shared_state.completed = true;
            if let Some(waker) = shared_state.waker.take() {
                waker.wake();
            }
        });

        Self { shared_state }
    }
}

/// Non-blocking timer future that intentionally misbehaves.
pub struct BrokenTimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

impl Future for BrokenTimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl BrokenTimerFuture {
    pub fn new(duration: Duration, fail_after: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState { completed: false, waker: None }));

        let thread_shared_state = shared_state.clone();
        // Let's mimic an event coming from somewhere else, like the system.
        thread::spawn(move || {
            thread::sleep(duration);
            let mut shared_state: MutexGuard<_> = thread_shared_state.lock().unwrap();
            shared_state.completed = true;
            if let Some(waker) = shared_state.waker.take() {
                // Do not consume `waker`.
                waker.wake_by_ref();
                // And this is the important part. We are going to call `wake()` a second time.
                if fail_after.is_zero() {
                    waker.wake();
                } else {
                    thread::spawn(move || {
                        thread::sleep(fail_after);
                        waker.wake();
                    });
                }
            }
        });

        Self { shared_state }
    }
}

#[uniffi::export]
pub fn greet(who: String) -> String {
    format!("Hello, {who}")
}

/// Async function that is immediately ready. Declared in the UDL to ensure UDL async works.
pub async fn always_ready() -> bool {
    true
}

#[uniffi::export]
pub async fn void() {}

#[uniffi::export]
pub async fn say() -> String {
    TimerFuture::new(Duration::from_secs(2)).await;
    "Hello, Future!".to_string()
}

#[uniffi::export]
pub async fn say_after(ms: u16, who: String) -> String {
    TimerFuture::new(Duration::from_millis(ms.into())).await;
    format!("Hello, {who}!")
}

#[uniffi::export]
pub async fn sleep(ms: u16) -> bool {
    TimerFuture::new(Duration::from_millis(ms.into())).await;
    true
}

#[uniffi::export]
pub async fn sleep_no_return(ms: u16) {
    TimerFuture::new(Duration::from_millis(ms.into())).await;
}

// Our error.
#[derive(thiserror::Error, uniffi::Error, Debug)]
pub enum MyError {
    #[error("Foo")]
    Foo,
}

// An async function that can throw.
#[uniffi::export]
pub async fn fallible_me(do_fail: bool) -> Result<u8, MyError> {
    if do_fail {
        Err(MyError::Foo)
    } else {
        Ok(42)
    }
}

// An async function returning a struct that can throw.
#[uniffi::export]
pub async fn fallible_struct(do_fail: bool) -> Result<Arc<Megaphone>, MyError> {
    if do_fail {
        Err(MyError::Foo)
    } else {
        Ok(new_megaphone())
    }
}

#[derive(uniffi::Record)]
pub struct MyRecord {
    pub a: String,
    pub b: u32,
}

#[derive(uniffi::Enum)]
pub enum AsyncItemState {
    Ready { timestamp_ms: u64 },
    Pending { reason: String },
}

#[derive(uniffi::Record)]
pub struct AsyncItem {
    pub id: u64,
    pub state: AsyncItemState,
}

#[uniffi::export]
pub async fn new_my_record(a: String, b: u32) -> MyRecord {
    MyRecord { a, b }
}

#[uniffi::export]
pub fn list_async_items() -> Vec<AsyncItem> {
    vec![
        AsyncItem { id: 1, state: AsyncItemState::Ready { timestamp_ms: 1111 } },
        AsyncItem { id: 2, state: AsyncItemState::Pending { reason: "syncing".to_string() } },
    ]
}

/// Non-blocking timer future used to test callback cancellation.
#[uniffi::export]
pub async fn broken_sleep(ms: u16, fail_after: u16) {
    BrokenTimerFuture::new(
        Duration::from_millis(ms.into()),
        Duration::from_millis(fail_after.into()),
    )
    .await;
}

/// Proc-macro-defined object with async methods (Megaphone)
#[derive(uniffi::Object)]
pub struct Megaphone;

#[uniffi::export]
impl Megaphone {
    /// Async constructor
    #[uniffi::constructor]
    pub async fn new() -> Arc<Self> {
        TimerFuture::new(Duration::from_millis(0)).await;
        Arc::new(Self)
    }

    /// Alternative async constructor
    #[uniffi::constructor]
    pub async fn secondary() -> Arc<Self> {
        TimerFuture::new(Duration::from_millis(0)).await;
        Arc::new(Self)
    }

    /// Async method that yells something after a certain time
    pub async fn say_after(self: Arc<Self>, ms: u16, who: String) -> String {
        say_after(ms, who).await.to_uppercase()
    }

    /// Async method without any extra arguments
    pub async fn silence(&self) -> String {
        String::new()
    }

    /// Async method that can throw
    pub async fn fallible_me(self: Arc<Self>, do_fail: bool) -> Result<u8, MyError> {
        if do_fail {
            Err(MyError::Foo)
        } else {
            Ok(42)
        }
    }
}

/// Mixed async/sync methods on the same object (using tokio runtime)
#[uniffi::export(async_runtime = "tokio")]
impl Megaphone {
    /// Sync method that yells something immediately
    pub fn say_now(&self, who: String) -> String {
        format!("Hello, {who}!").to_uppercase()
    }

    /// Async method using Tokio's timer
    pub async fn say_after_with_tokio(self: Arc<Self>, ms: u16, who: String) -> String {
        say_after_with_tokio(ms, who).await.to_uppercase()
    }
}

/// Sync function that generates a new `Megaphone`.
#[uniffi::export]
pub fn new_megaphone() -> Arc<Megaphone> {
    Arc::new(Megaphone)
}

/// Async function that generates a new `Megaphone`.
#[uniffi::export]
pub async fn async_new_megaphone() -> Arc<Megaphone> {
    new_megaphone()
}

/// Async function that possibly generates a new `Megaphone`.
#[uniffi::export]
pub async fn async_maybe_new_megaphone(y: bool) -> Option<Arc<Megaphone>> {
    if y {
        Some(new_megaphone())
    } else {
        None
    }
}

/// Async function that inputs `Megaphone`.
#[uniffi::export]
pub async fn say_after_with_megaphone(megaphone: Arc<Megaphone>, ms: u16, who: String) -> String {
    megaphone.say_after(ms, who).await
}

/// Async function that uses tokio runtime.
#[uniffi::export(async_runtime = "tokio")]
pub async fn say_after_with_tokio(ms: u16, who: String) -> String {
    tokio::time::sleep(Duration::from_millis(ms.into())).await;
    format!("Hello, {who} (with Tokio)!")
}

/// Fallible async constructor object
#[derive(uniffi::Object)]
pub struct FallibleMegaphone;

#[uniffi::export]
impl FallibleMegaphone {
    #[uniffi::constructor]
    pub async fn new() -> Result<Arc<Self>, MyError> {
        Err(MyError::Foo)
    }
}

/// Async runtime example that uses shared state to test timeouts.
#[derive(uniffi::Record)]
pub struct SharedResourceOptions {
    pub release_after_ms: u16,
    pub timeout_ms: u16,
}

// Our error for async resource usage.
#[derive(thiserror::Error, uniffi::Error, Debug)]
pub enum AsyncError {
    #[error("Timeout")]
    Timeout,
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn use_shared_resource(options: SharedResourceOptions) -> Result<(), AsyncError> {
    use tokio::sync::Mutex;
    use tokio::time::{sleep, timeout};

    static MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    let _guard = timeout(Duration::from_millis(options.timeout_ms.into()), MUTEX.lock())
        .await
        .map_err(|_| AsyncError::Timeout)?;

    sleep(Duration::from_millis(options.release_after_ms.into())).await;
    Ok(())
}

// Example of a trait with async methods.
#[uniffi::export]
#[async_trait::async_trait]
pub trait SayAfterTrait: Send + Sync {
    async fn say_after(&self, ms: u16, who: String) -> String;
}

// Example of async trait defined in the UDL file.
#[uniffi::trait_interface]
#[async_trait::async_trait]
pub trait SayAfterUdlTrait: Send + Sync {
    async fn say_after(&self, ms: u16, who: String) -> String;
}

struct SayAfterImpl1;
struct SayAfterImpl2;

#[async_trait::async_trait]
impl SayAfterTrait for SayAfterImpl1 {
    async fn say_after(&self, ms: u16, who: String) -> String {
        say_after(ms, who).await
    }
}

#[async_trait::async_trait]
impl SayAfterTrait for SayAfterImpl2 {
    async fn say_after(&self, ms: u16, who: String) -> String {
        say_after(ms, who).await
    }
}

#[uniffi::export]
pub fn get_say_after_traits() -> Vec<Arc<dyn SayAfterTrait>> {
    vec![Arc::new(SayAfterImpl1), Arc::new(SayAfterImpl2)]
}

#[async_trait::async_trait]
impl SayAfterUdlTrait for SayAfterImpl1 {
    async fn say_after(&self, ms: u16, who: String) -> String {
        say_after(ms, who).await
    }
}

#[async_trait::async_trait]
impl SayAfterUdlTrait for SayAfterImpl2 {
    async fn say_after(&self, ms: u16, who: String) -> String {
        say_after(ms, who).await
    }
}

#[uniffi::export]
pub fn get_say_after_udl_traits() -> Vec<Arc<dyn SayAfterUdlTrait>> {
    vec![Arc::new(SayAfterImpl1), Arc::new(SayAfterImpl2)]
}

/// UDL-defined object with async methods.
pub struct UdlMegaphone;

impl UdlMegaphone {
    pub async fn new() -> Self {
        TimerFuture::new(Duration::from_millis(0)).await;
        Self {}
    }

    pub async fn secondary() -> Self {
        TimerFuture::new(Duration::from_millis(0)).await;
        Self {}
    }

    pub async fn say_after(&self, ms: u16, who: String) -> String {
        TimerFuture::new(Duration::from_millis(ms.into())).await;
        format!("Hello, {who} (from UDL Megaphone)!").to_uppercase()
    }
}

// Async callback interface implemented in foreign code.
#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait AsyncParser: Send + Sync {
    // Simple async method
    async fn as_string(&self, delay_ms: i32, value: i32) -> String;
    // Async method that can throw
    async fn try_from_string(&self, delay_ms: i32, value: String) -> Result<i32, ParserError>;
    // Void return, which requires special handling
    async fn delay(&self, delay_ms: i32);
    // Void return that can also throw
    async fn try_delay(&self, delay_ms: String) -> Result<(), ParserError>;
}

#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait AsyncParserMirror: Send + Sync {
    async fn mirror_string(&self, delay_ms: i32, value: i32) -> String;
    async fn mirror_delay(&self, delay_ms: i32);
}

#[derive(thiserror::Error, uniffi::Error, Debug)]
pub enum ParserError {
    #[error("NotAnInt")]
    NotAnInt,
    #[error("UnexpectedError")]
    UnexpectedError,
}

impl From<uniffi::UnexpectedUniFFICallbackError> for ParserError {
    fn from(_: uniffi::UnexpectedUniFFICallbackError) -> Self {
        Self::UnexpectedError
    }
}

#[uniffi::export]
pub async fn as_string_using_trait(obj: Arc<dyn AsyncParser>, delay_ms: i32, value: i32) -> String {
    obj.as_string(delay_ms, value).await
}

#[uniffi::export]
pub async fn try_from_string_using_trait(
    obj: Arc<dyn AsyncParser>,
    delay_ms: i32,
    value: String,
) -> Result<i32, ParserError> {
    obj.try_from_string(delay_ms, value).await
}

#[uniffi::export]
pub async fn delay_using_trait(obj: Arc<dyn AsyncParser>, delay_ms: i32) {
    obj.delay(delay_ms).await
}

#[uniffi::export]
pub async fn try_delay_using_trait(
    obj: Arc<dyn AsyncParser>,
    delay_ms: String,
) -> Result<(), ParserError> {
    obj.try_delay(delay_ms).await
}

#[uniffi::export]
pub async fn cancel_delay_using_trait(obj: Arc<dyn AsyncParser>, delay_ms: i32) {
    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    thread::spawn(move || {
        // Simulate a different thread aborting the process
        thread::sleep(Duration::from_millis(1));
        abort_handle.abort();
    });

    let future = Abortable::new(obj.delay(delay_ms), abort_registration);
    assert_eq!(future.await, Err(Aborted));
}

#[uniffi::export]
pub async fn mirror_string_using_trait(
    obj: Arc<dyn AsyncParserMirror>,
    delay_ms: i32,
    value: i32,
) -> String {
    obj.mirror_string(delay_ms, value).await
}

#[uniffi::export]
pub async fn mirror_delay_using_trait(obj: Arc<dyn AsyncParserMirror>, delay_ms: i32) {
    obj.mirror_delay(delay_ms).await
}

// A bounded primitive result exercises failures after the Dart Future resolves.
#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait AsyncByteSource: Send + Sync {
    async fn value(&self) -> Result<u8, ParserError>;
}

#[uniffi::export]
pub async fn read_async_byte(source: Arc<dyn AsyncByteSource>) -> Result<u8, ParserError> {
    source.value().await
}

uniffi::include_scaffolding!("api");
