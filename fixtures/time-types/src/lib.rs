/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use chrono::offset::Utc;
use chrono::DateTime;

#[derive(Debug, thiserror::Error)]
pub enum ChronologicalError {
    #[error("Time overflow on an operation with {a:?} and {b:?}")]
    TimeOverflow { a: SystemTime, b: Duration },
    #[error("Time difference error {a:?} is before {b:?}")]
    TimeDiffError { a: SystemTime, b: SystemTime },
}

fn return_timestamp(a: SystemTime) -> Result<SystemTime> {
    Ok(a)
}

fn return_duration(a: Duration) -> Result<Duration> {
    Ok(a)
}

fn to_string_timestamp(a: SystemTime) -> String {
    let datetime: DateTime<Utc> = a.into();
    datetime.format("%Y-%m-%dT%H:%M:%S.%fZ").to_string()
}

fn get_pre_epoch_timestamp() -> SystemTime {
    std::time::SystemTime::UNIX_EPOCH.checked_sub(std::time::Duration::new(1, 1_000_000)).unwrap()
}

fn add(a: SystemTime, b: Duration) -> Result<SystemTime> {
    a.checked_add(b).ok_or(ChronologicalError::TimeOverflow { a, b })
}

fn diff(a: SystemTime, b: SystemTime) -> Result<Duration> {
    a.duration_since(b).map_err(|_| ChronologicalError::TimeDiffError { a, b })
}

fn now() -> SystemTime {
    SystemTime::now()
}

fn timestamp_from_parts(seconds: i64, nanos: u32) -> SystemTime {
    let offset = Duration::new(seconds.unsigned_abs(), nanos);
    if seconds < 0 {
        SystemTime::UNIX_EPOCH - offset
    } else {
        SystemTime::UNIX_EPOCH + offset
    }
}

fn timestamp_nanos(value: SystemTime) -> u32 {
    value
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_else(|error| error.duration())
        .subsec_nanos()
}

pub struct TimeBundle {
    at: SystemTime,
    optional_at: Option<SystemTime>,
    history: Vec<SystemTime>,
    named: HashMap<String, SystemTime>,
}

fn echo_time_bundle(value: TimeBundle) -> TimeBundle {
    value
}

pub trait TimeSource: Send + Sync {
    fn echo(&self, value: SystemTime) -> SystemTime;
}

fn callback_timestamp(source: Box<dyn TimeSource>, value: SystemTime) -> SystemTime {
    source.echo(value)
}

async fn return_timestamp_async(value: SystemTime) -> SystemTime {
    value
}

pub struct TimeKeeper {
    value: SystemTime,
}

impl TimeKeeper {
    fn new(value: SystemTime) -> Self {
        Self { value }
    }

    fn get(&self) -> SystemTime {
        self.value
    }

    fn echo(&self, value: SystemTime) -> SystemTime {
        value
    }
}

fn equal(a: SystemTime, b: SystemTime) -> bool {
    a == b
}

fn optional(a: Option<SystemTime>, b: Option<Duration>) -> bool {
    a.is_some() && b.is_some()
}

fn get_seconds_before_unix_epoch(b: SystemTime) -> Result<u64> {
    diff(SystemTime::UNIX_EPOCH, b).map(|duration| duration.as_secs())
}

fn set_seconds_before_unix_epoch(seconds: u64) -> Result<SystemTime> {
    let a = SystemTime::UNIX_EPOCH;
    let b = Duration::from_secs(seconds);

    a.checked_sub(b).ok_or(ChronologicalError::TimeOverflow { a, b })
}

type Result<T, E = ChronologicalError> = std::result::Result<T, E>;

uniffi::include_scaffolding!("api");
