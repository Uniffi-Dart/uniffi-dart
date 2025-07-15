use thiserror::Error as ThisError;
use uniffi::{Error, Record};

#[derive(Clone, Debug, Record)]
pub struct ErrorPayload {
    pub is_important: bool,
    pub index: u64,
    pub message: String,
}

impl Default for ErrorPayload {
    fn default() -> Self {
        Self {
            is_important: true,
            index: 42,
            message: "Very important error payload that greatly helps with debugging".to_owned(),
        }
    }
}

impl std::fmt::Display for ErrorPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "is_important: {}, index: {}, message: {}",
            self.is_important, self.index, self.message
        )
    }
}

#[repr(u32)]
#[derive(Clone, Debug, ThisError, Error)]
pub enum LargeError {
    #[error("Important debug description of what went wrong.")]
    Case1 = 10001,

    #[error("Important debug description of what went wrong, arg: '{arg1}'")]
    Case2 { arg1: String },

    #[error("Important debug description of what went wrong, arg: '{arg1}'")]
    Case3 { arg1: String },

    #[error("Important debug description of what went wrong, arg: '{arg1}'")]
    Case4 { arg1: String },

    #[error("Important debug description of what went wrong, arg: '{arg1}'")]
    Case5 { arg1: String },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}'")]
    Case20 { arg1: u64, arg2: u32 },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}'")]
    Case21 { arg1: u64, arg2: u32 },

    #[error("Important debug description of what went wrong, arg1: '{arg1}'")]
    Case30 { arg1: ErrorPayload },

    #[error("Important debug description of what went wrong, arg1: '{arg1}'")]
    Case31 { arg1: ErrorPayload },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case40 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    // ... (abbreviated from 100 cases to show pattern)
    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case100 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },
}

impl LargeError {
    pub fn discriminant(&self) -> u32 {
        unsafe { *<*const _>::from(self).cast::<u32>() }
    }
}

#[derive(Clone, Debug, Record)]
pub struct ErrorWithContext {
    pub error: LargeError,
    pub context: String,
}

#[uniffi::export]
pub fn error_message_from_error(error: &LargeError) -> String {
    format!("{}", error)
}

#[uniffi::export]
pub fn error_code_from_error(error: &LargeError) -> u32 {
    error.discriminant()
}

uniffi::include_scaffolding!("api"); 