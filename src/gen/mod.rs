use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Read;

use anyhow::Result;
use camino::Utf8Path;

use genco::fmt;
use genco::prelude::*;
use serde::{Deserialize, Serialize};
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
}

impl From<&ComponentInterface> for Config {
    fn from(ci: &ComponentInterface) -> Self {
        Config {
            package_name: Some(ci.namespace().to_owned()),
            cdylib_name: Some(ci.namespace().to_owned()),
            external_packages: HashMap::new(),
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
        let package_name = self.config.package_name();
        let libname = self.config.cdylib_name();

        let (type_helper_code, functions_definitions) = &self.type_renderer.render();

        fn uniffi_function_definitions(ci: &ComponentInterface) -> dart::Tokens {
            let mut definitions = quote!();
            let mut defined_functions = HashSet::new(); // Track defined function names

            for fun in ci.iter_ffi_function_definitions() {
                let fun_name = fun.name().to_owned();

                // Check for duplicate function names
                if !defined_functions.insert(fun_name.clone()) {
                    // Function name already exists, skip to prevent duplicate definition
                    continue;
                }

                let (native_return_type, dart_return_type) = match fun.return_type() {
                    Some(return_type) => (
                        quote! { $(DartCodeOracle::ffi_native_type_label(Some(return_type), ci)) },
                        quote! { $(DartCodeOracle::ffi_dart_type_label(Some(return_type), ci)) },
                    ),
                    None => (quote! { Void }, quote! { void }),
                };

                let (native_args, dart_args) = {
                    let mut native_args = quote!();
                    let mut dart_args = quote!();

                    for arg in fun.arguments() {
                        native_args.append(
                            quote!($(DartCodeOracle::ffi_native_type_label(Some(&arg.type_()), ci)),),
                        );
                        dart_args.append(
                            quote!($(DartCodeOracle::ffi_dart_type_label(Some(&arg.type_()), ci)),),
                        );
                    }

                    if fun.has_rust_call_status_arg() {
                        native_args.append(quote!(Pointer<RustCallStatus>));
                        dart_args.append(quote!(Pointer<RustCallStatus>));
                    }

                    (native_args, dart_args)
                };

                let lookup_fn = quote! {
                    _dylib.lookupFunction<
                        $native_return_type Function($(&native_args)),
                        $(&dart_return_type) Function($(&dart_args))
                    >($(format!("\"{fun_name}\"")))
                };

                definitions.append(quote! {
                    late final $dart_return_type Function($dart_args) $fun_name = $lookup_fn;
                });
            }

            definitions
        }

        quote! {
            library $package_name;

            $(type_helper_code) // Imports, Types and Type Helper
            
            $(functions_definitions)

            class _UniffiLib {
                _UniffiLib._();

                static final DynamicLibrary _dylib = _open();

                static DynamicLibrary _open() {
                  if (Platform.isAndroid) return DynamicLibrary.open($(format!("\"${{Directory.current.path}}/lib{libname}.so\"")));
                  if (Platform.isIOS) return DynamicLibrary.executable();
                  if (Platform.isLinux) return DynamicLibrary.open($(format!("\"${{Directory.current.path}}/lib{libname}.so\"")));
                  if (Platform.isMacOS) return DynamicLibrary.open($(format!("\"lib{libname}.dylib\"")));
                  if (Platform.isWindows) return DynamicLibrary.open($(format!("\"{libname}.dll\"")));
                  throw UnsupportedError("Unsupported platform: ${Platform.operatingSystem}");
                }

                static final _UniffiLib instance = _UniffiLib._();

                $(uniffi_function_definitions(self.ci))

                static void _checkApiVersion() {
                    final bindingsVersion = $(self.ci.uniffi_contract_version());
                    final scaffoldingVersion = _UniffiLib.instance.$(self.ci.ffi_uniffi_contract_version().name())();
                    if (bindingsVersion != scaffoldingVersion) {
                      throw UniffiInternalError.panicked("UniFFI contract version mismatch: bindings version $bindingsVersion, scaffolding version $scaffoldingVersion");
                    }
                }

                static void _checkApiChecksums() {
                    $(for (name, expected_checksum) in self.ci.iter_checksums() =>
                        if (_UniffiLib.instance.$(name)() != $expected_checksum) {
                          throw UniffiInternalError.panicked("UniFFI API checksum mismatch");
                        }
                    )
                }
            }

            void initialize() {
                _UniffiLib._open();
            }

            void ensureInitialized() {
                _UniffiLib._checkApiVersion();
                _UniffiLib._checkApiChecksums();
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
            let filename = settings
                .out_dir
                .join(format!("{}.dart", ci.namespace()));
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
        return Ok(content);
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
        uniffi_bindgen::library_mode::generate_bindings(
            library_file,
            None,
            &DartBindingGenerator {},
            &LocalConfigSupplier(udl_file.to_string()),
            None,
            out_dir_override.unwrap(),
            false,
        )?;
        Ok(())
    } else {
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
