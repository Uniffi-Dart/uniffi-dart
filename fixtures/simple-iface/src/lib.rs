use std::sync::Arc;
mod abi_probe;
mod handle_cases;

#[derive(Debug)]
pub enum ProbeError {
    Failed,
}
impl std::fmt::Display for ProbeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("expected handle ABI probe error")
    }
}
impl std::error::Error for ProbeError {}

#[derive(Debug)]
pub struct Object {
    inner: i32,
}

#[derive(Debug)]
pub struct PayloadError {
    message: String,
}

#[derive(Debug)]
pub struct ProtocolError {
    payload_error: Arc<PayloadError>,
}

impl Object {
    pub fn new(inner: i32) -> Self {
        Self { inner }
    }

    pub fn get_inner(&self) -> i32 {
        self.inner
    }

    pub fn fail(&self) -> Result<(), ProbeError> {
        Err(ProbeError::Failed)
    }

    pub fn some_method(self: Arc<Self>) -> Option<Arc<Self>> {
        None
    }
}

impl PayloadError {
    pub fn new(message: String) -> Self {
        Self { message }
    }

    pub fn message(&self) -> String {
        self.message.clone()
    }
}

impl ProtocolError {
    pub fn new(message: String) -> Self {
        Self { payload_error: Arc::new(PayloadError::new(message)) }
    }

    pub fn payload_error(self: Arc<Self>) -> Option<Arc<PayloadError>> {
        Some(self.payload_error.clone())
    }
}

pub fn make_object(inner: i32) -> Arc<Object> {
    Arc::new(Object::new(inner))
}

pub fn get_protocol_error(message: String) -> Arc<ProtocolError> {
    Arc::new(ProtocolError::new(message))
}

uniffi::include_scaffolding!("api");

pub trait ObjectMapper: Send + Sync {
    fn map(&self, value: Arc<Object>) -> Arc<Object>;
}

pub fn map_object(mapper: Box<dyn ObjectMapper>, value: Arc<Object>) -> Arc<Object> {
    mapper.map(value)
}

pub async fn make_object_async(inner: i32) -> Arc<Object> {
    // Exercise a real pending Rust future and a continuation on another thread.
    let ready = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut started = false;
    std::future::poll_fn(|cx| {
        if ready.load(std::sync::atomic::Ordering::Acquire) {
            return std::task::Poll::Ready(());
        }
        if !started {
            started = true;
            let ready = ready.clone();
            let waker = cx.waker().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(5));
                ready.store(true, std::sync::atomic::Ordering::Release);
                waker.wake();
            });
        }
        std::task::Poll::Pending
    })
    .await;
    make_object(inner)
}
