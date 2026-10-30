#[macro_use]
mod macros;
mod boolean;
mod duration;
mod string;
mod timestamp;

pub use boolean::BooleanCodeType;
pub use duration::DurationCodeType;
use genco::prelude::*;
use paste::paste;
pub use string::StringCodeType;
pub use timestamp::TimestampCodeType;
use uniffi_bindgen::interface::{
    DefaultValue as InterfaceDefaultValue, Literal as InterfaceLiteral, Radix as InterfaceRadix,
    Type as InterfaceType,
};
use uniffi_bindgen::pipeline::general::nodes::{
    Literal as PipelineLiteral, Radix as PipelineRadix, Type as PipelineType, TypeNode,
};

use crate::gen::render::{Renderable, TypeHelperRenderer};
use crate::gen::CodeType;

pub(crate) fn escape_dart_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn render_literal(literal: &PipelineLiteral) -> String {
    fn typed_number(type_node: &TypeNode, num_str: String) -> String {
        match &type_node.ty {
            PipelineType::Int8
            | PipelineType::UInt8
            | PipelineType::Int16
            | PipelineType::UInt16
            | PipelineType::Int32
            | PipelineType::UInt32
            | PipelineType::UInt64
            | PipelineType::Float32
            | PipelineType::Float64
            | PipelineType::Duration => num_str,
            _ => panic!("Unexpected literal: {num_str} is not a number"),
        }
    }

    match literal {
        PipelineLiteral::Boolean(v) => format!("{v}"),
        PipelineLiteral::String(s) => format!("'{}'", escape_dart_string(s)),
        PipelineLiteral::Int(i, radix, type_node) => typed_number(
            type_node,
            match radix {
                PipelineRadix::Octal => format!("{i:#x}"),
                PipelineRadix::Decimal => format!("{i}"),
                PipelineRadix::Hexadecimal => format!("{i:#x}"),
            },
        ),
        PipelineLiteral::UInt(i, radix, type_node) => typed_number(
            type_node,
            match radix {
                PipelineRadix::Octal => format!("{i:#x}"),
                PipelineRadix::Decimal => format!("{i}"),
                PipelineRadix::Hexadecimal => format!("{i:#x}"),
            },
        ),
        PipelineLiteral::Float(string, type_node) => typed_number(type_node, string.clone()),
        _ => unreachable!("Literal"),
    }
}

fn default_for_interface_type(ty: &InterfaceType) -> Option<String> {
    match ty {
        InterfaceType::Boolean => Some("false".to_string()),
        InterfaceType::String => Some("''".to_string()),
        InterfaceType::Int8
        | InterfaceType::Int16
        | InterfaceType::Int32
        | InterfaceType::Int64
        | InterfaceType::UInt8
        | InterfaceType::UInt16
        | InterfaceType::UInt32
        | InterfaceType::UInt64 => Some("0".to_string()),
        InterfaceType::Float32 | InterfaceType::Float64 => Some("0.0".to_string()),
        InterfaceType::Duration => Some("Duration.zero".to_string()),
        InterfaceType::Optional { .. } => Some("null".to_string()),
        InterfaceType::Sequence { .. } => Some("const []".to_string()),
        InterfaceType::Map { .. } => Some("const {}".to_string()),
        _ => None,
    }
}

fn render_interface_literal<F>(
    literal: &InterfaceLiteral,
    field_type: &InterfaceType,
    enum_variant_renderer: F,
) -> Option<String>
where
    F: Fn(&InterfaceType, &str) -> Option<String> + Copy,
{
    match literal {
        InterfaceLiteral::Boolean(value) => Some(value.to_string()),
        InterfaceLiteral::String(value) => Some(format!("'{}'", escape_dart_string(value))),
        InterfaceLiteral::Int(value, radix, _) => Some(match radix {
            InterfaceRadix::Decimal => format!("{value}"),
            InterfaceRadix::Octal | InterfaceRadix::Hexadecimal => format!("{value:#x}"),
        }),
        InterfaceLiteral::UInt(value, radix, _) => Some(match radix {
            InterfaceRadix::Decimal => format!("{value}"),
            InterfaceRadix::Octal | InterfaceRadix::Hexadecimal => format!("{value:#x}"),
        }),
        InterfaceLiteral::Float(value, _) => Some(value.clone()),
        InterfaceLiteral::Enum(variant, ty) => enum_variant_renderer(ty, variant)
            .or_else(|| enum_variant_renderer(field_type, variant)),
        InterfaceLiteral::EmptySequence => Some("const []".to_string()),
        InterfaceLiteral::EmptyMap => Some("const {}".to_string()),
        InterfaceLiteral::None => Some("null".to_string()),
        InterfaceLiteral::Some { inner } => {
            render_interface_default_value(inner, field_type, enum_variant_renderer)
        }
    }
}

