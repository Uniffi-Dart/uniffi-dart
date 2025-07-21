use crate::{BasicError, Object, RecordWithBytes};
use std::sync::Arc;

#[uniffi::export(with_foreign)]
pub trait TestCallbackInterface: Send + Sync {
    fn do_nothing(&self);
    fn add(&self, a: i32, b: i32) -> i32;
    fn optional(&self, a: Option<i32>) -> i32;
    fn with_bytes(&self, rwb: RecordWithBytes) -> Vec<u8>;
    fn try_parse_int(&self, value: String) -> Result<i32, BasicError>;
    fn callback_handler(&self, o: Arc<Object>) -> i32;
    fn get_other_callback_interface(&self) -> Box<dyn OtherCallbackInterface>;
}

#[uniffi::export(with_foreign)]
pub trait OtherCallbackInterface: Send + Sync {
    fn multiply(&self, a: i32, b: i32) -> i32;
} 