use genco::lang::dart;
use genco::quote;
use heck::{ToLowerCamelCase, ToUpperCamelCase};
use uniffi_bindgen::interface::ffi::ExternalFfiMetadata;
use uniffi_bindgen::interface::{Argument, ObjectImpl};


use crate::gen::CodeType;
use uniffi_bindgen::interface::{AsType, Callable, FfiType, Type};
use uniffi_bindgen::ComponentInterface;

use crate::gen::primitives;

// use super::render::{AsRenderable, Renderable};
use super::{callback_interface, compounds, custom, enums, objects, records};

pub struct DartCodeOracle;

impl DartCodeOracle {
    pub fn find(type_: &Type) -> Box<dyn CodeType> {
        type_.clone().as_type().as_codetype()
    }

    /// Sanitize a Dart identifier, appending an underscore if it's a reserved keyword.
    pub fn sanitize_identifier(id: &str) -> String {
        if Self::is_reserved_identifier(id) {
            format!("{}_", id)
        } else {
            id.to_string()
        }
    }

    /// Check if the given identifier is a reserved keyword in Dart.
    pub fn is_reserved_identifier(id: &str) -> bool {
        RESERVED_IDENTIFIERS.contains(&id)
    }

    /// Get the idiomatic Dart rendering of a class name (for enums, records, errors, etc).
    pub fn class_name(nm: &str) -> String {
        let name = Self::sanitize_identifier(&nm.to_upper_camel_case());
        // Handle the special case where the name is exactly "Error" to avoid conflicts with Dart's Exception
        if name == "Error" {
            "ErrorException".to_string()
        } else {
            // Only replace "Error" at the end of the name for other cases
            match name.strip_suffix("Error") {
                None => name,
                Some(stripped) => format!("{stripped}Exception"),
            }
        }
    }

    /// Get the idiomatic Dart rendering of a function name.
    pub fn fn_name(nm: &str) -> String {
        Self::sanitize_identifier(&nm.to_lower_camel_case())
    }

    /// Get the idiomatic Dart rendering of a variable name.
    pub fn var_name(nm: &str) -> String {
        Self::sanitize_identifier(&nm.to_lower_camel_case())
    }

    /// Get the idiomatic Dart rendering of an individual enum variant.
    pub fn enum_variant_name(nm: &str) -> String {
        Self::sanitize_identifier(&nm.to_lower_camel_case())
    }

    /// Get the idiomatic Dart rendering of an FFI callback function name
    fn ffi_callback_name(nm: &str) -> String {
        format!(
            "Pointer<NativeFunction<{}>>",
            Self::callback_name(&nm.to_upper_camel_case())
        )
    }

    /// Helper method to generate the callback name based on `Type`.
    fn callback_name(name: &str) -> String {
        format!("Uniffi{}", name.to_upper_camel_case())
    }

    /// Helper method to generate external Dart type labels.
    fn external_type_label(name: &str) -> String {
        format!("External{}", name.to_upper_camel_case())
    }

    /// Helper method to generate external native type labels.
    fn external_native_type_label(name: &str) -> String {
        format!("Pointer<{}>", Self::external_type_label(name))
    }

    /// Get the idiomatic Dart rendering of an exception name
    // pub fn error_name(nm: &str) -> String {
    //     let name = Self::class_name(nm);
    //     match name.strip_suffix("Error") {
    //         None => name,
    //         Some(stripped) => format!("{stripped}Exception"),
    //     }
    // }

    pub fn find_lib_instance() -> dart::Tokens {
        quote!(_UniffiLib.instance)
    }

    /// Helper method to fully qualify imports of external `RustBuffer`s
    fn rust_buffer_name(meta: &Option<ExternalFfiMetadata>, ci: &ComponentInterface) -> dart::Tokens {
        if let Some(meta) = meta {
            return Self::rust_buffer_name_with_path(&meta.module_path, ci);
        }
        quote!(RustBuffer)
    }

    /// Helper method to fully qualify imports of external `RustBuffer`s
    fn rust_buffer_name_with_path(module_path: &str, ci: &ComponentInterface) -> dart::Tokens {
        let namespace = ci.namespace_for_module_path(module_path).expect("module path should exist");
        if namespace != ci.namespace() {
            return quote!($(namespace).RustBuffer);
        }
        quote!(RustBuffer)
    }

