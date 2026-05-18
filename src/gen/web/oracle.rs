use genco::prelude::*;
use uniffi_bindgen::interface::Type;

pub struct WebCodeOracle;

impl WebCodeOracle {
    pub fn dart_type_label(type_: &Type) -> Option<dart::Tokens> {
        match type_ {
            Type::Boolean => Some(quote!(bool)),
            Type::Int8 | Type::UInt8 | Type::Int16 | Type::UInt16 | Type::Int32 | Type::UInt32 => {
                Some(quote!(int))
            }
            Type::Int64 | Type::UInt64 => None,
            Type::Float32 | Type::Float64 => Some(quote!(double)),
            Type::String => Some(quote!(String)),
            Type::Bytes => Some(quote!(Uint8List)),
            _ => None,
        }
    }

    pub fn js_type_label(type_: &Type) -> Option<dart::Tokens> {
        match type_ {
            Type::Boolean => Some(quote!(JSBoolean)),
            Type::Int8
            | Type::UInt8
            | Type::Int16
            | Type::UInt16
            | Type::Int32
            | Type::UInt32
            | Type::Float32
            | Type::Float64 => Some(quote!(JSNumber)),
            Type::Int64 | Type::UInt64 => None,
            Type::String => Some(quote!(JSString)),
            Type::Bytes => Some(quote!(JSUint8Array)),
            _ => None,
        }
    }

    pub fn lower_expr(type_: &Type, expr: dart::Tokens) -> Option<dart::Tokens> {
        match type_ {
            Type::Boolean
            | Type::Int8
            | Type::UInt8
            | Type::Int16
            | Type::UInt16
            | Type::Int32
            | Type::UInt32
            | Type::Float32
            | Type::Float64
            | Type::String
            | Type::Bytes => Some(quote!($expr.toJS)),
            Type::Int64 | Type::UInt64 => None,
            _ => None,
        }
    }

    pub fn lift_expr(type_: &Type, expr: dart::Tokens) -> Option<dart::Tokens> {
        match type_ {
            Type::Boolean | Type::String | Type::Bytes => Some(quote!($expr.toDart)),
            Type::Int8 | Type::UInt8 | Type::Int16 | Type::UInt16 | Type::Int32 | Type::UInt32 => {
                Some(quote!($expr.toDartInt))
            }
            Type::Int64 | Type::UInt64 => None,
            Type::Float32 | Type::Float64 => Some(quote!($expr.toDartDouble)),
            _ => None,
        }
    }
}
