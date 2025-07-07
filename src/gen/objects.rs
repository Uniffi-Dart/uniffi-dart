use genco::prelude::*;
use std::fmt::Debug;

use std::string::ToString;
use heck::ToLowerCamelCase;
use crate::gen::callback_interface::{generate_callback_functions, generate_callback_interface, generate_callback_interface_vtable_init_function, generate_callback_vtable_interface};
use crate::gen::CodeType;
use uniffi_bindgen::backend::Literal;
use uniffi_bindgen::interface::{AsType, Method, Object, ObjectImpl};

use crate::gen::oracle::{AsCodeType, DartCodeOracle};
use crate::gen::render::AsRenderable;
use crate::gen::render::{Renderable, TypeHelperRenderer};

use super::stream::generate_stream;

#[derive(Debug)]
pub struct ObjectCodeType {
    id: String,
    imp: ObjectImpl,
}

impl ObjectCodeType {
    pub fn new(id: String, imp: ObjectImpl) -> Self {
        Self { id, imp }
    }
}

impl CodeType for ObjectCodeType {
    fn type_label(&self) -> String {
        DartCodeOracle::class_name(&self.id)
    }

    fn canonical_name(&self) -> String {
        self.id.to_string()
    }

    fn literal(&self, _literal: &Literal) -> String {
        unreachable!();
    }

    fn ffi_converter_name(&self) -> String {
        match self.imp {
            ObjectImpl::Struct => self.canonical_name().to_string(), // Objects will use factory methods
            ObjectImpl::CallbackTrait => format!("FfiConverterCallbackInterface{}", self.id),
            ObjectImpl::Trait => todo!("trait objects not supported"),
        }
    }
}

impl Renderable for ObjectCodeType {
    fn render_type_helper(&self, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
        if type_helper.check(&self.id) {
            quote!()
        } else if let Some(obj) = type_helper.get_object(&self.id) {
            generate_object(obj, type_helper)
        } else {
            unreachable!()
        }
    }
}
pub fn generate_object(obj: &Object, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
    type_helper.include_once_check(obj.name(), &obj.as_type());

    if obj.has_callback_interface() {
        let interface = generate_callback_interface(obj.name(), &obj.as_codetype().ffi_converter_name(), &obj.methods(), type_helper);
        let vtable_interface = generate_callback_vtable_interface(obj.name(), &obj.methods());
        let functions = generate_callback_functions(obj.name(), &obj.methods(), type_helper);
        let vtable_init = generate_callback_interface_vtable_init_function(
            obj.name(),
            &obj.methods(),
            type_helper.get_ci().namespace_for_type(&obj.as_type()).expect("object should have namespace")
        );
        return quote!(
            $interface
            $vtable_interface
            $functions
            $vtable_init
        )
    }

    let cls_name = &DartCodeOracle::class_name(obj.name());
    let finalizer_cls_name = &format!("{}Finalizer", cls_name);
    let lib_instance = &DartCodeOracle::find_lib_instance();
    let ffi_object_free_name = obj.ffi_object_free().name();
    let ffi_object_clone_name = obj.ffi_object_clone().name();

    // Stream workaround, make it more elegant later

    let stream_glue = if obj.name().contains("StreamExt") {
        generate_stream(obj, type_helper)
    } else {
        quote!()
    };
    
    let constructor_definitions = obj.constructors().into_iter().map(|constructor| {
        let ffi_func_name = constructor.ffi_func().name();
        let constructor_name = constructor.name();

        let dart_constructor_decl = if constructor_name == "new" {
            quote!($cls_name)
        } else {
            quote!($cls_name.$(DartCodeOracle::fn_name(constructor_name)))
        };
        
        // Check if function can throw errors
        let error_handler = if let Some(error_type) = constructor.throws_type() {
            let error_name = DartCodeOracle::class_name(error_type.name().unwrap_or("UnknownError"));
            // Use the consistent Exception naming for error handlers
            let handler_name = format!("{}ErrorHandler", error_name.to_lower_camel_case());
            quote!($(handler_name))
        } else {
            quote!(null)
        };

        let dart_params = quote!($(for arg in constructor.arguments() =>
            $(DartCodeOracle::dart_type_label(Some(&arg.as_type()))) $(DartCodeOracle::var_name(arg.name())),
        ));

        let ffi_call_args = quote!($(for arg in constructor.arguments() =>
            $(DartCodeOracle::type_lower_fn(&arg.as_type(), quote!($(DartCodeOracle::var_name(arg.name()))))),) 
        );
        
        // Ensure argument types are included
        for arg in constructor.arguments() {
            type_helper.include_once_check(&arg.as_codetype().canonical_name(), &arg.as_type());
        }

        quote! {
            // Public constructor
            $dart_constructor_decl($dart_params) : _ptr = rustCall((status) =>
                $lib_instance.$ffi_func_name(
                    $ffi_call_args status
                ),
                $error_handler
            ) {
                 _$finalizer_cls_name.attach(this, _ptr, detach: this);
            }
        }
    });

    // For interface objects that are used as error types, generate error handlers
    let is_error_interface = type_helper.get_ci().is_name_used_as_error(obj.name());
    
    let error_handler_class = if is_error_interface {
        // Generate error handlers for specific error interfaces
        let error_handler_name = format!("{}ErrorHandler", cls_name);
        let instance_name = cls_name.to_lower_camel_case();
        quote! {
            class $(&error_handler_name) extends UniffiRustCallStatusErrorHandler {
                @override
                Exception lift(RustBuffer errorBuf) {
                    return $(cls_name).read(errorBuf.asUint8List()).value;
                }
            }

            final $(&error_handler_name) $(instance_name)ErrorHandler = $(&error_handler_name)();
        }
    } else {
        quote!()
    };

    let implements_exception = if is_error_interface {
        quote!( implements Exception)
    } else {
        quote!()
    };

    
    // Generate toString() method for error interfaces
    let to_string_method: dart::Tokens = if is_error_interface && !obj.is_trait_interface() {
        // Only generate toString for regular error interfaces, skip trait interfaces for now
        let dart_class_name = format!("\"{}\"", cls_name);
        quote! {
            @override
            String toString() {
                return $(&dart_class_name);
            }
        }
    } else {
        quote!()
    };

    quote! {
        final _$finalizer_cls_name = Finalizer<Pointer<Void>>((ptr) {
          rustCall((status) => $lib_instance.$ffi_object_free_name(ptr, status));
        });

        class $cls_name $implements_exception {
            late final Pointer<Void> _ptr;

            // Private constructor for internal use / lift
            $cls_name._(this._ptr) {
                _$finalizer_cls_name.attach(this, _ptr, detach: this);
            }

            // Public constructors generated from UDL
            $( for ctor_def in constructor_definitions => $ctor_def )

            // Factory for lifting pointers
            factory $cls_name.lift(Pointer<Void> ptr) {
                return $cls_name._(ptr);
            }

            static Pointer<Void> lower($cls_name value) {
                return value.uniffiClonePointer();
            }

            Pointer<Void> uniffiClonePointer() {
                return rustCall((status) => $lib_instance.$ffi_object_clone_name(_ptr, status));
            }

            // A Rust pointer is 8 bytes
            static int allocationSize($cls_name value) {
                return 8;
            }

            static LiftRetVal<$cls_name> read(Uint8List buf) {
                final handle = buf.buffer.asByteData(buf.offsetInBytes).getInt64(0);
                final pointer = Pointer<Void>.fromAddress(handle);
                return LiftRetVal($cls_name.lift(pointer), 8);
            }

            static int write($cls_name value, Uint8List buf) {
                final handle = lower(value);
                buf.buffer.asByteData(buf.offsetInBytes).setInt64(0, handle.address);
                return 8;
            }

            void dispose() {
                _$finalizer_cls_name.detach(this);
                rustCall((status) => $lib_instance.$ffi_object_free_name(_ptr, status));
            }

            $to_string_method

            $(for mt in &obj.methods() => $(generate_method(mt, type_helper)))
        }

        $error_handler_class

        $(stream_glue)
    }
}

