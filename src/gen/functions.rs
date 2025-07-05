use genco::prelude::*;
use uniffi_bindgen::interface::{AsType, Function};
use heck::ToLowerCamelCase;

use crate::gen::oracle::DartCodeOracle;
use crate::gen::render::AsRenderable;

use super::oracle::AsCodeType;
use super::render::TypeHelperRenderer;

pub fn generate_function(func: &Function, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
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
        // Function can throw errors - use the appropriate error handler
        let error_name = match error_type {
            uniffi_bindgen::interface::Type::Enum { name, .. } => name,
            uniffi_bindgen::interface::Type::Object { name, .. } => name,
            _ => {
                // For other error types, try to get the name from the type
                "UnknownError"
            }
        };
        // Use the same naming convention as in enums.rs
        let dart_cls_name = DartCodeOracle::class_name(error_name);
        let error_handler_instance = format!("{}ErrorHandler", dart_cls_name.to_lower_camel_case());
        quote!($(error_handler_instance))
    } else {
        // Function doesn't throw errors - use null handler
        quote!(null)
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
                            $(for arg in &func.arguments() => $(DartCodeOracle::lower_arg_with_callback_handling(arg)),) status
                        );
                    }, $error_handler);
                }
            )
        } else {
            quote!(
                $ret $(DartCodeOracle::fn_name(func.name()))($args) {
                    return rustCall((status) => $lifter($(DartCodeOracle::find_lib_instance()).$(func.ffi_func().name())(
                        $(for arg in &func.arguments() => $(DartCodeOracle::lower_arg_with_callback_handling(arg)),) status
                    )), $error_handler);
                }
            )
        }
    }
}
