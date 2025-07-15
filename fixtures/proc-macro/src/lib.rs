use std::{collections::HashMap, sync::Arc};

mod callback_interface;
use callback_interface::TestCallbackInterface;

#[derive(uniffi::Record)]
pub struct One {
    inner: i32,
}

#[derive(uniffi::Record)]
pub struct Two {
    a: String,
}

#[derive(uniffi::Record)]
pub struct RecordWithBytes {
    some_bytes: Vec<u8>,
}

#[derive(uniffi::Object)]
pub struct Object;

#[uniffi::export]
impl Object {
    #[uniffi::constructor]
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }

    fn is_heavy(&self) -> MaybeBool {
        MaybeBool::Uncertain
    }

    fn get_trait(&self, inc: Option<Arc<dyn Trait>>) -> Arc<dyn Trait> {
        inc.unwrap_or_else(|| Arc::new(TraitImpl {}))
    }

    fn get_trait_with_foreign(
        &self,
        inc: Option<Arc<dyn TraitWithForeign>>,
    ) -> Arc<dyn TraitWithForeign> {
        inc.unwrap_or_else(|| Arc::new(RustTraitImpl {}))
    }
}

#[uniffi::export]
pub trait Trait: Send + Sync {
    fn concat_strings(&self, a: &str, b: &str) -> String;
}

struct TraitImpl {}

impl Trait for TraitImpl {
    fn concat_strings(&self, a: &str, b: &str) -> String {
        format!("{a}{b}")
    }
}

#[uniffi::export(with_foreign)]
pub trait TraitWithForeign: Send + Sync {
    fn name(&self) -> String;
}

struct RustTraitImpl {}

impl TraitWithForeign for RustTraitImpl {
    fn name(&self) -> String {
        "RustTraitImpl".to_string()
    }
}

#[uniffi::export]
pub fn make_one(inner: i32) -> One {
    One { inner }
}

#[uniffi::export]
pub fn take_two(two: Two) -> String {
    two.a
}

#[uniffi::export]
pub fn make_hashmap(k: i8, v: u64) -> HashMap<i8, u64> {
    HashMap::from([(k, v)])
}

#[derive(uniffi::Enum)]
pub enum MaybeBool {
    True,
    False,
    Uncertain,
}

#[derive(uniffi::Enum)]
pub enum MixedEnum {
    None,
    String(String),
    Int(i64),
    Both(String, i64),
}

#[derive(thiserror::Error, uniffi::Error, Debug, PartialEq, Eq)]
pub enum BasicError {
    #[error("InvalidInput")]
    InvalidInput,
    #[error("OsError")]
    OsError,
    #[error("UnexpectedError")]
    UnexpectedError { reason: String },
}

#[uniffi::export]
pub fn always_fails() -> Result<(), BasicError> {
    Err(BasicError::OsError)
}

// Type that's defined in the UDL and not wrapped with #[uniffi::export]
pub struct Zero {
    inner: String,
}

#[uniffi::export]
fn make_zero() -> Zero {
    Zero {
        inner: String::from("ZERO"),
    }
}

// UDL functions that reference proc-macro types
fn get_one(one: Option<One>) -> One {
    one.unwrap_or(One { inner: 0 })
}

fn get_bool(b: Option<MaybeBool>) -> MaybeBool {
    b.unwrap_or(MaybeBool::Uncertain)
}

fn get_object(o: Option<Arc<Object>>) -> Arc<Object> {
    o.unwrap_or_else(Object::new)
}

fn get_trait(o: Option<Arc<dyn Trait>>) -> Arc<dyn Trait> {
    o.unwrap_or_else(|| Arc::new(TraitImpl {}))
}

fn get_trait_with_foreign(o: Option<Arc<dyn TraitWithForeign>>) -> Arc<dyn TraitWithForeign> {
    o.unwrap_or_else(|| Arc::new(RustTraitImpl {}))
}

#[derive(Default)]
struct Externals {
    one: Option<One>,
    bool: Option<MaybeBool>,
}

fn get_externals(e: Option<Externals>) -> Externals {
    e.unwrap_or_default()
}

uniffi::include_scaffolding!("api"); 