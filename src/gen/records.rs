use super::oracle::{AsCodeType, DartCodeOracle};
use super::primitives::render_interface_default_value;
use super::render::{Renderable, TypeHelperRenderer};
use super::types::generate_type;
use crate::gen::CodeType;
use genco::prelude::*;
use uniffi_bindgen::interface::{AsType, Record, Type};
use uniffi_bindgen::pipeline::general::nodes::Literal as PipelineLiteral;

#[derive(Debug)]
pub struct RecordCodeType {
    id: String,
    // module_path: String,
}

impl RecordCodeType {
    pub fn new(id: String, //module_path: String
    ) -> Self {
        Self { id, // module_path 
            }
    }
}

impl CodeType for RecordCodeType {
    fn type_label(&self) -> String {
        DartCodeOracle::class_name(&self.id)
    }

    fn canonical_name(&self) -> String {
        self.id.to_string()
    }

    fn literal(&self, _literal: &PipelineLiteral) -> String {
        todo!("literal not implemented for RecordCodeType");
    }
}

fn enum_name_from_type(ty: &Type) -> Option<String> {
    match ty {
        Type::Enum { name, .. } => Some(DartCodeOracle::class_name(name)),
        Type::Optional { inner_type } => enum_name_from_type(inner_type),
        _ => None,
    }
}

impl Renderable for RecordCodeType {
    fn render_type_helper(&self, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
        if type_helper.check(&self.id) {
            quote!()
        } else if let Some(record_) = type_helper.get_record(&self.id) {
            generate_record(record_, type_helper)
        } else {
            todo!("render_type_helper not implemented for unknown record type");
        }
    }
}

pub fn generate_record(obj: &Record, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
    let cls_name = &DartCodeOracle::class_name(obj.name());
    let ffi_conv_name = &DartCodeOracle::class_name(&obj.as_codetype().ffi_converter_name());
    let constructor_params: Vec<dart::Tokens> = obj
        .fields()
        .iter()
        .map(|field| {
            let name = DartCodeOracle::var_name(field.name());
            if let Some(default_value) = field.default_value() {
                if let Some(default_expr) = render_interface_default_value(
                    default_value,
                    &field.as_type(),
                    |ty, variant| {
                        let enum_name = enum_name_from_type(ty)?;
                        let variant_name = DartCodeOracle::enum_variant_name(variant);
                        Some(format!("{enum_name}.{variant_name}"))
                    },
                ) {
                    quote!(this.$name = $(default_expr))
                } else {
                    // Fallback to required when a default is present but cannot be rendered safely.
                    quote!(required this.$name)
                }
            } else if matches!(field.as_type(), Type::Optional { .. }) {
                quote!(this.$name)
            } else {
                quote!(required this.$name)
            }
        })
        .collect();

    let constructor = if obj.fields().is_empty() {
        quote!($(cls_name)();)
    } else {
        quote!($(cls_name)({$(for param in constructor_params => $param, )});)
    };

    for f in obj.fields() {
        type_helper.include_once_check(&f.as_codetype().canonical_name(), &f.as_type());
    }
    quote! {
        class $cls_name {
            $(for f in obj.fields() => final $(generate_type(&f.as_type())) $(DartCodeOracle::var_name(f.name()));)

            $constructor
        }

        class $ffi_conv_name {
            static $cls_name lift( RustBuffer buf) {
                return $ffi_conv_name.read(buf.asUint8List()).value;
            }

            static LiftRetVal<$cls_name> read( Uint8List buf) {
                int new_offset = buf.offsetInBytes;

                $(for f in obj.fields() =>
                    final $(DartCodeOracle::var_name(f.name()))_lifted = $(f.as_type().as_codetype().ffi_converter_name()).read(Uint8List.view(buf.buffer, new_offset));
                    final $(DartCodeOracle::var_name(f.name())) = $(DartCodeOracle::var_name(f.name()))_lifted.value;
                    new_offset += $(DartCodeOracle::var_name(f.name()))_lifted.bytesRead;
                )
                return LiftRetVal($(cls_name)(
                    $(for f in obj.fields() => $(DartCodeOracle::var_name(f.name())): $(DartCodeOracle::var_name(f.name())),)
                ), new_offset - buf.offsetInBytes);
            }

            static RustBuffer lower( $cls_name value) {
                final total_length = $(for f in obj.fields() => $(f.as_type().as_codetype().ffi_converter_name()).allocationSize(value.$(DartCodeOracle::var_name(f.name()))) + ) 0;
                final buf = Uint8List(total_length);
                write(value, buf);
                return toRustBuffer(buf);
            }

            static int write( $cls_name value, Uint8List buf) {
                int new_offset = buf.offsetInBytes;

                $(for f in obj.fields() =>
                new_offset += $(f.as_type().as_codetype().ffi_converter_name()).write(value.$(DartCodeOracle::var_name(f.name())), Uint8List.view(buf.buffer, new_offset));
                )
                return new_offset - buf.offsetInBytes;
            }

            static int allocationSize($cls_name value) {
                return $(for f in obj.fields() => $(f.as_type().as_codetype().ffi_converter_name()).allocationSize(value.$(DartCodeOracle::var_name(f.name()))) + ) 0;
            }
        }
    }
}
