use crate::gen::CodeType;
use genco::prelude::*;
use heck::ToUpperCamelCase;
use uniffi_bindgen::interface::Type;
use uniffi_bindgen::interface::{
    ffi::{FfiStruct, FfiType},
    AsType, Method,
};

use crate::gen::oracle::{AsCodeType, DartCodeOracle};
use crate::gen::render::AsRenderable;
use crate::gen::render::{Renderable, TypeHelperRenderer};

#[derive(Debug)]
pub struct CallbackInterfaceCodeType {
    name: String,
    self_type: Type,
}

impl CallbackInterfaceCodeType {
    pub fn new(name: String, self_type: Type) -> Self {
        Self { name, self_type }
    }
}

impl CodeType for CallbackInterfaceCodeType {
    fn type_label(&self) -> String {
        super::DartCodeOracle::class_name(&self.name)
    }

    fn canonical_name(&self) -> String {
        format!("CallbackInterface{}", self.type_label())
    }

    fn initialization_fn(&self) -> Option<String> {
        Some(format!("_uniffiInitializeCallbackInterface{}", self.name))
    }
}

impl Renderable for CallbackInterfaceCodeType {
    fn render_type_helper(&self, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
        type_helper.include_once_check(&self.canonical_name(), &self.self_type);
        let callback = type_helper
            .get_ci()
            .get_callback_interface_definition(&self.name)
            .unwrap();

        // Generate all necessary components for the callback interface
        let interface = generate_callback_interface(
            callback.name(),
            &callback.as_codetype().ffi_converter_name(),
            &callback.methods(),
            type_helper,
            None,
        );
        let vtable_interface =
            generate_callback_vtable_interface(callback.name(), &callback.methods());
        let functions =
            generate_callback_functions(callback.name(), &callback.methods(), type_helper);
        let namespace = type_helper
            .get_ci()
            .namespace_for_type(&callback.as_type())
            .unwrap_or_else(|_| type_helper.get_ci().namespace())
            .to_string();
        let ffi_module = DartCodeOracle::infer_ffi_module(type_helper.get_ci(), move || namespace);
        let vtable_init = generate_callback_interface_vtable_init_function(
            callback.name(),
            &callback.methods(),
            &ffi_module,
        );

        quote! {
            $interface
            $vtable_interface
            $functions
            $vtable_init
        }
    }
}

