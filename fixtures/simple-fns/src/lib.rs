// Object map keys deliberately use stable pointer identity for Hash and Eq.
// Allow this for both the fixture and its generated UniFFI scaffolding.
#![allow(clippy::mutable_key_type)]

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

// Free-standing functions
fn get_string() -> String {
    "String created by Rust".to_owned()
}

fn get_int() -> i32 {
    1289
}

fn string_identity(s: String) -> String {
    s
}

fn hash_map_identity(h: HashMap<String, String>) -> HashMap<String, String> {
    h
}

fn object_map_identity(m: HashMap<Arc<MyHashSet>, u32>) -> HashMap<Arc<MyHashSet>, u32> {
    m
}

fn byte_to_u32(byte: u8) -> u32 {
    byte.into()
}

fn new_set() -> Arc<MyHashSet> {
    Arc::new(MyHashSet::new())
}

fn add_to_set(set: Arc<MyHashSet>, value: String) {
    set.add(value);
}

fn set_contains(set: Arc<MyHashSet>, value: String) -> bool {
    set.contains(value)
}

// This used to generate broken bindings because the type inside `Option` (and
// other generic builtin types) wasn't being added as a known type.
fn dummy(_arg: Option<i8>) {}

// MyHashSet implementation
#[derive(Default)]
pub struct MyHashSet {
    inner: Mutex<HashSet<String>>,
}

impl MyHashSet {
    pub fn new() -> Self {
        Self { inner: Mutex::new(HashSet::new()) }
    }

    pub fn add(&self, value: String) {
        self.inner.lock().unwrap().insert(value);
    }

    pub fn contains(&self, value: String) -> bool {
        self.inner.lock().unwrap().contains(&value)
    }
}

impl PartialEq for MyHashSet {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for MyHashSet {}

impl Hash for MyHashSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self as *const Self as usize).hash(state);
    }
}

uniffi::include_scaffolding!("api");
