use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TraitMethods {
    val: String,
}

impl TraitMethods {
    pub fn new(val: String) -> Self {
        Self { val }
    }
}

impl std::fmt::Display for TraitMethods {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TraitMethods({})", self.val)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, uniffi::Object)]
#[uniffi::export(Debug, Display, Eq, Hash)]
pub struct ProcTraitMethods {
    val: String,
}

#[uniffi::export]
impl ProcTraitMethods {
    #[uniffi::constructor]
    fn new(val: String) -> Arc<Self> {
        Arc::new(Self { val })
    }
}

impl std::fmt::Display for ProcTraitMethods {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProcTraitMethods({})", self.val)
    }
}

uniffi::include_scaffolding!("api");

#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi::export(Debug, Display)]
pub enum MessageError {
    #[error("foreign utxo missing witness_utxo or non_witness_utxo")]
    MissingUtxo,
    #[error("failed to convert input: {message}")]
    InputConversion { message: String },
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi::export(Display, Debug)]
pub enum UnitMessageError {
    #[error("the requested item does not exist")]
    Missing,
    #[error("the operation is not permitted")]
    Denied,
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi::export(Debug)]
pub enum DebugOnlyError {
    #[error("this Display implementation is not exported")]
    Detail { code: u32 },
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum LocalMessageError {
    #[error("this Display implementation is not exported either")]
    Detail { code: u32 },
}

#[uniffi::export]
pub fn fail_with_message(message: Option<String>) -> Result<(), MessageError> {
    Err(match message {
        Some(message) => MessageError::InputConversion { message },
        None => MessageError::MissingUtxo,
    })
}
#[uniffi::export]
pub fn fail_with_unit_message() -> Result<(), UnitMessageError> {
    Err(UnitMessageError::Missing)
}
#[uniffi::export]
pub fn fail_with_debug_only() -> Result<(), DebugOnlyError> {
    Err(DebugOnlyError::Detail { code: 7 })
}
#[uniffi::export]
pub fn fail_with_local_message() -> Result<(), LocalMessageError> {
    Err(LocalMessageError::Detail { code: 8 })
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum FlatLegacyError {
    #[error("flat errors are return-only")]
    Missing,
}
#[uniffi::export]
pub fn fail_with_flat_legacy() -> Result<(), FlatLegacyError> {
    Err(FlatLegacyError::Missing)
}