pub(crate) fn render_interface_default_value<F>(
    default: &InterfaceDefaultValue,
    field_type: &InterfaceType,
    enum_variant_renderer: F,
) -> Option<String>
where
    F: Fn(&InterfaceType, &str) -> Option<String> + Copy,
{
    match default {
        InterfaceDefaultValue::Default => default_for_interface_type(field_type),
        InterfaceDefaultValue::Literal(literal) => {
            render_interface_literal(literal, field_type, enum_variant_renderer)
        }
    }
}

impl_code_type_for_primitive!(BytesCodeType, "Uint8List", "Uint8List");
impl_code_type_for_primitive!(Int8CodeType, "int", "Int8");
impl_code_type_for_primitive!(Int16CodeType, "int", "Int16");
impl_code_type_for_primitive!(Int32CodeType, "int", "Int32");
impl_code_type_for_primitive!(Int64CodeType, "int", "Int64");
impl_code_type_for_primitive!(UInt8CodeType, "int", "UInt8");
impl_code_type_for_primitive!(UInt16CodeType, "int", "UInt16");
impl_code_type_for_primitive!(UInt32CodeType, "int", "UInt32");
impl_code_type_for_primitive!(UInt64CodeType, "int", "UInt64");
impl_code_type_for_primitive!(Float32CodeType, "double", "Double32");
impl_code_type_for_primitive!(Float64CodeType, "double", "Double64");

impl_renderable_for_primitive!(BytesCodeType, "Uint8List", "Uint8List");
impl_renderable_for_primitive!(Int8CodeType, "int", "Int8", 1, -128, 127, "i8");
impl_renderable_for_primitive!(Int16CodeType, "int", "Int16", 2, -32768, 32767, "i16");
impl_renderable_for_primitive!(Int32CodeType, "int", "Int32", 4, -2147483648, 2147483647, "i32");
impl_renderable_for_primitive!(
    Int64CodeType,
    "int",
    "Int64",
    8,
    -9223372036854775808,
    9223372036854775807,
    "i64"
);
impl_renderable_for_primitive!(UInt8CodeType, "int", "UInt8", 1, 0, 255, "u8");
impl_renderable_for_primitive!(UInt16CodeType, "int", "UInt16", 2, 0, 65535, "u16");
impl_renderable_for_primitive!(UInt32CodeType, "int", "UInt32", 4, 0, 4294967295, "u32");
impl_renderable_for_primitive!(Float32CodeType, "double", "Double32", 4);
impl_renderable_for_primitive!(Float64CodeType, "double", "Double64", 8);

// Keep u64 on the legacy int path for now; full upper-bound validation lands with BigInt support.
impl Renderable for UInt64CodeType {
    fn render_type_helper(&self, _type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
        let cl_name = &self.ffi_converter_name();
        let type_signature = &self.type_label();

        quote! {
            class $cl_name {
                static $type_signature lift($type_signature value) => value;

                static LiftRetVal<$type_signature> read(Uint8List buf) {
                    return LiftRetVal(buf.buffer.asByteData(buf.offsetInBytes).getUint64(0), 8);
                }

                static $type_signature lower($type_signature value) {
                    if (value < 0) {
                        throw ArgumentError("Value out of range for u64: " + value.toString());
                    }
                    return value;
                }

                static int allocationSize([$type_signature value = 0]) {
                    return 8;
                }

                static int write($type_signature value, Uint8List buf) {
                    buf.buffer.asByteData(buf.offsetInBytes).setUint64(0, lower(value));
                    return 8;
                }
            }
        }
    }
}
