use std::sync::{Arc, Mutex};

use crate::BasicError;

#[uniffi::export]
pub fn combine_status(status: u32, status_1: u32, status_2: u32) -> u32 {
    status * 100 + status_1 * 10 + status_2
}

#[uniffi::export]
pub fn check_status(status: u32) -> Result<(), BasicError> {
    if status == 0 {
        Ok(())
    } else {
        Err(BasicError::InvalidInput)
    }
}

#[derive(uniffi::Object)]
pub struct StatusObject {
    value: Mutex<u32>,
}

#[uniffi::export]
impl StatusObject {
    #[uniffi::constructor]
    pub fn new(status: u32) -> Arc<Self> {
        Arc::new(Self { value: Mutex::new(status) })
    }

    #[uniffi::constructor]
    pub fn with_status(status: u32, status_1: u32) -> Arc<Self> {
        Self::new(status + status_1)
    }

    pub fn add_status(&self, status: u32) -> u32 {
        *self.value.lock().unwrap() + status
    }

    pub fn set_status(&self, status: u32) {
        *self.value.lock().unwrap() = status;
    }

    pub fn as_status_trait(self: Arc<Self>) -> Arc<dyn StatusTrait> {
        self
    }

    pub fn as_foreign_status_trait(self: Arc<Self>) -> Arc<dyn ForeignStatusTrait> {
        self
    }
}

#[uniffi::export]
pub trait StatusTrait: Send + Sync {
    fn add_status(&self, status: u32) -> u32;
    fn set_status(&self, status: u32);
}

impl StatusTrait for StatusObject {
    fn add_status(&self, status: u32) -> u32 {
        StatusObject::add_status(self, status)
    }

    fn set_status(&self, status: u32) {
        StatusObject::set_status(self, status)
    }
}

#[uniffi::export(with_foreign)]
pub trait ForeignStatusTrait: Send + Sync {
    fn describe_status(&self, status: u32) -> Result<String, BasicError>;
    fn add_status(&self, status: u32) -> u32;
    fn set_status(&self, status: u32);
}

impl ForeignStatusTrait for StatusObject {
    fn describe_status(&self, status: u32) -> Result<String, BasicError> {
        check_status(status)?;
        Ok(format!("status: {status}"))
    }

    fn add_status(&self, status: u32) -> u32 {
        StatusObject::add_status(self, status)
    }

    fn set_status(&self, status: u32) {
        StatusObject::set_status(self, status)
    }
}

#[uniffi::export]
pub fn call_foreign_status(callback: Arc<dyn ForeignStatusTrait>, status: u32) -> u32 {
    callback.set_status(status);
    callback.add_status(2)
}

#[uniffi::export]
pub fn describe_foreign_status(
    callback: Arc<dyn ForeignStatusTrait>,
    status: u32,
) -> Result<String, BasicError> {
    callback.describe_status(status)
}
