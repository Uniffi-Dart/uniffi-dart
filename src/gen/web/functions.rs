use genco::prelude::*;
use heck::ToUpperCamelCase;
use uniffi_bindgen::interface::{AsType, Function, Type};

use crate::gen::oracle::DartCodeOracle;

use super::oracle::WebCodeOracle;

pub fn is_supported_function(func: &Function) -> bool {
    !func.is_async()
        && func.throws_type().is_none()
        && func
            .arguments()
            .iter()
            .all(|arg| is_supported_type(&arg.as_type()))
        && func.return_type().map(is_supported_type).unwrap_or(true)
}

pub fn is_supported_type(type_: &Type) -> bool {
    WebCodeOracle::dart_type_label(type_).is_some() && WebCodeOracle::js_type_label(type_).is_some()
}

pub fn uses_bytes(func: &Function) -> bool {
    func.arguments()
        .iter()
        .any(|arg| matches!(arg.as_type(), Type::Bytes))
        || func
            .return_type()
            .map(|ret| matches!(ret, Type::Bytes))
            .unwrap_or(false)
}

pub fn generate_function(func: &Function, module_name: &str, namespace: &str) -> dart::Tokens {
    let public_name = DartCodeOracle::fn_name(func.name());
    let external_name = format!("_uniffiWeb{}", func.name().to_upper_camel_case());
    let external_call_name = external_name.clone();
    let js_export_name = format!(
        "{}_{}",
        namespace.replace('-', "_"),
        func.name().replace('-', "_")
    );

    let public_args: Vec<dart::Tokens> = func
        .arguments()
        .iter()
        .map(|arg| {
            let arg_name = DartCodeOracle::var_name(arg.name());
            let arg_type = WebCodeOracle::dart_type_label(&arg.as_type())
                .expect("supported function argument should have Dart type");
            quote!(required $arg_type $arg_name)
        })
        .collect();

    let external_args: Vec<dart::Tokens> = func
        .arguments()
        .iter()
        .map(|arg| {
            let arg_name = DartCodeOracle::var_name(arg.name());
            let arg_type = WebCodeOracle::js_type_label(&arg.as_type())
                .expect("supported function argument should have JS type");
            quote!($arg_type $arg_name)
        })
        .collect();

    let call_args: Vec<dart::Tokens> = func
        .arguments()
        .iter()
        .map(|arg| {
            let arg_name = DartCodeOracle::var_name(arg.name());
            WebCodeOracle::lower_expr(&arg.as_type(), quote!($arg_name))
                .expect("supported function argument should lower")
        })
        .collect();

    let public_params = if public_args.is_empty() {
        quote!()
    } else {
        quote!({$(for arg in &public_args => $arg, )})
    };
    let external_params =
        quote!($(for (i, arg) in external_args.iter().enumerate() => $(if i > 0 => , )$arg));
    let call_params =
        quote!($(for (i, arg) in call_args.iter().enumerate() => $(if i > 0 => , )$arg));

    let (external_return, public_return, body) = if let Some(ret) = func.return_type() {
        let external_return =
            WebCodeOracle::js_type_label(ret).expect("supported return should have JS type");
        let public_return =
            WebCodeOracle::dart_type_label(ret).expect("supported return should have Dart type");
        let lifted =
            WebCodeOracle::lift_expr(ret, quote!(rawResult)).expect("supported return should lift");
        (
            external_return,
            public_return,
            quote! {
                try {
                    final rawResult = $(&external_call_name)($call_params);
                    return $lifted;
                } catch (error) {
                    return _throwWebError(error);
                }
            },
        )
    } else {
        (
            quote!(void),
            quote!(void),
            quote! {
                try {
                    $(&external_call_name)($call_params);
                } catch (error) {
                    _throwWebError(error);
                }
            },
        )
    };

    quote! {
        @JS($(quoted(format!("{module_name}.{js_export_name}"))))
        external $external_return $(&external_name)($external_params);

        $public_return $(&public_name)($public_params) {
            $body
        }
    }
}