    // TODO: Replace instances of `generate_ffi_dart_type` with ffi_type_label
    pub fn ffi_dart_type_label(ffi_type: Option<&FfiType>, ci: &ComponentInterface) -> dart::Tokens {
        if let Some(ret_type) = ffi_type {
            match ret_type {
                FfiType::Int8 => quote!(int),
                FfiType::UInt8 => quote!(int),
                FfiType::Int16 => quote!(int),
                FfiType::UInt16 => quote!(int),
                FfiType::Int32 => quote!(int),
                FfiType::UInt32 => quote!(int),
                FfiType::Int64 => quote!(int),
                FfiType::UInt64 => quote!(int),
                FfiType::Float32 => quote!(double),
                FfiType::Float64 => quote!(double),
                FfiType::RustArcPtr(_) => quote!(Pointer<Void>),
                FfiType::RustBuffer(ext) => Self::rust_buffer_name(ext, ci),
                FfiType::ForeignBytes => quote!(ForeignBytes),
                FfiType::Handle => quote!(Pointer<Void>),
                FfiType::Callback(name) => quote!($(Self::ffi_callback_name(name))),
                FfiType::Reference(inner) => quote!($(Self::ffi_type_label_by_reference(inner))),
                _ => panic!("Unimplemented FfiType: {:?}", ret_type), // Fallback implementation
            }
        } else {
            quote!(void)
        }
    }

    pub fn ffi_native_type_label(ffi_ret_type: Option<&FfiType>, ci: &ComponentInterface) -> dart::Tokens {
        if let Some(ret_type) = ffi_ret_type {
            match ret_type {
                FfiType::Int8 => quote!(Int8),
                FfiType::UInt8 => quote!(Uint8),
                FfiType::Int16 => quote!(Int16),
                FfiType::UInt16 => quote!(Uint16),
                FfiType::Int32 => quote!(Int32),
                FfiType::UInt32 => quote!(Uint32),
                FfiType::Int64 => quote!(Int64),
                FfiType::UInt64 => quote!(Uint64),
                FfiType::Float32 => quote!(Float),
                FfiType::Float64 => quote!(Double),
                FfiType::RustArcPtr(_) => quote!(Pointer<Void>),
                FfiType::RustBuffer(ext) => Self::rust_buffer_name(ext, ci),
                FfiType::ForeignBytes => quote!(ForeignBytes),
                FfiType::Handle => quote!(Pointer<Void>),
                FfiType::Callback(name) => quote!($(Self::ffi_callback_name(name))),
                FfiType::Reference(inner) => quote!($(Self::ffi_type_label_by_reference(inner))),
                _ => panic!("Unimplemented FfiType: {:?}", ret_type), // Fallback implementation
            }
        } else {
            quote!(void)
        }
    }

    fn ffi_type_label_by_reference(ffi_type: &FfiType) -> dart::Tokens {
        match ffi_type {
            FfiType::Int8 => quote!(Int8),
            FfiType::UInt8 => quote!(Uint8),
            FfiType::Int16 => quote!(Int16),
            FfiType::UInt16 => quote!(Uint16),
            FfiType::Int32 => quote!(Int32),
            FfiType::UInt32 => quote!(Uint32),
            FfiType::Int64 => quote!(Int64),
            FfiType::UInt64 => quote!(Uint64),
            FfiType::Float32 => quote!(Float),
            FfiType::Float64 => quote!(Double),
            FfiType::RustArcPtr(_) => quote!(Pointer<Void>),
            FfiType::RustBuffer(_) => quote!(RustBuffer),
            FfiType::Callback(name) => quote!(Pointer<$(Self::ffi_callback_name(name))>),
            FfiType::Struct(name) => quote!(Pointer<$(Self::ffi_struct_name(name))>),
            _ => quote!(Pointer<Void>), // Fallback implementation
        }
    }

    pub fn ffi_struct_name(name: &str) -> dart::Tokens {
       quote!($(format!("Uniffi{}", name.to_upper_camel_case())))
    }

    
    // pub fn convert_from_rust_buffer(ty: &Type, inner: dart::Tokens) -> dart::Tokens {
    //     match ty {
    //         Type::Object { .. } => inner,
    //         Type::String | Type::Optional { .. } => quote!($(inner).asUint8List()),
    //         _ => inner,
    //     }
    // }

