#[uniffi::export]
pub fn owned_len(value: Vec<u8>) -> u64 {
    value.len() as u64
}

#[uniffi::export]
pub fn optional_len(value: Option<String>) -> u64 {
    value.map_or(0, |s| s.len() as u64)
}

#[uniffi::export]
pub fn optional_small(value: Option<u8>) -> u8 {
    value.unwrap_or(0)
}

uniffi::include_scaffolding!("api");
