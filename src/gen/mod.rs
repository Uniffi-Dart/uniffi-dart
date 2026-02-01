use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Read;
use std::process::Command;

use anyhow::{bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata::Metadata;

use genco::fmt;
use genco::prelude::*;
use serde::{Deserialize, Serialize};
use toml;
use uniffi_bindgen::BindgenCrateConfigSupplier;
use uniffi_bindgen::Component;
// use uniffi_bindgen::MergeWith;
use self::render::Renderer;
use self::types::TypeHelpersRenderer;
use crate::gen::oracle::DartCodeOracle;
use uniffi_bindgen::{BindingGenerator, ComponentInterface};

mod callback_interface;
mod code_type;
mod compounds;
mod custom;
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

pub struct DartWrapper<'a> {
    config: &'a Config,
    ci: &'a ComponentInterface,
    type_renderer: TypeHelpersRenderer<'a>,
}

impl<'a> DartWrapper<'a> {
    pub fn new(ci: &'a ComponentInterface, config: &'a Config) -> Self {
        let type_renderer = TypeHelpersRenderer::new(ci);
        DartWrapper {
            ci,
            config,
            type_renderer,
        }
    }

    fn generate(&self) -> dart::Tokens {
        let package_name = &self.config.package_name();

        let (type_helper_code, functions_definitions) = &self.type_renderer.render();

        // Generate @Native external function definitions
        fn uniffi_function_definitions(ci: &ComponentInterface, asset_id: &str) -> dart::Tokens {
            let mut definitions = quote!();
            let mut defined_functions = HashSet::new(); // Track defined function names

            for fun in ci.iter_ffi_function_definitions() {
                let fun_name = fun.name().to_owned();

                // Check for duplicate function names
                if !defined_functions.insert(fun_name.clone()) {
                    // Function name already exists, skip to prevent duplicate definition
                    continue;
                }

                // For @Native, we need both native types (for the annotation) and Dart types (for the external declaration)
                let native_return_type = match fun.return_type() {
                    Some(return_type) => {
                        quote! { $(DartCodeOracle::ffi_native_type_label(Some(return_type), ci)) }
                    }
                    None => quote! { Void },
                };

                let dart_return_type = match fun.return_type() {
                    Some(return_type) => {
                        quote! { $(DartCodeOracle::ffi_dart_type_label(Some(return_type), ci)) }
                    }
                    None => quote! { void },
                };

                let (native_args, dart_args) = {
                    let mut native_arg_vec = vec![];
                    let mut dart_arg_with_names_vec = vec![];

                    for arg in fun.arguments() {
                        let arg_name = arg.name();
                        let native_type =
                            DartCodeOracle::ffi_native_type_label(Some(&arg.type_()), ci);
                        let dart_type = DartCodeOracle::ffi_dart_type_label(Some(&arg.type_()), ci);

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

        let asset_id_suffix = &self.config.asset_id(); // e.g., "uniffi:hello_world"

        quote! {
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
            $(uniffi_function_definitions(self.ci, "_uniffiAssetId"))

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
        for Component { ci, config, .. } in components {
            let filename = settings.out_dir.join(format!("{}.dart", ci.namespace()));
            let tokens = DartWrapper::new(ci, config).generate();
            let file = std::fs::File::create(filename)?;

            let mut w = fmt::IoWriter::new(file);

            let mut fmt = fmt::Config::from_lang::<Dart>();
            if settings.try_format_code {
                fmt = fmt.with_indentation(fmt::Indentation::Space(2));
            }
            let config = dart::Config::default();

            tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
        }

        // Run full Dart formatter on the output directory as a best-effort step.
        // This is non-fatal: failures will only emit a warning.
        let mut format_command = Command::new("dart");
        format_command
            .current_dir(&settings.out_dir)
            .arg("format")
            .arg(".");
        match format_command.spawn().and_then(|mut c| c.wait()) {
            Ok(status) if status.success() => {}
            Ok(_) | Err(_) => {
                println!(
                    "WARNING: dart format failed or is unavailable; proceeding without full formatting"
                );
            }
        }
        Ok(())
    }

    fn new_config(&self, root_toml: &toml::value::Value) -> Result<Self::Config> {
        Ok(
            match root_toml.get("bindings").and_then(|b| b.get("dart")) {
                Some(v) => v.clone().try_into()?,
                None => Default::default(),
            },
        )
    }

    fn update_component_configs(
        &self,
        settings: &uniffi_bindgen::GenerationSettings,
        components: &mut Vec<uniffi_bindgen::Component<Self::Config>>,
    ) -> Result<()> {
        for c in &mut *components {
            c.config.cdylib_name.get_or_insert_with(|| {
                settings
                    .cdylib
                    .clone()
                    .unwrap_or_else(|| format!("uniffi_{}", c.ci.namespace()))
            });
        }
        Ok(())
    }
}

pub struct LocalConfigSupplier(String);
impl BindgenCrateConfigSupplier for LocalConfigSupplier {
    fn get_udl(&self, _crate_name: &str, _udl_name: &str) -> Result<String> {
        let file = std::fs::File::open(self.0.clone())?;
        let mut reader = std::io::BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        Ok(content)
    }
}

/// Config supplier for library mode that locates UDL files from dependency crates.
/// This implementation matches uniffi_bindgen's CrateConfigSupplier approach.
pub struct ConfigFileSupplier {
    config_file_path: String,
    crate_paths: HashMap<String, Utf8PathBuf>,
}

impl ConfigFileSupplier {
    /// Create a new ConfigFileSupplier from cargo metadata and a config file path
    pub fn new(config_file_path: String, metadata: Metadata) -> Self {
        // Build a map of crate names to their manifest directories
        // This matches uniffi_bindgen's CrateConfigSupplier::from(Metadata) implementation
        let crate_paths: HashMap<String, Utf8PathBuf> = metadata
            .packages
            .iter()
            .flat_map(|p| {
                p.targets
                    .iter()
                    .filter(|t| {
                        !t.is_bin()
                            && !t.is_example()
                            && !t.is_test()
                            && !t.is_bench()
                            && !t.is_custom_build()
                    })
                    .filter_map(|t| {
                        p.manifest_path
                            .parent()
                            .map(|p| (t.name.replace('-', "_"), p.to_owned()))
                    })
            })
            .collect();

        Self {
            config_file_path,
            crate_paths,
        }
    }
}

impl BindgenCrateConfigSupplier for ConfigFileSupplier {
    fn get_udl(&self, crate_name: &str, udl_name: &str) -> Result<String> {
        // This implementation matches uniffi_bindgen's CrateConfigSupplier::get_udl
        let path = self
            .crate_paths
            .get(crate_name)
            .context(format!("No path known to UDL files for '{crate_name}'"))?
            .join("src")
            .join(format!("{udl_name}.udl"));
        if path.exists() {
            Ok(std::fs::read_to_string(path)?)
        } else {
            bail!(format!("No UDL file found at '{path}'"));
        }
    }

    fn get_toml(&self, _crate_name: &str) -> Result<Option<toml::value::Table>> {
        // Load the config file specified for this binding generation
        let file = std::fs::File::open(self.config_file_path.clone())?;
        let mut reader = std::io::BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        let toml_value: toml::value::Value = toml::from_str(&content)?;
        if let toml::value::Value::Table(table) = toml_value {
            Ok(Some(table))
        } else {
            Ok(None)
        }
    }

    fn get_toml_path(&self, crate_name: &str) -> Option<Utf8PathBuf> {
        // This implementation matches uniffi_bindgen's CrateConfigSupplier::get_toml_path
        self.crate_paths
            .get(crate_name)
            .map(|p| p.join("uniffi.toml"))
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
        // In library mode, we need cargo metadata to locate UDL files from dependencies
        let metadata = cargo_metadata::MetadataCommand::new()
            .exec()
            .context("Failed to run cargo metadata")?;

        let config_supplier: Box<dyn BindgenCrateConfigSupplier> =
            if let Some(config_path) = config_file_override {
                Box::new(ConfigFileSupplier::new(config_path.to_string(), metadata))
            } else {
                Box::new(LocalConfigSupplier(udl_file.to_string()))
            };

        uniffi_bindgen::library_mode::generate_bindings(
            library_file,
            None, // crate name filter
            &DartBindingGenerator {},
            config_supplier.as_ref(),
            None,
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