    // pub fn convert_to_rust_buffer(ty: &Type, inner: dart::Tokens) -> dart::Tokens {
    //     match ty {
    //         Type::Object { .. } => inner,
    //         Type::String | Type::Optional { .. } | Type::Enum { .. } | Type::Sequence { .. } => {
    //             quote!(toRustBuffer($inner))
    //         }
    //         _ => inner,
    //     }
    // }


    
    pub fn type_lower_fn(ty: &Type, inner: dart::Tokens) -> dart::Tokens {
        match ty {
            Type::UInt32
            | Type::Int8
            | Type::UInt8
            | Type::Int16
            | Type::UInt16
            | Type::Int32
            | Type::Int64
            | Type::UInt64
            | Type::Float32
            | Type::Float64 => inner,
            Type::Boolean
            | Type::Duration
            | Type::String
            | Type::Object { .. }
            | Type::Enum { .. }
            | Type::Optional { .. }
            | Type::Record { .. } => {
                quote!($(ty.as_codetype().ffi_converter_name()).lower($inner))
            }
            _ => quote!($(ty.as_codetype().ffi_converter_name()).lower($inner)), // Fallback implementation
        }
    }

    pub fn async_poll(callable: impl Callable, ci: &ComponentInterface) -> dart::Tokens {
        let ffi_func = callable.ffi_rust_future_poll(ci);
        quote!($(Self::find_lib_instance()).$ffi_func)
    }

    pub fn async_complete(callable: impl Callable, ci: &ComponentInterface) -> dart::Tokens {
        let ffi_func = callable.ffi_rust_future_complete(ci);
        let call = quote!($(Self::find_lib_instance()).$ffi_func);
        call
    }

    pub fn async_free(callable: impl Callable, ci: &ComponentInterface) -> dart::Tokens {
        let ffi_func = callable.ffi_rust_future_free(ci);
        quote!($(Self::find_lib_instance()).$ffi_func)
    }

    /// Get the idiomatic Dart rendering of a class name based on `Type`.
    pub fn dart_type_label(type_: Option<&Type>) -> dart::Tokens {
        if let Some(ret_type) = type_ {
            match ret_type {
                Type::UInt8
                | Type::UInt16
                | Type::UInt32
                | Type::UInt64
                | Type::Int8
                | Type::Int16
                | Type::Int32
                | Type::Int64 => quote!(int),
                Type::Float32 | Type::Float64 => quote!(double),
                Type::Boolean => quote!(bool),
                Type::Bytes => quote!(Uint8List),
                Type::String => quote!(String),
                Type::Timestamp => quote!(DateTime),
                Type::Duration => quote!(Duration),
                // Reference types
                Type::Object { name, .. } => {
                    let class_name = &DartCodeOracle::class_name(name);
                    quote!($class_name)
                }
                Type::Optional { inner_type } => {
                    let inner = DartCodeOracle::dart_type_label(Some(inner_type));
                    quote!($inner?)
                }
                Type::Sequence { inner_type } => {
                    let inner = DartCodeOracle::dart_type_label(Some(inner_type));
                    quote!(List<$inner>)
                }
                Type::Map { key_type, value_type, .. } => {
                    let key = DartCodeOracle::dart_type_label(Some(key_type));
                    let value = DartCodeOracle::dart_type_label(Some(value_type));
                    quote!(Map<$key, $value>)
                }
                Type::Enum { name, .. } => {
                    let enum_name = &DartCodeOracle::class_name(name);
                    quote!($enum_name)
                }
                Type::Record { name, .. } => {
                    let rec_name = &DartCodeOracle::class_name(name);
                    quote!($rec_name)
                }
                Type::Custom { name, .. } => {
                    let type_name = &DartCodeOracle::class_name(name);
                    quote!($type_name)
                }
                _ => quote!(dynamic),
            }
        } else {
            quote!(void)
        }
    }

