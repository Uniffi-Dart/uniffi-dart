use std::collections::HashMap;

// Minimal Map<K, V> round-trip surface. Mirrors how our qdrant-edge-ffi crate
// exposes payload/config maps: plain proc-macro exports over HashMap<String, _>.

#[uniffi::export]
pub fn roundtrip_map(m: HashMap<String, i32>) -> HashMap<String, i32> {
    m
}

#[uniffi::export]
pub fn count_entries(m: HashMap<String, i32>) -> u32 {
    m.len() as u32
}

#[uniffi::export]
pub fn map_with_record_values(m: HashMap<String, Point>) -> HashMap<String, Point> {
    m
}

// Variable-length value converters (nested Map, Option) are what stress the Map
// FfiConverter's offset arithmetic — a fixed-size value (i32/Point) can't reveal
// a drifting offset. These mirror real payload shapes: string -> nested/nullable.

#[uniffi::export]
pub fn roundtrip_nested_map(
    m: HashMap<String, HashMap<String, i32>>,
) -> HashMap<String, HashMap<String, i32>> {
    m
}

#[uniffi::export]
pub fn roundtrip_optional_map(m: HashMap<String, Option<i32>>) -> HashMap<String, Option<i32>> {
    m
}

#[derive(uniffi::Record, Clone)]
pub struct Point {
    x: i64,
    y: i64,
}

uniffi::include_scaffolding!("api");
