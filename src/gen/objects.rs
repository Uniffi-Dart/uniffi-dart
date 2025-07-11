use genco::prelude::*;
use crate::gen::callback_interface::{generate_callback_functions, generate_callback_interface, generate_callback_interface_vtable_init_function, generate_callback_vtable_interface};
use crate::gen::CodeType;
use uniffi_bindgen::backend::{Literal, Type};
use uniffi_bindgen::interface::{Argument, AsType, Method, Object, ObjectImpl};

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
                )
            ) {
                 _$finalizer_cls_name.attach(this, _ptr, detach: this);
            }
        }
    });

    quote! {
        final _$finalizer_cls_name = Finalizer<Pointer<Void>>((ptr) {
          rustCall((status) => $lib_instance.$ffi_object_free_name(ptr, status));
        });

        class $cls_name {
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

            Pointer<Void> uniffiClonePointer() {
                return rustCall((status) => $lib_instance.$ffi_object_clone_name(_ptr, status));
            }

            void dispose() {
                _$finalizer_cls_name.detach(this);
                rustCall((status) => $lib_instance.$ffi_object_free_name(_ptr, status));
            }

            $(for mt in &obj.methods() => $(generate_method(mt, type_helper)))
        }

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

    fn lower_arg(arg: &Argument) -> dart::Tokens {
        let lower_arg = DartCodeOracle::type_lower_fn(&arg.as_type(), quote!($(DartCodeOracle::var_name(arg.name())))); 
        match arg.as_type() {
            Type::Object { imp, .. } if imp == ObjectImpl::CallbackTrait => quote!(Pointer<Void>.fromAddress($(lower_arg))),
            _ => lower_arg
        }
    }

    if func.is_async() {
        quote!(
            Future<$ret> $(DartCodeOracle::fn_name(func.name()))($args) {
                return uniffiRustCallAsync(
                  () => $(DartCodeOracle::find_lib_instance()).$(func.ffi_func().name())(
                    uniffiClonePointer(),
                    $(for arg in &func.arguments() => $(lower_arg(arg)),)
                  ),
                  $(DartCodeOracle::async_poll(func, type_helper.get_ci())),
                  $(DartCodeOracle::async_complete(func, type_helper.get_ci())),
                  $(DartCodeOracle::async_free(func, type_helper.get_ci())),
                  $lifter,
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
                            $(for arg in &func.arguments() => $(lower_arg(arg)),) status
                        );
                    });
                }
            )
        } else {
            quote!(
                $ret $(DartCodeOracle::fn_name(func.name()))($args) {
                    return rustCall((status) => $lifter($(DartCodeOracle::find_lib_instance()).$(func.ffi_func().name())(
                        uniffiClonePointer(),
                        $(for arg in &func.arguments() => $(lower_arg(arg)),) status
                    )));
                }
            )
      }
    }
}