pub fn generate_callback_interface(
    callback_name: &str,
    ffi_converter_name: &str,
    methods: &[&Method],
    type_helper: &dyn TypeHelperRenderer,
    rust_impl_name: Option<&str>,
) -> dart::Tokens {
    let cls_name = &DartCodeOracle::class_name(callback_name);
    let ffi_conv_name = &DartCodeOracle::class_name(ffi_converter_name);
    let init_fn_name = &format!("init{callback_name}VTable");
    let lift_rust_impl = rust_impl_name
        .map(|name| quote!(return $name._internal(handle);))
        .unwrap_or_else(|| {
            quote!(
                throw UniffiInternalError(
                    UniffiInternalError.unexpectedStaleHandle,
                    "Rust-owned callback interface handle is not supported for $cls_name",
                );
            )
        });
    let lower_rust_impl = rust_impl_name
        .map(|name| {
            quote!(
                if (value is $name) {
                    return value.uniffiClonePointer();
                }
            )
        })
        .unwrap_or_else(|| quote!());

    // TODO: Use global deduplication to avoid generating duplicate async types
    // when multiple async callback interfaces are defined
    let mut async_struct_defs: Vec<dart::Tokens> = Vec::new();
    let mut async_completion_typedefs: Vec<dart::Tokens> = Vec::new();

    for method in methods {
        if method.is_async() {
            let struct_def = method.foreign_future_ffi_result_struct();
            let struct_name = struct_def.name().to_string();

            if !type_helper.include_once_by_name(&struct_name) {
                async_struct_defs.push(generate_foreign_future_struct_definition(
                    &struct_def,
                    type_helper,
                ));
            }

            let completion_name = foreign_future_completion_name(method);
            if !type_helper.include_once_by_name(&completion_name) {
                async_completion_typedefs.push(generate_foreign_future_completion_typedef(
                    &completion_name,
                    &struct_name,
                ));
            }
        }
    }

    let async_support = if !async_struct_defs.is_empty() || !async_completion_typedefs.is_empty() {
        quote! {
            $(for typedef in &async_completion_typedefs => $typedef)
            $(for struct_def in &async_struct_defs => $struct_def)
        }
    } else {
        quote!()
    };

    let tokens = quote! {
        // This is the abstract class to be implemented
        abstract class $cls_name {
            $(for m in methods {
                $(generate_callback_methods_definitions(m, type_helper))
            })
        }

        // This is the type helper to convert from FFI to Dart
        class $ffi_conv_name {
            static final _handleMap = UniffiHandleMap<$cls_name>();
            static bool _vtableInitialized = false;

            static $cls_name lift(Pointer<Void> handle) {
                final rawHandle = handle.address;
                if ((rawHandle & 0x1) == 0) {
                    $lift_rust_impl
                }
                return _handleMap.remove(rawHandle);
            }

            static Pointer<Void> lower($cls_name value) {
                $lower_rust_impl
                _ensureVTableInitialized();
                final handle = _handleMap.insert(value);
                return Pointer<Void>.fromAddress(handle);
            }

            static void _ensureVTableInitialized() {
                if (!_vtableInitialized) {
                    $init_fn_name();
                    _vtableInitialized = true;
                }
            }

            static LiftRetVal<$cls_name> read(Uint8List buf) {
                final handle = buf.buffer.asByteData(buf.offsetInBytes).getInt64(0);
                final pointer = Pointer<Void>.fromAddress(handle);
                return LiftRetVal(lift(pointer), 8);
            }

            static int write($cls_name value, Uint8List buf) {
                final handle = lower(value);
                buf.buffer.asByteData(buf.offsetInBytes).setInt64(0, handle.address);
                return 8;
            }

            static int allocationSize($cls_name value) {
                return 8; // Just a handle (int64).
            }
        }

        // Additional support definitions for async callbacks
        $async_support

        // We must define callback signatures
        $(generate_callback_methods_signatures(cls_name, methods, type_helper))
    };

    tokens
}

fn generate_callback_methods_definitions(
    method: &Method,
    type_helper: &dyn TypeHelperRenderer,
) -> dart::Tokens {
    let method_name = DartCodeOracle::fn_name(method.name());
    let dart_args = &method
        .arguments()
        .iter()
        .map(|arg| {
            let arg_type = arg.as_renderable().render_type(&arg.as_type(), type_helper);
            let arg_name = DartCodeOracle::var_name(arg.name());

            quote!($arg_type $arg_name)
        })
        .collect::<Vec<_>>();

    let ret_type = if method.is_async() {
        if let Some(ret) = method.return_type() {
            let rendered = ret.as_renderable().render_type(ret, type_helper);
            quote!(Future<$rendered>)
        } else {
            quote!(Future<void>)
        }
    } else if let Some(ret) = method.return_type() {
        ret.as_renderable().render_type(ret, type_helper)
    } else {
        quote!(void)
    };

    quote!(
        $ret_type $method_name($(for a in dart_args => $a,));
    )
}

