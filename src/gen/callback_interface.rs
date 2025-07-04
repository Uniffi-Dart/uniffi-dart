use genco::prelude::*;
use crate::gen::CodeType;
use uniffi_bindgen::interface::Type;
use uniffi_bindgen::interface::{AsType, Method};

use crate::gen::oracle::{AsCodeType, DartCodeOracle};
use crate::gen::render::AsRenderable;
use crate::gen::render::{Renderable, TypeHelperRenderer};

// Removed problematic context structure - will implement simpler improvements

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
        let callback = type_helper.get_ci().get_callback_interface_definition(&self.name).unwrap();
        
        // Generate all necessary components for the callback interface
        let interface = generate_callback_interface(callback.name(), &callback.as_codetype().ffi_converter_name(), &callback.methods(), type_helper);
        let vtable_interface = generate_callback_vtable_interface(callback.name(), &callback.methods());
        let functions = generate_callback_functions(callback.name(), &callback.methods(), type_helper);
        let namespace = type_helper.get_ci().namespace_for_type(&callback.as_type())
            .unwrap_or_else(|_| type_helper.get_ci().namespace());
        let vtable_init = generate_callback_interface_vtable_init_function(callback.name(), &callback.methods(), namespace);
        
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
) -> dart::Tokens {
    let cls_name = &DartCodeOracle::class_name(callback_name);
    let ffi_conv_name = &DartCodeOracle::class_name(ffi_converter_name);
    let init_fn_name = &format!("init{}VTable", callback_name);

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

            static $cls_name lift(int handle) {
                return _handleMap.get(handle);
            }
            
            static int lower($cls_name value) {
                _ensureVTableInitialized();
                return _handleMap.insert(value);
            }

            static void _ensureVTableInitialized() {
                if (!_vtableInitialized) {
                    $init_fn_name();
                    _vtableInitialized = true;
                }
            }
        
            static LiftRetVal<$cls_name> read(Uint8List buf) {
                final handle = buf.buffer.asByteData(buf.offsetInBytes).getInt64(0);
                return LiftRetVal(lift(handle), 8);
            }
        
            static int write($cls_name value, Uint8List buf) {
                final handle = lower(value);
                buf.buffer.asByteData(buf.offsetInBytes).setInt64(0, handle);
                return 8;
            }
        
            static int allocationSize($cls_name value) {
                return 8; // Just a handle (int64).
            }
        }

        // We must define callback signatures
        $(generate_callback_methods_signatures(cls_name, methods, type_helper))
    };

    tokens
}

fn generate_callback_methods_definitions(method: &Method, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
    let method_name = DartCodeOracle::fn_name(&method.name());
    let dart_args = &method.arguments().iter().map(|arg| {
        let arg_type = arg.as_renderable().render_type(&arg.as_type(), type_helper);
        let arg_name = DartCodeOracle::var_name(arg.name());


        quote!($arg_type $arg_name)
    }).into_iter().collect::<Vec<_>>();

    let ret_type = if let Some(ret) = method.return_type() {
        ret.as_renderable().render_type(ret, type_helper)
    } else {
        quote!(void)
    };

    quote!(
        $ret_type $method_name($(for a in dart_args => $a,));
    )
}

fn generate_callback_methods_signatures(callback_name: &str, methods: &[&Method], type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
    let mut tokens = dart::Tokens::new();
    for (method_index, method) in methods.iter().enumerate() {
        //let method_name = DartCodeOracle::fn_name(method.name());

        let ffi_method_type = format!(
            "UniffiCallbackInterface{}Method{}",
            callback_name,
            method_index
        );

        let dart_method_type = format!(
            "UniffiCallbackInterface{}Method{}Dart",
            callback_name,
            method_index
        );
        
        let method_return_type = if let Some(ret) = method.return_type() {
            DartCodeOracle::native_type_label(Some(ret), type_helper.get_ci())
        } else {
            quote!(Void)
        };

        tokens.append(quote! {
            typedef $ffi_method_type = Void Function(
                Uint64, $(for arg in &method.arguments() => $(DartCodeOracle::native_type_label(Some(&arg.as_type()), type_helper.get_ci())),)
                Pointer<$(&method_return_type)>, Pointer<RustCallStatus>);
            typedef $dart_method_type = void Function(
                int, $(for arg in &method.arguments() => $(DartCodeOracle::native_dart_type_label(Some(&arg.as_type()), type_helper.get_ci())),)
                Pointer<$(&method_return_type)>, Pointer<RustCallStatus>);
        });
    }

    tokens.append(quote! {
        typedef UniffiCallbackInterface$(callback_name)Free = Void Function(Uint64);
        typedef UniffiCallbackInterface$(callback_name)FreeDart = void Function(int);
    });

    tokens
}

