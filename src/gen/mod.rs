use std::collections::{HashMap, HashSet};
use std::process::Command;

use anyhow::{Context, Result};
use camino::Utf8Path;
use genco::fmt;
use genco::prelude::*;
use serde::{Deserialize, Serialize};
use toml;
use uniffi_bindgen::{BindingGenerator, Component, ComponentInterface};

// use uniffi_bindgen::MergeWith;
use self::render::Renderer;
use self::types::TypeHelpersRenderer;
use crate::gen::oracle::DartCodeOracle;

mod callback_interface;
mod code_type;
mod compounds;
mod custom;
mod defaults;
mod enums;
mod functions;
mod objects;
mod oracle;
mod primitives;
mod records;
mod render;
pub mod stream;
mod types;

pub use code_type::CodeType;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    package_name: Option<String>,
    cdylib_name: Option<String>,
    #[serde(default)]
    external_packages: HashMap<String, String>,
    asset_id: Option<String>,
}

impl From<&ComponentInterface> for Config {
    fn from(ci: &ComponentInterface) -> Self {
        Config {
            package_name: Some(ci.namespace().to_owned()),
            cdylib_name: Some(ci.namespace().to_owned()),
            external_packages: HashMap::new(),
            asset_id: None,
        }
    }
}

impl Config {
    pub fn package_name(&self) -> String {
        if let Some(package_name) = &self.package_name {
            package_name.clone()
        } else {
            "uniffi".into()
        }
    }

    pub fn cdylib_name(&self) -> String {
        if let Some(cdylib_name) = &self.cdylib_name {
            cdylib_name.clone()
        } else {
            "uniffi".into()
        }
    }

    pub fn asset_id(&self) -> String {
        if let Some(asset_id) = &self.asset_id {
            asset_id.clone()
        } else {
            // Default: uniffi:{cdylib_name}
            // Dart's Native Assets system automatically prefixes this with package:{dart_package_name}/
            // so the full ID becomes package:{dart_package_name}/uniffi:{cdylib_name}
            format!("uniffi:{}", self.cdylib_name())
        }
    }
}

/// The `RustBuffer` alloc/free/reserve helpers, which `RustBuffer`'s own methods
/// call. They live alongside `RustBuffer` in the shared runtime rather than being
/// repeated in every component file.
fn is_rustbuffer_fn(ci: &ComponentInterface, name: &str) -> bool {
    ci.iter_rust_buffer_ffi_function_definitions().any(|f| f.name() == name)
}

/// Emit the `@Native` extern declarations for `ci`'s FFI functions.
///
/// `include` partitions them: the shared runtime file takes the `rustbuffer_*`
/// helpers it calls from `RustBuffer`, and each component file takes the rest.
/// Declaring a symbol in both places would be a duplicate definition.
fn uniffi_function_definitions(
    ci: &ComponentInterface,
    asset_id: &str,
    include: impl Fn(&str) -> bool,
) -> dart::Tokens {
    let mut definitions = quote!();
    let mut defined_functions = HashSet::new(); // Track defined function names

    for fun in ci.iter_ffi_function_definitions() {
        let fun_name = fun.name().to_owned();

        if !include(&fun_name) {
            continue;
        }

        // Check for duplicate function names
        if !defined_functions.insert(fun_name.clone()) {
            // Function name already exists, skip to prevent duplicate definition
            continue;
        }

        // For @Native, we need both native types (for the annotation) and Dart types (for the external declaration)
        let native_return_type = match fun.return_type() {
            Some(return_type) => {
                quote! { $(DartCodeOracle::ffi_native_type_label(Some(return_type))) }
            }
            None => quote! { Void },
        };

        let dart_return_type = match fun.return_type() {
            Some(return_type) => {
                quote! { $(DartCodeOracle::ffi_dart_type_label(Some(return_type))) }
            }
            None => quote! { void },
        };

        let (native_args, dart_args) = {
            let mut native_arg_vec = vec![];
            let mut dart_arg_with_names_vec = vec![];

            for arg in fun.arguments() {
                let arg_name = arg.name();
                let native_type = DartCodeOracle::ffi_native_type_label(Some(&arg.type_()));
                let dart_type = DartCodeOracle::ffi_dart_type_label(Some(&arg.type_()));

                native_arg_vec.push(native_type);
                dart_arg_with_names_vec.push(quote!($dart_type $arg_name));
            }

            if fun.has_rust_call_status_arg() {
                native_arg_vec.push(quote!(Pointer<RustCallStatus>));
                dart_arg_with_names_vec.push(quote!(Pointer<RustCallStatus> uniffiStatus));
            }

            let native_args = quote!($(for (i, arg) in native_arg_vec.iter().enumerate() => $(if i > 0 => , )$[' ']$arg));
            let dart_args = quote!($(for (i, arg) in dart_arg_with_names_vec.iter().enumerate() => $(if i > 0 => , )$[' ']$arg));
            (native_args, dart_args)
        };

        // Generate @Native annotation with assetId
        // @Native uses the function name as symbol automatically
        // assetId references the _uniffiAssetId constant
        definitions.append(quote! {
            @Native<$(&native_return_type) Function($(&native_args))>(
              assetId: $asset_id
            )
            external $(&dart_return_type) $fun_name($(&dart_args));
            $['\n']
        });
    }

    definitions
}