fn generate_callback_methods_signatures(
    callback_name: &str,
    methods: &[&Method],
    type_helper: &dyn TypeHelperRenderer,
) -> dart::Tokens {
    let mut tokens = dart::Tokens::new();
    for (method_index, method) in methods.iter().enumerate() {
        //let method_name = DartCodeOracle::fn_name(method.name());

        let ffi_method_type = format!("UniffiCallbackInterface{callback_name}Method{method_index}");
        let dart_method_type =
            format!("UniffiCallbackInterface{callback_name}Method{method_index}Dart");

        let arg_native_types: Vec<dart::Tokens> = method
            .arguments()
            .iter()
            .map(|arg| {
                DartCodeOracle::native_type_label(Some(&arg.as_type()), type_helper.get_ci())
            })
            .collect();

        let arg_dart_types: Vec<dart::Tokens> = method
            .arguments()
            .iter()
            .map(|arg| {
                DartCodeOracle::native_dart_type_label(Some(&arg.as_type()), type_helper.get_ci())
            })
            .collect();

        if method.is_async() {
            let completion_base = foreign_future_completion_name(method);
            let completion_native = format!("Uniffi{}", completion_base.to_upper_camel_case());
            let completion_pointer = format!("Pointer<NativeFunction<{}>>", completion_native);

            tokens.append(quote! {
                typedef $ffi_method_type = Void Function(
                    Uint64, $(for arg in &arg_native_types => $arg,)
                    $(&completion_pointer), Uint64, Pointer<UniffiForeignFuture>);
                typedef $dart_method_type = void Function(
                    int, $(for arg in &arg_dart_types => $arg,)
                    $(&completion_pointer), int, Pointer<UniffiForeignFuture>);
            });
        } else {
            let method_return_type = if let Some(ret) = method.return_type() {
                DartCodeOracle::native_type_label(Some(ret), type_helper.get_ci())
            } else {
                quote!(Void)
            };

            tokens.append(quote! {
                typedef $ffi_method_type = Void Function(
                    Uint64, $(for arg in &arg_native_types => $arg,)
                    Pointer<$(&method_return_type)>, Pointer<RustCallStatus>);
                typedef $dart_method_type = void Function(
                    int, $(for arg in &arg_dart_types => $arg,)
                    Pointer<$(&method_return_type)>, Pointer<RustCallStatus>);
            });
        }
    }

    tokens.append(quote! {
        typedef UniffiCallbackInterface$(callback_name)Free = Void Function(Uint64);
        typedef UniffiCallbackInterface$(callback_name)FreeDart = void Function(int);
        typedef UniffiCallbackInterface$(callback_name)Clone = Uint64 Function(Uint64);
        typedef UniffiCallbackInterface$(callback_name)CloneDart = int Function(int);
    });

    tokens
}

pub fn generate_callback_vtable_interface(
    callback_name: &str,
    methods: &[&Method],
) -> dart::Tokens {
    let vtable_name = format!("UniffiVTableCallbackInterface{callback_name}");
    let methods_vec: Vec<_> = methods.iter().enumerate().collect();

    quote! {
        final class $vtable_name extends Struct {
            external Pointer<NativeFunction<UniffiCallbackInterface$(callback_name)Free>> uniffiFree;
            external Pointer<NativeFunction<UniffiCallbackInterface$(callback_name)Clone>> uniffiClone;
            $(for (index, m) in &methods_vec =>
                external Pointer<NativeFunction<UniffiCallbackInterface$(callback_name)Method$(format!("{}",index))>> $(DartCodeOracle::fn_name(m.name()));
            )
        }
    }
}

