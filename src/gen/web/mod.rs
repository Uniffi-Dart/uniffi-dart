use anyhow::{bail, Result};
use genco::prelude::*;
use uniffi_bindgen::ComponentInterface;

use crate::gen::Config;

mod functions;
mod oracle;
mod types;

pub struct WebDartWrapper<'a> {
    ci: &'a ComponentInterface,
    config: &'a Config,
}

impl<'a> WebDartWrapper<'a> {
    pub fn new(ci: &'a ComponentInterface, config: &'a Config) -> Self {
        Self { ci, config }
    }

    pub fn generate(&self) -> Result<dart::Tokens> {
        let ns = self.ci.namespace();
        let module_name = self.config.wasm_module_name(ns);
        let supported_functions: Vec<_> = self
            .ci
            .function_definitions()
            .iter()
            .filter(|func| functions::is_supported_function(func))
            .collect();

        emit_unsupported_diagnostics(self.ci, self.config)?;

        let mut hidden_names = vec!["ensureInitialized".to_string(), "initialize".to_string()];
        hidden_names.extend(
            supported_functions
                .iter()
                .map(|func| crate::gen::oracle::DartCodeOracle::fn_name(func.name())),
        );
        let stub_export = quote!(export $(quoted(format!("{ns}_stub.dart"))) hide $(for (i, name) in hidden_names.iter().enumerate() => $(if i > 0 => , )$name););

        let function_tokens = quote! {
            $(for func in &supported_functions =>
                $(functions::generate_function(func, &module_name, ns))
            )
        };
        let typed_data_import = if supported_functions
            .iter()
            .any(|func| functions::uses_bytes(func))
        {
            quote!(import "dart:typed_data";)
        } else {
            quote!()
        };
        let runtime = types::generate_web_runtime(&module_name);
        let version_check = self.generate_version_check(&module_name);
        let checksum_check = self.generate_checksum_check(&module_name);

        Ok(quote! {
            import "dart:async";
            import "dart:js_interop";
            $typed_data_import

            $stub_export

            $runtime

            $version_check

            $checksum_check

            $function_tokens
        })
    }

    fn generate_version_check(&self, module_name: &str) -> dart::Tokens {
        let external_name = "_uniffiWebContractVersion";
        let contract_version_fn = self.ci.ffi_uniffi_contract_version();
        let js_name = contract_version_fn.name();
        let bindings_version = self.ci.uniffi_contract_version();

        quote! {
            @JS($(quoted(format!("{module_name}.{js_name}"))))
            external JSNumber $external_name();

            void _checkApiVersion() {
                final bindingsVersion = $bindings_version;
                final scaffoldingVersion = $external_name().toDartInt;
                if (bindingsVersion != scaffoldingVersion) {
                    throw UniffiInternalError.panicked("UniFFI contract version mismatch: bindings version $bindingsVersion, scaffolding version $scaffoldingVersion");
                }
            }
        }
    }

    fn generate_checksum_check(&self, module_name: &str) -> dart::Tokens {
        let declarations = quote! {
            $(for (name, _) in self.ci.iter_checksums() =>
                @JS($(quoted(format!("{module_name}.{name}"))))
                external JSNumber $(format!("_uniffiWebChecksum_{name}"))();
            )
        };
        let checks = quote! {
            $(for (name, expected_checksum) in self.ci.iter_checksums() =>
                if ($(format!("_uniffiWebChecksum_{name}"))().toDartInt != $expected_checksum) {
                    throw UniffiInternalError.panicked("UniFFI API checksum mismatch");
                }
            )
        };

        quote! {
            $declarations

            void _checkApiChecksums() {
                $checks
            }
        }
    }
}

pub fn unsupported_web_api_names(ci: &ComponentInterface) -> Vec<String> {
    let mut unsupported: Vec<String> = ci
        .function_definitions()
        .iter()
        .filter(|func| !functions::is_supported_function(func))
        .map(|func| format!("function {}", func.name()))
        .collect();

    unsupported.extend(
        ci.object_definitions()
            .iter()
            .map(|obj| format!("object {}", obj.name())),
    );
    unsupported.extend(
        ci.callback_interface_definitions()
            .iter()
            .map(|callback| format!("callback interface {}", callback.name())),
    );
    unsupported
}

pub fn emit_unsupported_diagnostics(ci: &ComponentInterface, config: &Config) -> Result<()> {
    let unsupported = unsupported_web_api_names(ci);
    if unsupported.is_empty() {
        return Ok(());
    }

    let message = format!(
        "unsupported web APIs for namespace '{}': {}",
        ci.namespace(),
        unsupported.join(", ")
    );

    if config.web_unsupported_policy() == "error" {
        bail!("{message}");
    }

    println!("WARNING: {message}");
    Ok(())
}

pub fn validate_unique_wasm_module_names<I>(modules: I) -> Result<()>
where
    I: IntoIterator<Item = (String, String)>,
{
    let mut seen = std::collections::HashMap::new();
    for (namespace, module_name) in modules {
        if let Some(previous_namespace) = seen.insert(module_name.clone(), namespace.clone()) {
            bail!(
                "Duplicate wasm_module_name `{module_name}` for namespaces `{previous_namespace}` and `{namespace}`. Configure a distinct wasm_module_name for one component."
            );
        }
    }
    Ok(())
}
