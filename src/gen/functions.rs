use genco::prelude::*;
use uniffi_bindgen::interface::{AsType, Function};

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

    // Use centralized callback-aware argument lowering

    if func.is_async() {
        quote!(
            Future<$ret> $(DartCodeOracle::fn_name(func.name()))($args) {
                return uniffiRustCallAsync(
                  () => $(DartCodeOracle::find_lib_instance()).$(func.ffi_func().name())(
                    $(for arg in &func.arguments() => $(DartCodeOracle::lower_arg_with_callback_handling(arg)),)
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
                            $(for arg in &func.arguments() => $(DartCodeOracle::lower_arg_with_callback_handling(arg)),) status
                        );
                    });
                }
            )
        } else {
            quote!(
                $ret $(DartCodeOracle::fn_name(func.name()))($args) {
                    return rustCall((status) => $lifter($(DartCodeOracle::find_lib_instance()).$(func.ffi_func().name())(
                        $(for arg in &func.arguments() => $(DartCodeOracle::lower_arg_with_callback_handling(arg)),) status
                    )));
                }
            )
        }
    }
}