pub fn generate_callback_functions(
    callback_name: &str,
    methods: &[&Method],
    type_helper: &dyn TypeHelperRenderer,
) -> dart::Tokens {
    let cls_name = &DartCodeOracle::class_name(callback_name);

    let functions: Vec<dart::Tokens> = methods.iter().enumerate().map(|(index, m)| {
        let method_name = &DartCodeOracle::fn_name(m.name()).to_string();
        let ffi_method_type = &format!("UniffiCallbackInterface{callback_name}Method{index}");
        let _dart_method_type = &format!("UniffiCallbackInterface{callback_name}Method{index}Dart");

        // Get parameter types using the oracle
        let param_types: Vec<dart::Tokens> = m
            .arguments()
            .iter()
            .map(|arg| {
                let arg_name = DartCodeOracle::var_name(arg.name());
                DartCodeOracle::callback_param_type(&arg.as_type(), &arg_name, type_helper.get_ci())
            })
            .collect();

        // Get argument lifts using the oracle
        let arg_lifts: Vec<dart::Tokens> = m
            .arguments()
            .iter()
            .enumerate()
            .map(|(arg_idx, arg)| {
                let arg_name = DartCodeOracle::var_name(arg.name());
                DartCodeOracle::callback_arg_lift_indexed(&arg.as_type(), &arg_name, arg_idx)
            })
            .collect();

        // Prepare arg names for the method call using indexes
        let arg_names: Vec<dart::Tokens> = m
            .arguments()
            .iter()
            .enumerate()
            .map(|(arg_idx, arg)| DartCodeOracle::callback_arg_name(&arg.as_type(), arg_idx))
            .collect();

        // Generate the function body
        let callback_method_name =
            &format!("{}{}", &DartCodeOracle::fn_name(callback_name), &DartCodeOracle::class_name(m.name()));

        if m.is_async() {
            let completion_base = foreign_future_completion_name(m);
            let completion_native = format!("Uniffi{}", completion_base.to_upper_camel_case());
            let completion_pointer = format!("Pointer<NativeFunction<{}>>", completion_native);
            let completion_dart = format!("{completion_native}Dart");
            let result_struct = m.foreign_future_ffi_result_struct();
            let struct_tokens = DartCodeOracle::ffi_struct_name(result_struct.name());
            let struct_tokens_alt = struct_tokens.clone();

            if let Some(ret) = m.return_type() {
                type_helper.include_once_check(&ret.as_codetype().canonical_name(), ret);
            }

            let success_return = if let Some(ret) = m.return_type() {
                let converter = ret.as_codetype().ffi_converter_name();
                quote!(resultStructPtr.ref.returnValue = $(&converter).lower(result);)
            } else {
                quote!()
            };

            quote! {
                void $callback_method_name(
                    int uniffiHandle,
                    $(for param in &param_types => $param,)
                    $(&completion_pointer) uniffiFutureCallback,
                    int uniffiCallbackData,
                    Pointer<UniffiForeignFuture> outReturn,
                ) {
                    final obj = FfiConverterCallbackInterface$cls_name._handleMap.get(uniffiHandle);
                    $(arg_lifts)
                    final callback = uniffiFutureCallback.asFunction<$(&completion_dart)>();
                    final state = _UniffiForeignFutureState();
                    final handle = _uniffiForeignFutureHandleMap.insert(state);
                    outReturn.ref.handle = handle;
                    outReturn.ref.free = _uniffiForeignFutureFreePointer;

                    () async {
                        try {
                            final result = await obj.$method_name($(for arg in &arg_names => $arg,));
                            final removedState = _uniffiForeignFutureHandleMap.maybeRemove(handle);
                            final effectiveState = removedState ?? state;
                            if (effectiveState.cancelled) {
                                return;
                            }
                            effectiveState.cancelled = true;
                            final resultStructPtr = calloc<$struct_tokens>();
                            try {
                                $success_return
                                resultStructPtr.ref.callStatus.code = CALL_SUCCESS;
                                callback(uniffiCallbackData, resultStructPtr.ref);
                            } finally {
                                calloc.free(resultStructPtr);
                            }
                        } catch (e) {
                            final removedState = _uniffiForeignFutureHandleMap.maybeRemove(handle);
                            final effectiveState = removedState ?? state;
                            if (effectiveState.cancelled) {
                                return;
                            }
                            effectiveState.cancelled = true;
                            final resultStructPtr = calloc<$struct_tokens_alt>();
                            try {
                                resultStructPtr.ref.callStatus.code = CALL_UNEXPECTED_ERROR;
                                resultStructPtr.ref.callStatus.errorBuf =
                                    FfiConverterString.lower(e.toString());
                                callback(uniffiCallbackData, resultStructPtr.ref);
                            } finally {
                                calloc.free(resultStructPtr);
                            }
                        }
                    }();
                }

                final Pointer<NativeFunction<$ffi_method_type>> $(callback_method_name)Pointer =
                    Pointer.fromFunction<$ffi_method_type>($callback_method_name);
            }
        } else {
            // Handle return value using the oracle
            let call_dart_method = if let Some(ret) = m.return_type() {
                DartCodeOracle::callback_return_handling(ret, method_name, arg_names)
            } else {
                // Handle void return types
                DartCodeOracle::callback_void_handling(method_name, arg_names)
            };

            // Get the appropriate out return type
            let out_return_type = DartCodeOracle::callback_out_return_type(m.return_type());

            quote! {
                void $callback_method_name(int uniffiHandle, $(for param in &param_types => $param,) $out_return_type outReturn, Pointer<RustCallStatus> callStatus) {
                    final status = callStatus.ref;
                    try {
                        final obj = FfiConverterCallbackInterface$cls_name._handleMap.get(uniffiHandle);
                        $(arg_lifts)
                        $call_dart_method
                    } catch (e) {
                        status.code = CALL_UNEXPECTED_ERROR;
                        status.errorBuf = FfiConverterString.lower(e.toString());
                    }
                }

                final Pointer<NativeFunction<$ffi_method_type>> $(callback_method_name)Pointer =
                    Pointer.fromFunction<$ffi_method_type>($callback_method_name);
            }
        }
    }).collect();

    // Free callback
    let free_callback_fn = &format!("{}FreeCallback", DartCodeOracle::fn_name(callback_name));
    let free_callback_pointer = &format!("{}FreePointer", DartCodeOracle::fn_name(callback_name));
    let free_callback_type = &format!("UniffiCallbackInterface{callback_name}Free");

    // Clone callback
    let clone_callback_fn = &format!("{}CloneCallback", DartCodeOracle::fn_name(callback_name));
    let clone_callback_pointer = &format!("{}ClonePointer", DartCodeOracle::fn_name(callback_name));
    let clone_callback_type = &format!("UniffiCallbackInterface{callback_name}Clone");

    quote! {
        $(functions)

        void $free_callback_fn(int handle) {
            try {
                FfiConverterCallbackInterface$cls_name._handleMap.remove(handle);
            } catch (e) {
                // Optionally log error, but do not return anything.
            }
        }

        final Pointer<NativeFunction<$free_callback_type>> $free_callback_pointer =
            Pointer.fromFunction<$free_callback_type>($free_callback_fn);

        int $clone_callback_fn(int handle) {
            try {
                final obj = FfiConverterCallbackInterface$cls_name._handleMap.get(handle);
                final newHandle = FfiConverterCallbackInterface$cls_name._handleMap.insert(obj);
                return newHandle;
            } catch (e) {
                // Return 0 on error, which should trigger an error on the Rust side
                return 0;
            }
        }

        final Pointer<NativeFunction<$clone_callback_type>> $clone_callback_pointer =
            Pointer.fromFunction<$clone_callback_type>($clone_callback_fn, 0);
    }
}