#[allow(unused_variables)]
pub fn generate_method(func: &Method, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
    // if func.takes_self_by_arc() {} // TODO: Do something about this condition
    let args = quote!($(for arg in &func.arguments() => $(&arg.as_renderable().render_type(&arg.as_type(), type_helper)) $(DartCodeOracle::var_name(arg.name())),));

    let (ret, lifter) = if let Some(ret) = func.return_type() {
        (
            ret.as_renderable().render_type(ret, type_helper),
            quote!($(ret.as_codetype().lift())),
        )
    } else {
        (quote!(void), quote!((_) {}))
    };

    // Check if function can throw errors
    let error_handler = if let Some(error_type) = func.throws_type() {
        let error_name = DartCodeOracle::class_name(error_type.name().unwrap_or("UnknownError"));
        // Use the consistent Exception naming for error handlers
        let handler_name = format!("{}ErrorHandler", error_name.to_lower_camel_case());
        quote!($(handler_name))
    } else {
        quote!(null)
    };

    if func.is_async() {
        quote!(
            Future<$ret> $(DartCodeOracle::fn_name(func.name()))($args) {
                return uniffiRustCallAsync(
                  () => $(DartCodeOracle::find_lib_instance()).$(func.ffi_func().name())(
                    uniffiClonePointer(),
                    $(for arg in &func.arguments() => $(DartCodeOracle::lower_arg_with_callback_handling(arg)),)
                  ),
                  $(DartCodeOracle::async_poll(func, type_helper.get_ci())),
                  $(DartCodeOracle::async_complete(func, type_helper.get_ci())),
                  $(DartCodeOracle::async_free(func, type_helper.get_ci())),
                  $lifter,
                  $error_handler,
                );
            }

        )
    } else {
        if ret == quote!(void) {
            quote!(
                $ret $(DartCodeOracle::fn_name(func.name()))($args) {
                    return rustCall((status) {
                        $(DartCodeOracle::find_lib_instance()).$(func.ffi_func().name())(
                            uniffiClonePointer(),
                            $(for arg in &func.arguments() => $(DartCodeOracle::lower_arg_with_callback_handling(arg)),) status
                        );
                    }, $error_handler);
                }
            )
        } else {
            quote!(
                $ret $(DartCodeOracle::fn_name(func.name()))($args) {
                    return rustCall((status) => $lifter($(DartCodeOracle::find_lib_instance()).$(func.ffi_func().name())(
                        uniffiClonePointer(),
                        $(for arg in &func.arguments() => $(DartCodeOracle::lower_arg_with_callback_handling(arg)),) status
                    )), $error_handler);
                }
            )
      }
    }
}
