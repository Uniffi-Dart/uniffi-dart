use uniffi;

use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex, MutexGuard},
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

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
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

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

/// Non-blocking timer future.
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
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        let thread_shared_state = shared_state.clone();

        // Let's mimic an event coming from somewhere else, like the system.
        thread::spawn(move || {
            thread::sleep(duration);

            let mut shared_state: MutexGuard<_> = thread_shared_state.lock().unwrap();
            shared_state.completed = true;

            if let Some(waker) = shared_state.waker.take() {
                // Do not consume `waker`.
                waker.wake_by_ref();

                // And this is the important part. We are going to call
                // `wake()` a second time. That's incorrect, but that's on
                // purpose, to see how foreign languages will react.
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

#[uniffi::export]
pub async fn always_ready() -> bool {
    true
}

#[uniffi::export]
pub async fn void_function() {}

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

#[uniffi::export(async_runtime = "tokio")]
pub async fn say_after_with_tokio(ms: u16, who: String) -> String {
    tokio::time::sleep(Duration::from_millis(ms.into())).await;
    format!("Hello, {who} (with Tokio)!")
}

#[derive(uniffi::Record)]
pub struct MyRecord {
    pub a: String,
    pub b: u32,
}

#[uniffi::export]
pub async fn new_my_record(a: String, b: u32) -> MyRecord {
    MyRecord { a, b }
}

#[uniffi::export]
pub async fn broken_sleep(ms: u16, fail_after: u16) {
    BrokenTimerFuture::new(
        Duration::from_millis(ms.into()),
        Duration::from_millis(fail_after.into()),
    )
    .await;
}

// UDL-defined async function
pub async fn udl_always_ready() -> bool {
    true
}

// UDL-defined async trait
pub trait SayAfterUdlTrait: Send + Sync {
    async fn say_after(&self, ms: u16, who: String) -> String;
}

// UDL-defined object with async methods
pub struct UdlMegaphone;

impl UdlMegaphone {
    pub async fn new() -> Self {
        TimerFuture::new(Duration::from_millis(0)).await;
        UdlMegaphone
    }

    pub async fn secondary() -> Self {
        TimerFuture::new(Duration::from_millis(0)).await;
        UdlMegaphone
    }

    pub async fn say_after(&self, ms: u16, who: String) -> String {
        TimerFuture::new(Duration::from_millis(ms.into())).await;
        format!("Hello, {who} (from UDL Megaphone)!").to_uppercase()
    }
}

// Proc-macro-defined object with async methods (Megaphone)
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
        TimerFuture::new(Duration::from_millis(ms.into())).await;
        format!("Hello, {who}!").to_uppercase()
    }

    /// Async method without any extra arguments
    pub async fn silence(&self) -> String {
        TimerFuture::new(Duration::from_millis(100)).await;
        String::new()
    }

    /// Async method that can throw
    pub async fn fallible_me(self: Arc<Self>, do_fail: bool) -> Result<u8, MyError> {
        TimerFuture::new(Duration::from_millis(10)).await;
        if do_fail {
            Err(MyError::Foo)
        } else {
            Ok(42)
        }
    }
}

// Mixed async/sync methods on the same object (using tokio runtime)
#[uniffi::export(async_runtime = "tokio")]
impl Megaphone {
    /// Sync method that yells something immediately
    pub fn say_now(&self, who: String) -> String {
        format!("Hello, {who}!").to_uppercase()
    }

    /// Async method using Tokio's timer
    pub async fn say_after_with_tokio(self: Arc<Self>, ms: u16, who: String) -> String {
        tokio::time::sleep(Duration::from_millis(ms.into())).await;
        format!("Hello, {who} (with Tokio)!").to_uppercase()
    }
}

// Additional async functions that work with objects
#[uniffi::export]
pub fn new_megaphone() -> Arc<Megaphone> {
    Arc::new(Megaphone)
}

#[uniffi::export]
pub async fn async_new_megaphone() -> Arc<Megaphone> {
    Arc::new(Megaphone)
}

#[uniffi::export]
pub async fn async_maybe_new_megaphone(should_create: bool) -> Option<Arc<Megaphone>> {
    TimerFuture::new(Duration::from_millis(10)).await;
    if should_create {
        Some(Arc::new(Megaphone))
    } else {
        None
    }
}

#[uniffi::export]
pub async fn say_after_with_megaphone(megaphone: Arc<Megaphone>, ms: u16, who: String) -> String {
    megaphone.say_after(ms, who).await
}

#[uniffi::export]
pub async fn fallible_struct(do_fail: bool) -> Result<Arc<Megaphone>, MyError> {
    TimerFuture::new(Duration::from_millis(10)).await;
    if do_fail {
        Err(MyError::Foo)
    } else {
        Ok(Arc::new(Megaphone))
    }
}

// Fallible async constructor object
#[derive(uniffi::Object)]
pub struct FallibleMegaphone;

#[uniffi::export]
impl FallibleMegaphone {
    #[uniffi::constructor]
    pub async fn new() -> Result<Arc<Self>, MyError> {
        TimerFuture::new(Duration::from_millis(10)).await;
        Err(MyError::Foo) // Always fails for testing
    }
}

uniffi::include_scaffolding!("api");