    /// Get the native Dart FFI type rendering based on `Type`.
    pub fn native_type_label(native_ret_type: Option<&Type>, ci: &ComponentInterface) -> dart::Tokens {
        if let Some(ret_type) = native_ret_type {
            match ret_type {
                Type::UInt8 => quote!(Uint8),
                Type::UInt16 => quote!(Uint16),
                Type::UInt32 => quote!(Uint32),
                Type::UInt64 => quote!(Uint64),
                Type::Int8 => quote!(Int8),
                Type::Int16 => quote!(Int16),
                Type::Int32 => quote!(Int32),
                Type::Int64 => quote!(Int64),
                Type::Float32 => quote!(Float),
                Type::Float64 => quote!(Double),
                Type::Boolean => quote!(Int8),
                Type::Bytes => quote!(RustBuffer),
                Type::String => quote!(RustBuffer),
                Type::Timestamp => quote!(Int64),
                Type::Duration => quote!(Int64),
                Type::Optional { inner_type } => match **inner_type {
                    Type::String => quote!(RustBuffer),
                    _ => quote!(RustBuffer),
                },
                Type::Sequence { .. } => quote!(RustBuffer),
                Type::Map { .. } => quote!(RustBuffer),
                Type::Object { .. } => quote!(Pointer<Void>),
                Type::Enum { .. } => quote!(Int32),
                Type::Record { module_path, .. } => Self::rust_buffer_name_with_path(module_path, ci),
                Type::Custom { name, .. } => {
                    let class_name = &DartCodeOracle::class_name(name);
                    quote!($class_name)
                },
                _ => quote!(Pointer<Void>),
            }
        } else {
            quote!(Void)
        }
    }

    /// Get the native Dart FFI type rendering based on `Type`.
    pub fn native_dart_type_label(native_ret_type: Option<&Type>, ci: &ComponentInterface) -> dart::Tokens {
        if let Some(ret_type) = native_ret_type {
            match ret_type {
                Type::UInt8
                | Type::UInt16
                | Type::UInt32
                | Type::UInt64
                | Type::Int8
                | Type::Int16
                | Type::Int32
                | Type::Int64 => quote!(int), // Adjust based on actual FFI size
                Type::Float32 => quote!(double),
                Type::Float64 => quote!(double),
                Type::Boolean => quote!(int),
                Type::Bytes => quote!(RustBuffer),
                Type::String => quote!(RustBuffer),
                Type::Timestamp => quote!(int),
                Type::Duration => quote!(int),
                Type::Optional { inner_type } => match **inner_type {
                    Type::String => quote!(RustBuffer),
                    _ => quote!(RustBuffer),
                },
                Type::Sequence { .. } => quote!(RustBuffer),
                Type::Map { .. } => quote!(RustBuffer),
                Type::Object { .. } => quote!(Pointer<Void>),
                Type::Enum { .. } => quote!(int),
                Type::Record { module_path, .. } => Self::rust_buffer_name_with_path(module_path, ci),
                Type::Custom { name, .. } => {
                    let type_name = &DartCodeOracle::class_name(name);
                    quote!($type_name)
                },
                _ => quote!(dynamic),
            }
        } else {
            quote!(void)
        }
    }

    // Method to get the appropriate callback parameter type
    pub fn callback_param_type(arg_type: &Type, arg_name: &str, ci: &ComponentInterface) -> dart::Tokens {
        match arg_type {
            Type::Boolean => quote!(int $arg_name),
            Type::Bytes => quote!(RustBuffer $(arg_name)Buffer),
            Type::String => quote!(RustBuffer $(arg_name)Buffer),
            Type::Optional { inner_type } => {
                if let Type::String = **inner_type {
                    quote!(RustBuffer $(arg_name)Buffer)
                } else {
                    //let type_label = DartCodeOracle::dart_type_label(Some(arg_type));
                    quote!(RustBuffer $arg_name)
                }
            },
            Type::Sequence { inner_type } => {
                if let Type::Int32 = **inner_type {
                    quote!(RustBuffer $(arg_name)Buffer)
                } else {
                    //let type_label = DartCodeOracle::dart_type_label(Some(arg_type));
                    quote!(RustBuffer $arg_name)
                }
            },
            Type::Record { module_path, .. } => quote!($(Self::rust_buffer_name_with_path(module_path, ci)) $arg_name),
            _ => {
                let type_label = DartCodeOracle::dart_type_label(Some(arg_type));
                quote!($type_label $arg_name)
            }
        }
    }

