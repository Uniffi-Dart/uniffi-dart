use std::collections::HashMap;
use std::str;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

static U64_CALLS: AtomicU32 = AtomicU32::new(0);

fn take_i8(v: i8) -> i8 {
    v
}

fn take_i16(v: i16) -> i16 {
    v
}

fn take_i32(v: i32) -> i32 {
    v
}

fn take_i64(v: i64) -> i64 {
    v
}

fn take_u8(v: u8) -> u8 {
    v
}

fn take_u16(v: u16) -> u16 {
    v
}

fn take_u32(v: u32) -> u32 {
    v
}

fn take_u64(v: u64) -> u64 {
    v
}

// Only the rejection test uses this entry point. Ordinary round trips can run
// concurrently in other Dart isolates without changing its counter.
fn observed_u64(v: u64) -> u64 {
    U64_CALLS.fetch_add(1, Ordering::Relaxed);
    v
}

fn take_f32(v: f32) -> f32 {
    v
}

fn take_f64(v: f64) -> f64 {
    v
}

fn take_string(v: String) -> String {
    assert!(str::from_utf8(v.as_bytes()).is_ok());
    v
}

fn take_bytes(v: Vec<u8>) -> Vec<u8> {
    v
}

uniffi::include_scaffolding!("api");

fn max_u64() -> u64 {
    u64::MAX
}
fn default_u64(v: u64) -> u64 {
    v
}
fn optional_u64(v: Option<u64>) -> Option<u64> {
    v
}
fn sequence_u64(v: Vec<Option<u64>>) -> Vec<Option<u64>> {
    v
}
fn map_u64(v: HashMap<u64, Vec<Option<u64>>>) -> HashMap<u64, Vec<Option<u64>>> {
    v
}
fn record_u64(v: U64Record) -> U64Record {
    v
}
fn enum_u64(v: U64Value) -> U64Value {
    v
}
fn alias_u64(v: U64Alias) -> U64Alias {
    v
}
fn u64_calls() -> u32 {
    U64_CALLS.load(Ordering::Relaxed)
}

#[derive(Clone)]
pub struct U64Alias(pub u64);
uniffi::custom_newtype!(U64Alias, u64);

pub struct U64Record {
    pub value: u64,
    pub default_max: u64,
    pub default_zero: u64,
    pub maybe: Option<u64>,
    pub optional_max: Option<u64>,
    pub values: Vec<Option<u64>>,
    pub mapping: HashMap<u64, Vec<Option<u64>>>,
    pub alias: U64Alias,
}

pub enum U64Value {
    Single { value: u64 },
    Multiple { value: u64, values: Vec<Option<u64>> },
}

pub struct U64Box {
    value: u64,
}
impl U64Box {
    fn new(value: u64) -> Self {
        Self { value }
    }
    fn get(&self) -> u64 {
        self.value
    }
    fn echo(&self, value: u64) -> u64 {
        value
    }
}

pub trait U64Callback: Send + Sync {
    fn transform(&self, value: u64) -> u64;
    fn transform_alias(&self, value: U64Alias) -> U64Alias;
    fn transform_sequence(&self, values: Vec<Option<u64>>) -> Vec<Option<u64>>;
}
fn callback_u64(callback: Box<dyn U64Callback>, v: u64) -> u64 {
    callback.transform(v)
}
fn callback_sequence(callback: Box<dyn U64Callback>, v: Vec<Option<u64>>) -> Vec<Option<u64>> {
    callback.transform_sequence(v)
}

#[uniffi::export]
pub async fn async_u64(v: u64) -> u64 {
    v
}

#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait AsyncU64Callback: Send + Sync {
    async fn transform(&self, value: u64) -> u64;
    async fn transform_sequence(&self, values: Vec<Option<u64>>) -> Vec<Option<u64>>;
}

#[uniffi::export]
pub async fn async_callback_u64(callback: Arc<dyn AsyncU64Callback>, v: u64) -> u64 {
    callback.transform(v).await
}

#[uniffi::export]
pub async fn async_callback_sequence(
    callback: Arc<dyn AsyncU64Callback>,
    v: Vec<Option<u64>>,
) -> Vec<Option<u64>> {
    callback.transform_sequence(v).await
}

fn callback_alias(callback: Box<dyn U64Callback>, v: U64Alias) -> U64Alias {
    callback.transform_alias(v)
}

fn default_optional(v: Option<u64>) -> Option<u64> {
    v
}