fn generate_foreign_future_struct_definition(
    ffi_struct: &FfiStruct,
    type_helper: &dyn TypeHelperRenderer,
) -> dart::Tokens {
    let struct_name = DartCodeOracle::ffi_struct_name(ffi_struct.name());
    let fields: Vec<dart::Tokens> = ffi_struct
        .fields()
        .iter()
        .map(|field| {
            let field_name = DartCodeOracle::var_name(field.name());
            let ffi_field_type = field.type_();
            let field_type = match &ffi_field_type {
                FfiType::RustCallStatus => quote!(RustCallStatus),
                _ => {
                    DartCodeOracle::ffi_dart_type_label(Some(&ffi_field_type), type_helper.get_ci())
                }
            };
            if let Some(annotation) = foreign_future_field_annotation(&ffi_field_type) {
                quote! {
                    $annotation
                    external $field_type $field_name;
                }
            } else {
                quote! {
                    external $field_type $field_name;
                }
            }
        })
        .collect();

    quote! {
        final class $struct_name extends Struct {
            $(for field in fields => $field)
        }
    }
}

fn generate_foreign_future_completion_typedef(
    callback_base: &str,
    struct_name: &str,
) -> dart::Tokens {
    let native_callback_name = format!("Uniffi{}", callback_base.to_upper_camel_case());
    let dart_callback_name = format!("{native_callback_name}Dart");
    let struct_tokens = DartCodeOracle::ffi_struct_name(struct_name);
    let struct_tokens_alt = struct_tokens.clone();

    let mut tokens = dart::Tokens::new();
    tokens.append(quote!(typedef $native_callback_name = Void Function(Uint64, $struct_tokens);));
    tokens.append(quote!(typedef $dart_callback_name = void Function(int, $struct_tokens_alt);));
    tokens
}

