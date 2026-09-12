use core::time::Duration;

#[uniffi::export]
pub fn make_duration(seconds: u64, nanos: u32) -> Duration {
    Duration::new(seconds, nanos)
}

#[uniffi::export]
pub fn get_seconds(duration: Duration) -> u64 {
    duration.as_secs()
}

#[uniffi::export]
pub fn get_nanos(duration: Duration) -> u32 {
    duration.subsec_nanos()
}

uniffi::include_scaffolding!("api");
