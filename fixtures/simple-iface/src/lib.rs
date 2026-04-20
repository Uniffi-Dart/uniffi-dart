use std::sync::Arc;

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
        Self {
            payload_error: Arc::new(PayloadError::new(message)),
        }
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
