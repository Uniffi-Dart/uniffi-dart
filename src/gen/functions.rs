use genco::prelude::*;
use uniffi_bindgen::backend::Type;
use uniffi_bindgen::interface::{Argument, AsType, Function, ObjectImpl};

use crate::gen::oracle::DartCodeOracle;
use crate::gen::render::AsRenderable;

use super::oracle::AsCodeType;
use super::render::TypeHelperRenderer;

pub fn generate_function(func: &Function, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
    // if func.takes_self() {} // TODO: Do something about this condition
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
                            $(for arg in &func.arguments() => $(lower_arg(arg)),) status
                        );
                    });
                }
            )
        } else {
            quote!(
                $ret $(DartCodeOracle::fn_name(func.name()))($args) {
                    return rustCall((status) => $lifter($(DartCodeOracle::find_lib_instance()).$(func.ffi_func().name())(
                        $(for arg in &func.arguments() => $(lower_arg(arg)),) status
                    )));
                }
            )
        }
    }
}
