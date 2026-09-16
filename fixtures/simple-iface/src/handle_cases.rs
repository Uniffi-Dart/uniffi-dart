//! Rust-owned and foreign trait handles share the fixed-width UniFFI ABI.
use std::sync::Arc;

#[uniffi::export]
pub trait HandleLabel: Send + Sync {
    fn label(&self) -> u32;
}

#[uniffi::export(with_foreign)]
pub trait HandleDelegate: Send + Sync {
    fn label(&self) -> u32;
}

struct Label;
impl HandleLabel for Label {
    fn label(&self) -> u32 {
        42
    }
}
impl HandleDelegate for Label {
    fn label(&self) -> u32 {
        43
    }
}

#[uniffi::export]
pub fn make_handle_label() -> Arc<dyn HandleLabel> {
    Arc::new(Label)
}

#[uniffi::export]
pub fn make_handle_delegate() -> Arc<dyn HandleDelegate> {
    Arc::new(Label)
}

#[uniffi::export]
pub fn call_handle_delegate(value: Arc<dyn HandleDelegate>) -> u32 {
    value.label()
}

#[uniffi::export]
pub fn echo_handle_delegate(value: Arc<dyn HandleDelegate>) -> Arc<dyn HandleDelegate> {
    value
}
