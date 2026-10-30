//! Public input types differ from stored/lifted types only at u64 leaves.
use genco::prelude::*;
use uniffi_bindgen::interface::Type;

use super::oracle::DartCodeOracle;

pub(crate) fn accepts_u64(ty: &Type) -> bool {
    match ty {
        Type::UInt64 => true,
        Type::Optional { inner_type } | Type::Sequence { inner_type } => accepts_u64(inner_type),
        Type::Map { key_type, value_type } => accepts_u64(key_type) || accepts_u64(value_type),
        Type::Custom { builtin, .. } => accepts_u64(builtin),
        // Records and enum variants normalize their fields in their constructors.
        _ => false,
    }
}

pub(crate) fn input_type(ty: &Type) -> dart::Tokens {
    match ty {
        // A component can itself define a Rust object named Object.
        Type::UInt64 => quote!(uniffiCore.Object),
        Type::Optional { inner_type } => quote!($(input_type(inner_type))?),
        Type::Sequence { inner_type } => quote!(List<$(input_type(inner_type))>),
        Type::Map { key_type, value_type } => {
            quote!(Map<$(input_type(key_type)), $(input_type(value_type))>)
        }
        Type::Custom { builtin, .. } if accepts_u64(builtin) => input_type(builtin),
        _ => DartCodeOracle::dart_type_label(Some(ty)),
    }
}

pub(crate) fn normalize(ty: &Type, value: dart::Tokens) -> dart::Tokens {
    if !accepts_u64(ty) {
        return value;
    }
    match ty {
        Type::UInt64 => quote!(FfiConverterUInt64.normalize($value)),
        Type::Optional { inner_type } => {
            let inner = normalize(inner_type, value.clone());
            quote!(($value == null ? null : $inner))
        }
        Type::Sequence { inner_type } => {
            let inner = normalize(inner_type, quote!(element));
            let output = DartCodeOracle::dart_type_label(Some(inner_type));
            quote!($value.map<$output>((element) => $inner).toList())
        }
        Type::Map { key_type, value_type } => {
            let key = normalize(key_type, quote!(key));
            let val = normalize(value_type, quote!(value));
            quote!(uniffiNormalizeU64Map($value, (key) => $key, (value) => $val))
        }
        Type::Custom { builtin, .. } => normalize(builtin, value),
        _ => unreachable!(),
    }
}