pub struct DartWrapper<'a> {
    config: &'a Config,
    ci: &'a ComponentInterface,
    type_renderer: TypeHelpersRenderer<'a>,
}

impl<'a> DartWrapper<'a> {
    pub fn new(ci: &'a ComponentInterface, config: &'a Config) -> Self {
        let type_renderer = TypeHelpersRenderer::new(ci);
        DartWrapper { ci, config, type_renderer }
    }

    fn generate(&self) -> dart::Tokens {
        let package_name = &self.config.package_name();

        let (type_helper_code, functions_definitions) = &self.type_renderer.render();

        let asset_id_suffix = &self.config.asset_id(); // e.g., "uniffi:hello_world"

        quote! {
            // The runtime scaffolding moved to the shared module, so a component
            // file may not use every import it declares. Which ones go unused
            // depends on the interface, so the directive is simpler than tracking
            // usage per import.
            $("// ignore_for_file: unused_import")$['\n']
            library $package_name;

            $(type_helper_code) // Imports, Types and Type Helper

            // Generated by uniffi-dart – do NOT edit.
            // This asset ID is used by @Native annotations to locate the native library
            // via Native Assets. Dart automatically prefixes asset names with "package:{packageName}/",
            // so we construct the full ID here to match what the build hook registers.
            // The asset ID format is: package:{dart_package_name}/uniffi:{cdylib_name}
            const _uniffiAssetId = $(quoted(format!("package:{}/{}", package_name, asset_id_suffix)));

            $(functions_definitions)

            // FFI function definitions using @Native
            $(uniffi_function_definitions(self.ci, "_uniffiAssetId", |name| !is_rustbuffer_fn(self.ci, name)))

            // API version and checksum validation
            void _checkApiVersion() {
                final bindingsVersion = $(self.ci.uniffi_contract_version());
                final scaffoldingVersion = $(self.ci.ffi_uniffi_contract_version().name())();
                if (bindingsVersion != scaffoldingVersion) {
                  throw UniffiInternalError.panicked("UniFFI contract version mismatch: bindings version $bindingsVersion, scaffolding version $scaffoldingVersion");
                }
            }

            void _checkApiChecksums() {
                $(for (name, expected_checksum) in self.ci.iter_checksums() =>
                    if ($(name)() != $expected_checksum) {
                      throw UniffiInternalError.panicked("UniFFI API checksum mismatch");
                    }
                )
            }

            void ensureInitialized() {
                _checkApiVersion();
                _checkApiChecksums();
            }

            // Backwards-compatible entry point used by existing tests
            @Deprecated("Use ensureInitialized instead")
            void initialize() {
                ensureInitialized();
            }
        }
    }
}

pub struct DartBindingGenerator;

impl BindingGenerator for DartBindingGenerator {
    type Config = Config;

