use genco::prelude::*;
use uniffi_bindgen::interface::{Argument, AsType, DefaultValue, Type};

use super::oracle::DartCodeOracle;
use super::primitives::render_interface_default_value;

fn enum_name_from_type(ty: &Type) -> Option<String> {
    match ty {
        Type::Enum { name, .. } => Some(DartCodeOracle::class_name(name)),
        Type::Optional { inner_type } => enum_name_from_type(inner_type),
        _ => None,
    }
}

pub(crate) fn render_default_value(default_value: &DefaultValue, ty: &Type) -> Option<String> {
    render_interface_default_value(default_value, ty, |ty, variant| {
        let enum_name = enum_name_from_type(ty)?;
        let variant_name = DartCodeOracle::enum_variant_name(variant);
        Some(format!("{enum_name}.{variant_name}"))
    })
}

pub(crate) fn render_argument_param(arg: &Argument, type_tokens: dart::Tokens) -> dart::Tokens {
    let name = DartCodeOracle::var_name(arg.name());
    let type_tokens = if super::unsigned::accepts_u64(&arg.as_type()) {
        super::unsigned::input_type(&arg.as_type())
    } else {
        type_tokens
    };

    if let Some(default_value) = arg.default_value() {
        if let Some(default_expr) = render_default_value(default_value, &arg.as_type()) {
            quote!($type_tokens $name = $(default_expr))
        } else {
            quote!(required $type_tokens $name)
        }
    } else {
        quote!(required $type_tokens $name)
    }
}