fn foreign_future_completion_name(method: &Method) -> String {
    let return_ffi_type = method.return_type().cloned().map(FfiType::from);
    let suffix = FfiType::return_type_name(return_ffi_type.as_ref()).to_upper_camel_case();
    format!("ForeignFutureComplete{suffix}")
}

fn foreign_future_field_annotation(field_type: &FfiType) -> Option<dart::Tokens> {
    match field_type {
        FfiType::Int8 => Some(quote!(@Int8())),
        FfiType::UInt8 => Some(quote!(@Uint8())),
        FfiType::Int16 => Some(quote!(@Int16())),
        FfiType::UInt16 => Some(quote!(@Uint16())),
        FfiType::Int32 => Some(quote!(@Int32())),
        FfiType::UInt32 => Some(quote!(@Uint32())),
        FfiType::Int64 => Some(quote!(@Int64())),
        FfiType::UInt64 => Some(quote!(@Uint64())),
        FfiType::Float32 => Some(quote!(@Float())),
        FfiType::Float64 => Some(quote!(@Double())),
        _ => None,
    }
}

pub fn generate_callback_interface_vtable_init_function(
    callback_name: &str,
    methods: &[&Method],
    ffi_module: &str,
) -> dart::Tokens {
    let vtable_name = &format!("UniffiVTableCallbackInterface{callback_name}");
    let vtable_static_instance_name =
        format!("{}{}", DartCodeOracle::fn_name(callback_name), "VTable");
    let init_fn_name = &format!("init{callback_name}VTable");
    let snake_callback = callback_name.to_lowercase();

    quote! {
        late final Pointer<$vtable_name> $(&vtable_static_instance_name);

        void $init_fn_name() {
            // Make initialization idempotent - return early if already initialized
            if (FfiConverterCallbackInterface$(DartCodeOracle::class_name(callback_name))._vtableInitialized) {
                return;
            }

            $(&vtable_static_instance_name) = calloc<$vtable_name>();
            $(&vtable_static_instance_name).ref.uniffiFree = $(format!("{}FreePointer", DartCodeOracle::fn_name(callback_name)));
            $(&vtable_static_instance_name).ref.uniffiClone = $(format!("{}ClonePointer", DartCodeOracle::fn_name(callback_name)));
            $(for m in methods {
                $(&vtable_static_instance_name).ref.$(DartCodeOracle::fn_name(m.name())) = $(DartCodeOracle::fn_name(callback_name))$(DartCodeOracle::class_name(m.name()))Pointer;
            })

            rustCall((status) {
                uniffi_$(ffi_module)_fn_init_callback_vtable_$(snake_callback)(
                    $(vtable_static_instance_name),
                );
                checkCallStatus(NullRustCallStatusErrorHandler(), status);
            });

            // Update the flag to prevent re-initialization
            FfiConverterCallbackInterface$(DartCodeOracle::class_name(callback_name))._vtableInitialized = true;
        }
    }
}
