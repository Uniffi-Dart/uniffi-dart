use core::time::Duration;

use uniffi;

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

#[derive(uniffi::Record)]
pub struct DurationRecord {
    pub value: Duration,
}

#[uniffi::export]
pub fn echo_duration_record(value: DurationRecord) -> DurationRecord {
    value
}

#[uniffi::export]
pub fn echo_optional_duration(value: Option<Duration>) -> Option<Duration> {
    value
}

#[uniffi::export]
pub fn echo_duration_sequence(values: Vec<Duration>) -> Vec<Duration> {
    values
}

uniffi::include_scaffolding!("api");