    fn write_bindings(
        &self,
        settings: &uniffi_bindgen::GenerationSettings,
        components: &[uniffi_bindgen::Component<Self::Config>],
    ) -> Result<()> {
        let write_dart = |filename: camino::Utf8PathBuf, tokens: dart::Tokens| -> Result<()> {
            let file = std::fs::File::create(filename)?;
            let mut w = fmt::IoWriter::new(file);

            let mut fmt = fmt::Config::from_lang::<Dart>();
            if settings.try_format_code {
                fmt = fmt.with_indentation(fmt::Indentation::Space(2));
            }
            let config = dart::Config::default();

            tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
            Ok(())
        };

        // A component whose namespace matches the runtime module would write its
        // own file over the shared runtime.
        if let Some(Component { ci, .. }) =
            components.iter().find(|c| c.ci.namespace() == types::RUNTIME_MODULE)
        {
            bail!(
                "namespace `{}` collides with the shared runtime module `{}.dart`; \
                 rename the namespace",
                ci.namespace(),
                types::RUNTIME_MODULE
            );
        }

        // The shared runtime goes out once for the whole generation, ahead of the
        // per-component files that import it. The first component supplies the
        // `rustbuffer_*` symbols and asset id for all of them: only library mode
        // generates more than one component, every component there links into one
        // cdylib, and UniFFI gives each the same cdylib name. If a configuration
        // ever breaks that assumption, the runtime would resolve its allocator
        // against the wrong library, so check rather than trust it.
        let full_asset_id =
            |config: &Config| format!("package:{}/{}", config.package_name(), config.asset_id());
        if let Some(Component { config: first, .. }) = components.first() {
            for Component { ci, config, .. } in components {
                if full_asset_id(config) != full_asset_id(first) {
                    bail!(
                        "component `{}` has asset id `{}`, which differs from `{}`; \
                         the shared runtime module can only carry one asset id",
                        ci.namespace(),
                        full_asset_id(config),
                        full_asset_id(first)
                    );
                }
            }
        }

        if let Some(Component { ci, config, .. }) = components.first() {
            let asset_id = full_asset_id(config);
            let runtime = quote! {
                $("// ignore_for_file: unused_import")$['\n']
                $(format!("library {};", types::RUNTIME_MODULE))

                $(types::runtime_scaffolding(ci))

                const _uniffiAssetId = $(quoted(asset_id));

                $(uniffi_function_definitions(ci, "_uniffiAssetId", |name| is_rustbuffer_fn(ci, name)))
            };
            write_dart(settings.out_dir.join(format!("{}.dart", types::RUNTIME_MODULE)), runtime)?;
        }

        for Component { ci, config, .. } in components {
            write_dart(
                settings.out_dir.join(format!("{}.dart", ci.namespace())),
                DartWrapper::new(ci, config).generate(),
            )?;
        }

        // Run full Dart formatter on the output directory as a best-effort step.
        // This is non-fatal: failures will only emit a warning.
        let mut format_command = Command::new("dart");
        format_command.current_dir(&settings.out_dir).arg("format").arg(".");
        match format_command.spawn().and_then(|mut c| c.wait()) {
            Ok(status) if status.success() => {}
            Ok(_) | Err(_) => {
                eprintln!(
                    "WARNING: dart format failed or is unavailable; proceeding without full formatting"
                );
            }
        }
        Ok(())
    }

    fn new_config(&self, root_toml: &toml::value::Value) -> Result<Self::Config> {
        Ok(match root_toml.get("bindings").and_then(|b| b.get("dart")) {
            Some(v) => v.clone().try_into()?,
            None => Default::default(),
        })
    }

    fn update_component_configs(
        &self,
        settings: &uniffi_bindgen::GenerationSettings,
        components: &mut Vec<uniffi_bindgen::Component<Self::Config>>,
    ) -> Result<()> {
        for c in &mut *components {
            c.config.cdylib_name.get_or_insert_with(|| {
                settings.cdylib.clone().unwrap_or_else(|| format!("uniffi_{}", c.ci.namespace()))
            });
        }
        Ok(())
    }
}

pub fn generate_dart_bindings(
    udl_file: &Utf8Path,
    config_file_override: Option<&Utf8Path>,
    out_dir_override: Option<&Utf8Path>,
    library_file: &Utf8Path,
    library_mode: bool,
) -> anyhow::Result<()> {
    if library_mode {
        // Resolve each crate's `uniffi.toml` and UDL from cargo metadata, exactly
        // as uniffi-bindgen itself does. This is what makes `package_name` /
        // `cdylib_name` (and any per-crate config) resolve correctly — a custom
        // supplier that reads only a single file would default those and desync
        // the generated asset ids from the Native Assets build hook.
        let config_supplier =
            uniffi_bindgen::cargo_metadata::CrateConfigSupplier::from_cargo_metadata_command(
                false,
            )
            .context("Failed to build the crate config supplier from cargo metadata")?;

        uniffi_bindgen::library_mode::generate_bindings(
            library_file,
            None, // crate name filter
            &DartBindingGenerator {},
            &config_supplier,
            // Pass the `--config` override through so uniffi merges it into each
            // crate's config (values take precedence), rather than replacing every
            // crate's config with the same file.
            config_file_override,
            out_dir_override.unwrap(),
            true,
        )?;
        Ok(())
    } else {
        // Note: library_file is needed by uniffi_bindgen to extract metadata from proc macros,
        // even though we don't use it for DynamicLibrary.open() anymore (Native Assets handle that)
        uniffi_bindgen::generate_external_bindings(
            &DartBindingGenerator {},
            udl_file,
            config_file_override,
            out_dir_override,
            Some(library_file),
            None,
            true,
        )
    }
}