    // Method to generate code for handling callback return values
    pub fn callback_return_handling(ret_type: &Type, method_name: &str, args: Vec<dart::Tokens>) -> dart::Tokens {
        match ret_type {
            Type::Boolean => {
                // For boolean return values
                quote!(
                    final result = obj.$method_name($(for arg in &args => $arg,));
                    outReturn.value = result ? 1 : 0;
                )
            },
            Type::Optional { inner_type } => {
                // For optional return values
                if let Type::String = **inner_type {
                    quote!(
                        final result = obj.$method_name($(for arg in &args => $arg,));
                        if (result == null) {
                            outReturn.ref = toRustBuffer(Uint8List.fromList([0]));
                        } else {
                            final lowered = FfiConverterOptionalString.lower(result);
                            outReturn.ref = toRustBuffer(lowered.asUint8List());
                        }
                    )
                } else {
                    let lowered = ret_type.as_codetype().ffi_converter_name();
                    quote!(
                        final result = obj.$method_name($(for arg in &args => $arg,));
                        if (result == null) {
                            outReturn.ref = toRustBuffer(Uint8List.fromList([0]));
                        } else {
                            final lowered = $lowered.lower(result);
                            final buffer = Uint8List(1 + lowered.length);
                            buffer[0] = 1;
                            buffer.setAll(1, lowered.asUint8List());
                            outReturn.ref = toRustBuffer(buffer);
                        }
                    )
                }
            },
            Type::String => {
                // For string return values
                quote!(
                    final result = obj.$method_name($(for arg in &args => $arg,));
                    outReturn.ref = FfiConverterString.lower(result);
                    status.code = CALL_SUCCESS;
                )
            },
            Type::Sequence { inner_type } => {
                if let Type::Int32 = **inner_type {
                    // For int32 sequence return values
                    quote!(
                        final result = obj.$method_name($(for arg in &args => $arg,));
                        outReturn.ref = FfiConverterSequenceInt32.lower(result);
                    )
                } else {
                    // For other sequence types
                    let lowered = ret_type.as_codetype().ffi_converter_name();
                    quote!(
                        final result = obj.$method_name($(for arg in &args => $arg,));
                        outReturn.ref = $lowered.lower(result);
                    )
                }
            },
            _ => {
                // For other return types
                let lowered = ret_type.as_codetype().ffi_converter_name();
                quote!(
                    final result = obj.$method_name($(for arg in &args => $arg,));
                    outReturn.ref = $lowered.lower(result);
                )
            }
        }
    }

    // Method to get the appropriate return type for callback functions
    pub fn callback_out_return_type(ret_type: Option<&Type>) -> dart::Tokens {
        if let Some(ret) = ret_type {
            match ret {
                Type::Boolean => quote!(Pointer<Int8>),
                _ => quote!(Pointer<RustBuffer>)
            }
        } else {
            quote!(Pointer<Void>)
        }
    }

    // Method to handle void return values in callbacks
    pub fn callback_void_handling(method_name: &str, args: Vec<dart::Tokens>) -> dart::Tokens {
        quote!(
            obj.$method_name($(for arg in &args => $arg,));
            status.code = CALL_SUCCESS;
        )
    }

    // Method to get the appropriate lift expression for callback arguments with indexed variable names
    pub fn callback_arg_lift_indexed(arg_type: &Type, arg_name: &str, arg_idx: usize) -> dart::Tokens {
        // Use index-based variable names to avoid conflicts
        if let Type::Boolean = arg_type {
            quote!(final bool_arg$(arg_idx) = $arg_name == 1;)
        } else if let Type::Bytes = arg_type {
            quote!(final arg$(arg_idx) = FfiConverterUint8List.lift($(arg_name)Buffer);)
        } else if let Type::String = arg_type {
            quote!(final arg$(arg_idx) = FfiConverterString.lift($(arg_name)Buffer);)
        } else if let Type::Optional { inner_type } = arg_type {
            if let Type::String = **inner_type {
                quote!(final arg$(arg_idx) = FfiConverterOptionalString.lift($(arg_name)Buffer);)
            } else {
                let converter = arg_type.as_codetype().ffi_converter_name();
                quote!(final arg$(arg_idx) = $converter.lift($arg_name);)
            }
        } else if let Type::Sequence { inner_type } = arg_type {
            if let Type::Int32 = **inner_type {
                quote!(final arg$(arg_idx) = FfiConverterSequenceInt32.lift($(arg_name)Buffer);)
            } else {
                let converter = arg_type.as_codetype().ffi_converter_name();
                quote!(final arg$(arg_idx) = $converter.lift($arg_name);)
            }
        } else {
            let converter = arg_type.as_codetype().ffi_converter_name();
            quote!(final arg$(arg_idx) = $converter.lift($arg_name);)
        }
    }

