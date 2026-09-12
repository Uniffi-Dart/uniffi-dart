use std::sync::Arc;
use std::time::Duration;

#[derive(uniffi::Enum, PartialEq, Debug)]
pub enum Shade {
    Light,
    Dark,
}

#[derive(uniffi::Enum, PartialEq, Debug)]
pub enum Choice {
    Named { value: u32 },
}

pub struct Count(pub u32);
uniffi::custom_type!(Count, u32, {
    lower: |value| value.0,
    try_lift: |value| Ok(Count(value)),
});

pub struct Ratio(pub f64);
uniffi::custom_newtype!(Ratio, f64);

#[uniffi::export(with_foreign)]
pub trait AbiValues: Send + Sync {
    fn shade(&self, value: Shade) -> Shade;
    fn choice(&self, value: Choice) -> Choice;
    fn duration(&self, value: Duration) -> Duration;
    fn count(&self, value: Count) -> Count;
    fn ratio(&self, value: Ratio) -> Ratio;
}

#[uniffi::export]
pub fn roundtrip_shade(callback: Arc<dyn AbiValues>, value: Shade) -> Shade {
    callback.shade(value)
}

#[uniffi::export]
pub fn roundtrip_choice(callback: Arc<dyn AbiValues>, value: Choice) -> Choice {
    callback.choice(value)
}

#[uniffi::export]
pub fn roundtrip_duration(callback: Arc<dyn AbiValues>, value: Duration) -> Duration {
    callback.duration(value)
}

#[uniffi::export]
pub fn roundtrip_count(callback: Arc<dyn AbiValues>, value: Count) -> Count {
    callback.count(value)
}

#[uniffi::export]
pub fn roundtrip_ratio(callback: Arc<dyn AbiValues>, value: Ratio) -> Ratio {
    callback.ratio(value)
}

#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait AsyncAbiValues: Send + Sync {
    async fn shade(&self, value: Shade) -> Shade;
    async fn duration(&self, value: Duration) -> Duration;
    async fn count(&self, value: Count) -> Count;
}

#[uniffi::export]
pub async fn check_async_values(callback: Arc<dyn AsyncAbiValues>) -> bool {
    callback.shade(Shade::Light).await == Shade::Dark
        && callback.duration(Duration::from_micros(1_000_001)).await
            == Duration::from_micros(1_000_003)
        && callback.count(Count(41)).await.0 == 42
}

uniffi::include_scaffolding!("api");