pub fn generate_callback_vtable_interface(callback_name: &str, methods: &[&Method]) -> dart::Tokens {
    let vtable_name = format!("UniffiVTableCallbackInterface{}", callback_name);
    let methods_vec: Vec<_> = methods.into_iter().enumerate().collect();

    quote! {
        class $vtable_name extends Struct {
            $(for (index, m) in &methods_vec =>
                external Pointer<NativeFunction<UniffiCallbackInterface$(callback_name)Method$(format!("{}",index))>> $(DartCodeOracle::fn_name(m.name()));
            )
            external Pointer<NativeFunction<UniffiCallbackInterface$(callback_name)Free>> uniffiFree;
        }
    }
}

pub fn generate_callback_functions(callback_name: &str, methods: &[&Method], type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {    
    let cls_name = &DartCodeOracle::class_name(callback_name);

    let functions: Vec<dart::Tokens> = methods.iter().enumerate().map(|(index, m)| {
        let method_name = &format!("{}", &DartCodeOracle::fn_name(m.name()));
        let ffi_method_type = &format!("UniffiCallbackInterface{}Method{}", callback_name, index);
        let _dart_method_type = &format!("UniffiCallbackInterface{}Method{}Dart", callback_name, index);

        // Get parameter types using the oracle
        let param_types: Vec<dart::Tokens> = m.arguments().iter().map(|arg| {
            let arg_name = DartCodeOracle::var_name(arg.name());
            DartCodeOracle::callback_param_type(&arg.as_type(), &arg_name, type_helper.get_ci())
        }).collect();

        // Get argument lifts using the oracle
        let arg_lifts: Vec<dart::Tokens> = m.arguments().iter().enumerate().map(|(arg_idx, arg)| {
            let arg_name = DartCodeOracle::var_name(arg.name());
            DartCodeOracle::callback_arg_lift_indexed(&arg.as_type(), &arg_name, arg_idx)
        }).collect();

        // Prepare arg names for the method call using indexes
        let arg_names: Vec<dart::Tokens> = m.arguments().iter().enumerate().map(|(arg_idx, arg)| {
            DartCodeOracle::callback_arg_name(&arg.as_type(), arg_idx)
        }).collect();

        // Handle return value using the oracle
        let call_dart_method = if let Some(ret) = m.return_type() {
            DartCodeOracle::callback_return_handling(ret, method_name, arg_names)
        } else {
            // Handle void return types
            DartCodeOracle::callback_void_handling(method_name, arg_names)
        };

        // Get the appropriate out return type
        let out_return_type = DartCodeOracle::callback_out_return_type(m.return_type());

        // Generate the function body
        let callback_method_name = &format!("{}{}", &DartCodeOracle::fn_name(callback_name), &DartCodeOracle::class_name(m.name()));
        
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
    }).collect();

    // Free callback
    let free_callback_fn = &format!("{}FreeCallback", DartCodeOracle::fn_name(callback_name));
    let free_callback_pointer = &format!("{}FreePointer", DartCodeOracle::fn_name(callback_name));
    let free_callback_type = &format!("UniffiCallbackInterface{}Free", callback_name);
    
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
    }
}

pub fn generate_callback_interface_vtable_init_function(callback_name: &str, methods: &[&Method], namespace: &str) -> dart::Tokens {
    let vtable_name = &format!("UniffiVTableCallbackInterface{}", callback_name);
    let vtable_static_instance_name = format!("{}{}", DartCodeOracle::fn_name(callback_name), "VTable");
    let init_fn_name = &format!("init{}VTable", callback_name);

    quote! {
        late final Pointer<$vtable_name> $(&vtable_static_instance_name);

        void $init_fn_name() {
            // Make initialization idempotent - return early if already initialized
            if (FfiConverterCallbackInterface$(DartCodeOracle::class_name(callback_name))._vtableInitialized) {
                return;
            }

            $(&vtable_static_instance_name) = calloc<$vtable_name>();
            $(for m in methods {
                $(&vtable_static_instance_name).ref.$(DartCodeOracle::fn_name(m.name())) = $(DartCodeOracle::fn_name(callback_name))$(DartCodeOracle::class_name(m.name()))Pointer;
            })
            $(&vtable_static_instance_name).ref.uniffiFree = $(format!("{}FreePointer", DartCodeOracle::fn_name(callback_name)));

            rustCall((status) {
                _UniffiLib.instance.uniffi_$(namespace)_fn_init_callback_vtable_$(callback_name.to_lowercase())(
                    $(vtable_static_instance_name),
                );
                checkCallStatus(NullRustCallStatusErrorHandler(), status);
            });

            // Update the flag to prevent re-initialization
            FfiConverterCallbackInterface$(DartCodeOracle::class_name(callback_name))._vtableInitialized = true;
        }
    }
}
