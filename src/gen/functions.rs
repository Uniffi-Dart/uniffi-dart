use genco::prelude::*;
use heck::ToLowerCamelCase;
use uniffi_bindgen::interface::{AsType, Function};

use super::defaults::render_argument_param;
use super::oracle::AsCodeType;
use super::render::TypeHelperRenderer;
use crate::gen::oracle::DartCodeOracle;
use crate::gen::render::AsRenderable;

pub fn generate_function(func: &Function, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
    let args = if func.arguments().is_empty() {
        quote!()
    } else {
        quote!({$(for arg in &func.arguments() =>
            $(render_argument_param(
                arg,
                arg.as_renderable().render_type(&arg.as_type(), type_helper)
            )),
        )})
    };

    let (ret, lifter) = if let Some(ret) = func.return_type() {
        (ret.as_renderable().render_type(ret, type_helper), quote!($(ret.as_codetype().lift())))
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

    // Whether any argument takes the borrowed `&[u8]` path, whose native copy
    // must be freed after the call (see `wrap_ffi_call_stmt`).
    let has_borrowed = DartCodeOracle::any_borrowed_bytes(&func.arguments());

    // Use centralized callback-aware argument lowering
    if func.is_async() {
        // For async methods returning objects, we need to convert the int pointer to Pointer<Void>
        let async_lifter = if let Some(ret_type) = func.return_type() {
            match ret_type {
                uniffi_bindgen::interface::Type::Object { .. } => {
                    quote!((ptr) => $lifter(Pointer<Void>.fromAddress(ptr)))
                }
                _ => lifter.clone(),
            }
        } else {
            lifter.clone()
        };

        quote!(
            Future<$ret> $(DartCodeOracle::fn_name(func.name()))($args) {
                $(DartCodeOracle::wrap_ffi_call_stmt(has_borrowed, true, quote!(
                    uniffiRustCallAsync(
                      () => $(func.ffi_func().name())(
                        $(for arg in &func.arguments() => $(DartCodeOracle::lower_arg_with_callback_handling(arg)),)
                      ),
                      $(DartCodeOracle::async_poll(func, type_helper.get_ci())),
                      $(DartCodeOracle::async_complete(func, type_helper.get_ci())),
                      $(DartCodeOracle::async_free(func, type_helper.get_ci())),
                      $async_lifter,
                      $error_handler,
                    )
                )))
            }
        )
    } else if ret == quote!(void) {
        quote!(
            $ret $(DartCodeOracle::fn_name(func.name()))($args) {
                $(DartCodeOracle::wrap_ffi_call_stmt(has_borrowed, false, quote!(
                    rustCall((status) {
                        $(func.ffi_func().name())(
                            $(for arg in &func.arguments() => $(DartCodeOracle::lower_arg_with_callback_handling(arg)),) status
                        );
                    }, $error_handler)
                )))
            }
        )
    } else {
        quote!(
            $ret $(DartCodeOracle::fn_name(func.name()))($args) {
                $(DartCodeOracle::wrap_ffi_call_stmt(has_borrowed, false, quote!(
                    rustCallWithLifter(
                        (status) => $(func.ffi_func().name())(
                            $(for arg in &func.arguments() => $(DartCodeOracle::lower_arg_with_callback_handling(arg)),) status
                        ),
                        $lifter,
                        $error_handler
                    )
                )))
            }
        )
    }
}
