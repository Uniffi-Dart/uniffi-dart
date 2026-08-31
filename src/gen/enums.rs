use genco::prelude::*;
use heck::ToLowerCamelCase;
use uniffi_bindgen::interface::{AsType, Enum, Field, Type};
use uniffi_bindgen::pipeline::general::nodes::Literal;

use super::oracle::{AsCodeType, DartCodeOracle};
use super::render::{AsRenderable, Renderable, TypeHelperRenderer};
use crate::gen::CodeType;

#[derive(Debug)]
pub struct EnumCodeType {
    id: String,
}

impl EnumCodeType {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

impl CodeType for EnumCodeType {
    fn type_label(&self) -> String {
        DartCodeOracle::class_name(&self.id)
    }

    fn canonical_name(&self) -> String {
        DartCodeOracle::class_name(&self.id)
    }

    fn literal(&self, literal: &Literal) -> String {
        if let Literal::Enum(v, _) = literal {
            format!("{}{}", self.type_label(), DartCodeOracle::enum_variant_name(v))
        } else {
            unreachable!();
        }
    }

    fn ffi_converter_name(&self) -> String {
        format!("FfiConverter{}", DartCodeOracle::class_name(&self.id))
    }
}

impl Renderable for EnumCodeType {
    fn render_type_helper(&self, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
        let canonical = self.canonical_name();
        if type_helper.check(&canonical) {
            quote!()
        } else if let Some(enum_) = type_helper.get_enum(&self.id) {
            generate_enum(enum_, type_helper)
        } else {
            unreachable!()
        }
    }
}

pub fn generate_enum(obj: &Enum, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
    let dart_cls_name = &DartCodeOracle::class_name(obj.name());
    let ffi_converter_name = &obj.as_codetype().ffi_converter_name();
    if obj.is_flat() {
        let is_error_enum = type_helper.get_ci().is_name_used_as_error(obj.name());
        let implements_exception =
            if is_error_enum { quote!( implements Exception) } else { quote!() };

        // For flat error enums, generate an error handler
        let error_handler_class = if is_error_enum {
            let error_handler_name = format!("{dart_cls_name}ErrorHandler");
            let instance_name = dart_cls_name.to_lower_camel_case();
            quote! {
                class $(&error_handler_name) extends UniffiRustCallStatusErrorHandler {
                    @override
                    Exception lift(RustBuffer errorBuf) {
                        return $ffi_converter_name.lift(errorBuf);
                    }
                }

                final $(&error_handler_name) $(instance_name)ErrorHandler = $(&error_handler_name)();
            }
        } else {
            quote!()
        };

        quote! {
            enum $dart_cls_name $implements_exception {
                $(for variant in obj.variants() =>
                $(DartCodeOracle::enum_variant_name(variant.name())),)
                ;
            }

            class $ffi_converter_name {
                static LiftRetVal<$dart_cls_name> read( Uint8List buf) {
                    final index = buf.buffer.asByteData(buf.offsetInBytes).getInt32(0);
                    switch(index) {
                        $(for (index, variant) in obj.variants().iter().enumerate() =>
                        case $(index + 1):
                            return LiftRetVal(
                                $dart_cls_name.$(DartCodeOracle::enum_variant_name(variant.name())),
                                4,
                            );
                        )
                        default:
                            throw UniffiInternalError(UniffiInternalError.unexpectedEnumCase, "Unable to determine enum variant");
                    }
                }

                static $dart_cls_name lift( RustBuffer buffer) {
                    return $ffi_converter_name.read(buffer.asUint8List()).value;
                }

                static RustBuffer lower( $dart_cls_name input) {
                    return toRustBuffer(createUint8ListFromInt(input.index + 1));
                }

                static int allocationSize($dart_cls_name _value) {
                    return 4;
                }

                static int write( $dart_cls_name value, Uint8List buf) {
                    buf.buffer
                        .asByteData(buf.offsetInBytes)
                        .setInt32(0, value.index + 1);
                    return 4;
                }
            }

            $error_handler_class
        }
    } else {
        let mut variants = vec![];

        // helper functions to get the sanitized field name and type strings
        fn field_name(field: &Field, field_num: usize) -> String {
            if field.name().is_empty() {
                format!("v{field_num}")
            } else {
                DartCodeOracle::var_name(field.name())
            }
        }
        fn field_type(field: &Field, type_helper: &dyn TypeHelperRenderer) -> String {
            field
                .as_type()
                .as_renderable()
                .render_type(&field.as_type(), type_helper)
                .to_string()
                .expect("Could not stringify type")
                .replace("Error", "Exception")
        }
        fn field_ffi_converter_name(field: &Field) -> String {
            field.as_type().as_codetype().ffi_converter_name().replace("Error", "Exception")
        }
        fn is_flat_enum(field: &Field, type_helper: &dyn TypeHelperRenderer) -> bool {
            if let Type::Enum { name, .. } = &field.as_type() {
                if let Some(enum_def) = type_helper.get_enum(name) {
                    return enum_def.is_flat();
                }
            }
            false
        }

        // A data-carrying variant's Dart class is named `{Variant}{Enum}`, which
        // can collide with a real top-level type of the same name (e.g. enum
        // `Condition` variant `Field` -> `FieldCondition`, clashing with a record
        // `FieldCondition`). Collect the names of every type that emits a
        // top-level Dart class so a colliding variant class can be disambiguated
        // instead of emitted twice. (Custom types lower to a typedef/converter,
        // not a class of that name, so they are deliberately excluded.)
        let reserved_type_names: std::collections::HashSet<String> = {
            let ci = type_helper.get_ci();
            ci.record_definitions()
                .iter()
                .map(|d| DartCodeOracle::class_name(d.name()))
                .chain(ci.enum_definitions().iter().map(|d| DartCodeOracle::class_name(d.name())))
                .chain(ci.object_definitions().iter().map(|d| DartCodeOracle::class_name(d.name())))
                .chain(
                    ci.callback_interface_definitions()
                        .iter()
                        .map(|d| DartCodeOracle::class_name(d.name())),
                )
                .collect()
        };

        // Name a data-carrying variant's Dart class `{Variant}{Enum}`, suffixing
        // on collision so it doesn't clash with a same-named top-level type (a
        // duplicate declaration). Used both where the class is defined and where
        // the FfiConverter's `read` dispatches to it, so the two stay in sync.
        let variant_class_name = |variant_name: &str| -> String {
            let base = format!("{}{}", DartCodeOracle::class_name(variant_name), dart_cls_name);
            if !reserved_type_names.contains(&base) {
                return base;
            }
            // Loop rather than suffix once: a type literally named `{base}Variant`
            // would otherwise reintroduce the very duplicate this guards against.
            let mut candidate = format!("{base}Variant");
            let mut n = 2;
            while reserved_type_names.contains(&candidate) {
                candidate = format!("{base}Variant{n}");
                n += 1;
            }
            candidate
        };

        for (index, variant_obj) in obj.variants().iter().enumerate() {
            for f in variant_obj.fields() {
                type_helper.include_once_check(&f.as_codetype().canonical_name(), &f.as_type());
            }
            let variant_dart_cls_name = &variant_class_name(variant_obj.name());

            // Prepare constructor parameters
            let constructor_params = variant_obj
                .fields()
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    let param_name = field_name(field, i);
                    let param_type = field_type(field, type_helper);
                    if variant_obj.fields().len() > 1 {
                        quote!(required $param_type this.$param_name)
                    } else {
                        quote!($param_type this.$param_name)
                    }
                })
                .collect::<Vec<_>>();

            let constructor_param_list = if variant_obj.fields().len() > 1 {
                quote!({ $( for p in constructor_params => $p, ) })
            } else {
                quote!($( for p in constructor_params => $p, ))
            };

            // Pre-process field reading code
            let field_read_code: Vec<dart::Tokens> = variant_obj.fields().iter().enumerate().map(|(i, field)| {
                if is_flat_enum(field, type_helper) {
                    // Handle flat enums specially - they serialize as int32 (4 bytes)
                    quote!(
                        final $(field_name(field, i))_int = buf.buffer.asByteData(new_offset).getInt32(0);
                        final $(field_name(field, i)) = $(field_ffi_converter_name(field)).lift(toRustBuffer(createUint8ListFromInt($(field_name(field, i))_int)));
                        new_offset += 4;
                    )
                } else {
                    quote!(
                        final $(field_name(field, i))_lifted = $(field_ffi_converter_name(field)).read(Uint8List.view(buf.buffer, new_offset));
                        final $(field_name(field, i)) = $(field_name(field, i))_lifted.value;
                        new_offset += $(field_name(field, i))_lifted.bytesRead;
                    )
                }
            }).collect();

            // Pre-process allocation size calculation
            let allocation_parts: Vec<dart::Tokens> = variant_obj.fields().iter().enumerate().map(|(i, field)| {
                if is_flat_enum(field, type_helper) {
                    quote!(4 + ) // Flat enums are 4 bytes (int32)
                } else {
                    quote!($(field_ffi_converter_name(field)).allocationSize($(field_name(field, i))) + )
                }
            }).collect();

            // Pre-process field write code
            let field_write_code: Vec<dart::Tokens> = variant_obj.fields().iter().enumerate().map(|(i, field)| {
                if is_flat_enum(field, type_helper) {
                    // Handle flat enums specially - lower to RustBuffer and extract int32
                    quote!(
                        final $(field_name(field, i))_buffer = $(field_ffi_converter_name(field)).lower($(field_name(field, i)));
                        final $(field_name(field, i))_int = $(field_name(field, i))_buffer.asUint8List().buffer.asByteData().getInt32(0);
                        buf.buffer.asByteData(new_offset).setInt32(0, $(field_name(field, i))_int);
                        new_offset += 4;
                    )
                } else {
                    quote!(
                        new_offset += $(field_ffi_converter_name(field)).write($(field_name(field, i)), Uint8List.view(buf.buffer, new_offset));
                    )
                }
            }).collect();

            // Generate simple toString() method for error enum variants
            let to_string_method: dart::Tokens =
                if type_helper.get_ci().is_name_used_as_error(obj.name()) {
                    if variant_obj.has_fields() {
                        let field_interpolations = variant_obj
                            .fields()
                            .iter()
                            .enumerate()
                            .map(|(i, field)| format!("${}", field_name(field, i)))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let to_string_with_fields =
                            format!("\"{variant_dart_cls_name}({field_interpolations})\"");
                        quote!(
                            @override
                            String toString() {
                                return $(&to_string_with_fields);
                            }
                        )
                    } else {
                        quote!(
                            @override
                            String toString() {
                                return $(format!("\"{}\"", variant_dart_cls_name));
                            }
                        )
                    }
                } else {
                    quote!()
                };

            variants.push(quote!{
                class $variant_dart_cls_name extends $dart_cls_name {
                    $(for (i, field) in variant_obj.fields().iter().enumerate() => final $(field_type(field, type_helper)) $(field_name(field, i));  )

                    // Add the public const constructor
                    $variant_dart_cls_name($constructor_param_list);

                    // Keep the private constructor used by `read`
                    $variant_dart_cls_name._($(for (i, field) in variant_obj.fields().iter().enumerate() => $(field_type(field, type_helper)) this.$(field_name(field, i)), ));

                    static LiftRetVal<$variant_dart_cls_name> read( Uint8List buf) {
                        int new_offset = buf.offsetInBytes;

                        $(for code in &field_read_code => $code)
                        return LiftRetVal($variant_dart_cls_name._(
                            $(for (i, field) in variant_obj.fields().iter().enumerate() => $(field_name(field, i)),)
                        ), new_offset);
                    }

                    @override
                    RustBuffer lower() {
                        final buf = Uint8List(allocationSize());
                        write(buf);
                        return toRustBuffer(buf);
                    }

                    @override
                    int allocationSize() {
                        return $(for part in &allocation_parts => $part) 4;
                    }

                    @override
                    int write( Uint8List buf) {
                        buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, $(index + 1)); // write index into first position;
                        int new_offset = buf.offsetInBytes + 4;

                        $(for code in &field_write_code => $code)

                        return new_offset;
                    }

                    $to_string_method
                }
            });
        }

        let is_error_enum = type_helper.get_ci().is_name_used_as_error(obj.name());
        let implements_exception =
            if is_error_enum { quote!( implements Exception) } else { quote!() };

        // For error enums, also generate an error handler
        let error_handler_class = if is_error_enum {
            let error_handler_name = format!("{dart_cls_name}ErrorHandler");
            let instance_name = dart_cls_name.to_lower_camel_case();
            quote! {
                class $(&error_handler_name) extends UniffiRustCallStatusErrorHandler {
                    @override
                    Exception lift(RustBuffer errorBuf) {
                        return $ffi_converter_name.lift(errorBuf);
                    }
                }

                final $(&error_handler_name) $(instance_name)ErrorHandler = $(&error_handler_name)();
            }
        } else {
            quote!()
        };

        quote! {
            abstract class $dart_cls_name $implements_exception {
                RustBuffer lower();
                int allocationSize();
                int write( Uint8List buf);
            }

            class $ffi_converter_name {
                static $dart_cls_name lift( RustBuffer buffer) {
                    return $ffi_converter_name.read(buffer.asUint8List()).value;
                }

                static LiftRetVal<$dart_cls_name> read( Uint8List buf) {
                    final index = buf.buffer.asByteData(buf.offsetInBytes).getInt32(0);
                    final subview = Uint8List.view(buf.buffer, buf.offsetInBytes + 4);
                    switch(index) {
                        $(for (index, variant) in obj.variants().iter().enumerate() =>
                        case $(index + 1):
                            final lifted = $(variant_class_name(variant.name())).read(subview);
                            return LiftRetVal<$dart_cls_name>(lifted.value, lifted.bytesRead - subview.offsetInBytes + 4);
                        )
                        default:  throw UniffiInternalError(UniffiInternalError.unexpectedEnumCase, "Unable to determine enum variant");
                    }
                }

                static RustBuffer lower( $dart_cls_name value) {
                    return value.lower();
                }

                static int allocationSize($dart_cls_name value) {
                    return value.allocationSize();
                }

                static int write( $dart_cls_name value, Uint8List buf) {
                    return value.write(buf) - buf.offsetInBytes;
                }
            }

            $(variants)

            $error_handler_class
        }
    }
}