    // Method to get argument name for a callback method based on type and index
    pub fn callback_arg_name(arg_type: &Type, arg_idx: usize) -> dart::Tokens {
        if let Type::Boolean = arg_type {
            quote!(bool_arg$(arg_idx))
        } else {
            quote!(arg$(arg_idx))
        }
    }

    /// Lower argument with special handling for callback traits
    pub fn lower_arg_with_callback_handling(arg: &Argument) -> dart::Tokens {
        let base_lower = Self::type_lower_fn(&arg.as_type(), quote!($(Self::var_name(arg.name()))));
        match arg.as_type() {
            Type::Object { imp, .. } if imp == ObjectImpl::CallbackTrait => {
                quote!(Pointer<Void>.fromAddress($(base_lower)))
            }
            _ => base_lower
        }
    }

}

// https://dart.dev/guides/language/language-tour#keywords
pub static RESERVED_IDENTIFIERS: [&str; 63] = [
    "abstract",
    "as",
    "assert",
    "async",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "covariant",
    "default",
    "deferred",
    "do",
    "dynamic",
    "else",
    "enum",
    "export",
    "extends",
    "extension",
    "external",
    "factory",
    "false",
    "final",
    "finally",
    "for",
    "Function",
    "get",
    "hide",
    "if",
    "implements",
    "import",
    "in",
    "interface",
    "is",
    "late",
    "library",
    "mixin",
    "new",
    "null",
    "on",
    "operator",
    "part",
    "required",
    "rethrow",
    "return",
    "set",
    "show",
    "static",
    "super",
    "switch",
    "sync",
    "this",
    "throw",
    "true",
    "try",
    "typedef",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

pub trait AsCodeType {
    fn as_codetype(&self) -> Box<dyn CodeType>;
}

impl<T: AsType> AsCodeType for T {
    fn as_codetype(&self) -> Box<dyn CodeType> {
        match self.as_type() {
            Type::UInt8 => Box::new(primitives::UInt8CodeType),
            Type::Int8 => Box::new(primitives::Int8CodeType),
            Type::UInt16 => Box::new(primitives::UInt16CodeType),
            Type::Int16 => Box::new(primitives::Int16CodeType),
            Type::UInt32 => Box::new(primitives::UInt32CodeType),
            Type::Int32 => Box::new(primitives::Int32CodeType),
            Type::UInt64 => Box::new(primitives::UInt64CodeType),
            Type::Int64 => Box::new(primitives::Int64CodeType),
            Type::Float32 => Box::new(primitives::Float32CodeType),
            Type::Float64 => Box::new(primitives::Float64CodeType),
            Type::Boolean => Box::new(primitives::BooleanCodeType),
            Type::String => Box::new(primitives::StringCodeType),
            Type::Duration => Box::new(primitives::DurationCodeType),
            Type::Bytes => Box::new(primitives::BytesCodeType),
            Type::Object { name, imp, .. } => Box::new(objects::ObjectCodeType::new(name, imp)),
            Type::Optional { inner_type } => Box::new(compounds::OptionalCodeType::new(
                self.as_type(),
                *inner_type,
            )),
            Type::Sequence { inner_type } => Box::new(compounds::SequenceCodeType::new(
                self.as_type(),
                *inner_type,
            )),
            Type::Enum { name, .. } => Box::new(enums::EnumCodeType::new(name)),
            Type::Record {name, .. } => Box::new(records::RecordCodeType::new(name)),
            Type::CallbackInterface { name, .. } => Box::new(callback_interface::CallbackInterfaceCodeType::new(name, self.as_type())),
            Type::Custom { name, module_path, builtin } => Box::new(custom::CustomCodeType::new(name, module_path, builtin)),
            _ => todo!("As Type for Type::{:?}", self.as_type()),
        }
    }
}
