use uniffi;
use thiserror;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum MathError {
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },
}

#[uniffi::export]
pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

#[uniffi::export]
pub fn multiply(left: u32, right: u32) -> u32 {
    left * right
}

#[uniffi::export]
pub fn divide(left: u32, right: u32) -> Option<u32> {
    Some(left / right)
}

#[uniffi::export]
pub fn can_divide(left: u32, right: u32) -> Option<bool> {
    Some(left > right)
}

#[uniffi::export]
pub fn divide_checked(left: u32, right: u32) -> Option<u32> {
    left.checked_div(right)
}

#[uniffi::export]
pub fn add_u8(left: u8, right: u8) -> Option<u8> {
    Some(left + right)
}

#[uniffi::export]
pub fn add_u16(left: u16, right: u16) -> Option<u16> {
    Some(left + right)
}

#[uniffi::export]
pub fn add_u64(left: u64, right: u64) -> Option<u64> {
    Some(left + right)
}

#[uniffi::export]
pub fn add_i8(left: i8, right: i8) -> Option<i8> {
    Some(left + right)
}

#[uniffi::export]
pub fn add_i16(left: i16, right: i16) -> Option<i16> {
    Some(left + right)
}

#[uniffi::export]
pub fn add_i32(left: i32, right: i32) -> Option<i32> {
    Some(left + right)
}

#[uniffi::export]
pub fn add_i64(left: i64, right: i64) -> Option<i64> {
    Some(left + right)
}

#[uniffi::export]
pub fn add_f32(left: f32, right: f32) -> Option<f32> {
    Some(left + right)
}

#[uniffi::export]
pub fn add_f64(left: f64, right: f64) -> Option<f64> {
    Some(left + right)
}

#[uniffi::export]
pub fn divide_by_zero(numerator: u64, denominator: u64) -> Result<u64, MathError> {
    if denominator == 0 {
        Err(MathError::DivisionByZero)
    } else {
        Ok(numerator / denominator)
    }
}

macro_rules! get_back {
    ($T:ty, $n:ident ) => {
        #[uniffi::export]
        pub fn $n(value: $T) -> Option<$T> {
            Some(value)
        }
    };
}

// Make sure optional types behave correctly
get_back!(u8, get_back_u8);
get_back!(u16, get_back_u16);
get_back!(u32, get_back_u32);
get_back!(u64, get_back_u64);
get_back!(i8, get_back_i8);
get_back!(i16, get_back_i16);
get_back!(i32, get_back_i32);
get_back!(i64, get_back_i64);
get_back!(f64, get_back_f64);
get_back!(f32, get_back_f32);

uniffi::include_scaffolding!("api");
