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
use uniffi_bindgen::interface::AsType;
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
mod web;

pub use code_type::CodeType;

fn default_web_unsupported_policy() -> String {
    "warn".to_string()
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    package_name: Option<String>,
    cdylib_name: Option<String>,
    #[serde(default)]
    external_packages: HashMap<String, String>,
    asset_id: Option<String>,
    #[serde(default)]
    generate_web: bool,
    wasm_module_name: Option<String>,
    #[serde(default)]
    wasm_crate_features: Vec<String>,
    #[serde(default = "default_web_unsupported_policy")]
    web_unsupported_policy: String,
}

impl From<&ComponentInterface> for Config {
    fn from(ci: &ComponentInterface) -> Self {
        Config {
            package_name: Some(ci.namespace().to_owned()),
            cdylib_name: Some(ci.namespace().to_owned()),
            external_packages: HashMap::new(),
            asset_id: None,
            generate_web: false,
            wasm_module_name: None,
            wasm_crate_features: Vec::new(),
            web_unsupported_policy: default_web_unsupported_policy(),
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

    pub fn generate_web(&self) -> bool {
        self.generate_web
    }

    pub fn wasm_module_name(&self, namespace: &str) -> String {
        self.wasm_module_name
            .clone()
            .unwrap_or_else(|| format!("__uniffi_{namespace}"))
    }

    pub fn web_unsupported_policy(&self) -> &str {
        &self.web_unsupported_policy
    }

    pub fn wasm_crate_features(&self) -> &[String] {
        &self.wasm_crate_features
    }
}

pub struct DartWrapper<'a> {
    config: &'a Config,
    ci: &'a ComponentInterface,
    type_renderer: TypeHelpersRenderer<'a>,
}

impl<'a> DartWrapper<'a> {
    pub fn new(ci: &'a ComponentInterface, config: &'a Config) -> Self {
        let type_renderer = TypeHelpersRenderer::with_import_prefix(ci, "../".to_string());
        DartWrapper {
            ci,
            config,
            type_renderer,
        }
    }

    fn generate_entry_point(&self) -> dart::Tokens {
        let ns = self.ci.namespace();
        quote! {
            export $(quoted(format!("src/{ns}_stub.dart")))
              if (dart.library.ffi) $(quoted(format!("src/{ns}_native.dart")))
              if (dart.library.js_interop) $(quoted(format!("src/{ns}_web.dart")));
        }
    }

    fn generate_web_placeholder(&self) -> dart::Tokens {
        let ns = self.ci.namespace();
        quote! {
            export $(quoted(format!("{ns}_stub.dart")));
        }
    }

    fn generate_web(&self) -> Result<dart::Tokens> {
        web::WebDartWrapper::new(self.ci, self.config).generate()
    }

    fn generate_native(&self) -> dart::Tokens {
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

    fn generate_stub(&self) -> dart::Tokens {
        let ns = self.ci.namespace();
        let import_prefix = "../";

        let modules_to_import = self
            .ci
            .iter_external_types()
            .map(|ty| {
                self.ci
                    .namespace_for_type(ty)
                    .expect("external type should have module_path")
            })
            .collect::<std::collections::BTreeSet<_>>();

        let stub_imports = quote! {
            import "dart:async";
            import "dart:typed_data";
            $( for imp in &modules_to_import {
                $(format!("import \"{}{}.dart\"", import_prefix, imp));
            })
        };

        let record_stubs = self.generate_stub_records();
        let enum_stubs = self.generate_stub_enums();
        let object_stubs = self.generate_stub_objects();
        let function_stubs = self.generate_stub_functions();
        let callback_stubs = self.generate_stub_callback_interfaces();
        let stream_stubs = self.generate_stub_streams();

        quote! {
            $stub_imports

            $record_stubs

            $enum_stubs

            $callback_stubs

            $object_stubs

            $function_stubs

            $stream_stubs

            void ensureInitialized() {
                throw UnsupportedError($(quoted(format!("{ns} is not supported on this platform"))));
            }

            @Deprecated("Use ensureInitialized instead")
            void initialize() {
                throw UnsupportedError($(quoted(format!("{ns} is not supported on this platform"))));
            }
        }
    }

    fn generate_stub_records(&self) -> dart::Tokens {
        let mut tokens = quote!();
        for rec in self.ci.record_definitions() {
            let cls_name = DartCodeOracle::class_name(rec.name());
            let fields: Vec<dart::Tokens> = rec
                .fields()
                .iter()
                .map(|f| {
                    let field_name = DartCodeOracle::var_name(f.name());
                    let field_type = types::generate_type(&f.as_type());
                    quote!(final $field_type $field_name;)
                })
                .collect();
            let constructor_params: Vec<dart::Tokens> = rec
                .fields()
                .iter()
                .map(|f| {
                    let field_name = DartCodeOracle::var_name(f.name());
                    quote!(required this.$field_name)
                })
                .collect();
            let constructor = if rec.fields().is_empty() {
                quote!($(&cls_name)();)
            } else {
                quote!($(&cls_name)({$(for p in constructor_params => $p, )});)
            };
            tokens.append(quote! {
                class $(&cls_name) {
                    $(for f in fields => $f)
                    $constructor
                }
            });
        }
        tokens
    }

    fn generate_stub_enums(&self) -> dart::Tokens {
        let mut tokens = quote!();
        for enm in self.ci.enum_definitions() {
            let cls_name = DartCodeOracle::class_name(enm.name());
            let is_error = self.ci.is_name_used_as_error(enm.name());
            let implements_exception = if is_error {
                quote!( implements Exception)
            } else {
                quote!()
            };

            if enm.is_flat() {
                tokens.append(quote! {
                    enum $(&cls_name) $(&implements_exception) {
                        $(for v in enm.variants() =>
                            $(DartCodeOracle::enum_variant_name(v.name())),)
                        ;
                    }
                });
            } else {
                tokens.append(quote! {
                    abstract class $(&cls_name) $(&implements_exception) {}
                });
                for variant in enm.variants() {
                    let variant_cls = format!(
                        "{}{}",
                        DartCodeOracle::class_name(variant.name()),
                        &cls_name
                    );
                    let field_decls: Vec<dart::Tokens> = variant
                        .fields()
                        .iter()
                        .enumerate()
                        .map(|(i, f)| {
                            let name = if f.name().is_empty() {
                                format!("v{i}")
                            } else {
                                DartCodeOracle::var_name(f.name())
                            };
                            let ty = types::generate_type(&f.as_type());
                            quote!(final $ty $name;)
                        })
                        .collect();
                    let ctor_params: Vec<dart::Tokens> = variant
                        .fields()
                        .iter()
                        .enumerate()
                        .map(|(i, f)| {
                            let name = if f.name().is_empty() {
                                format!("v{i}")
                            } else {
                                DartCodeOracle::var_name(f.name())
                            };
                            if variant.fields().len() > 1 {
                                quote!(required this.$name)
                            } else {
                                quote!(this.$name)
                            }
                        })
                        .collect();
                    let ctor_list = if variant.fields().len() > 1 {
                        quote!({$(for p in ctor_params => $p, )})
                    } else {
                        quote!($(for p in ctor_params => $p, ))
                    };
                    let to_string_method: dart::Tokens = if is_error {
                        quote! {
                            @override
                            String toString() { return $(quoted(&variant_cls)); }
                        }
                    } else {
                        quote!()
                    };
                    tokens.append(quote! {
                        class $(&variant_cls) extends $(&cls_name) {
                            $(for f in &field_decls => $f)
                            $(&variant_cls)($ctor_list);
                            $to_string_method
                        }
                    });
                }
            }
        }
        tokens
    }

    fn generate_stub_objects(&self) -> dart::Tokens {
        let mut tokens = quote!();
        for obj in self.ci.object_definitions() {
            let cls_name = DartCodeOracle::class_name(obj.name());

            if obj.has_callback_interface() {
                let methods: Vec<dart::Tokens> = obj
                    .methods()
                    .iter()
                    .map(|m| {
                        let method_name = DartCodeOracle::fn_name(m.name());
                        let ret = stub_return_type(m.return_type(), m.is_async());
                        let params = stub_method_params(&m.arguments());
                        quote!($ret $method_name($params);)
                    })
                    .collect();
                tokens.append(quote! {
                    abstract class $(&cls_name) {
                        $(for m in methods => $m)
                    }
                });
                continue;
            }

            if obj.is_trait_interface() {
                let methods: Vec<dart::Tokens> = obj
                    .methods()
                    .iter()
                    .map(|m| {
                        let method_name = DartCodeOracle::fn_name(m.name());
                        let ret = stub_return_type(m.return_type(), m.is_async());
                        let params = stub_method_params(&m.arguments());
                        quote!($ret $method_name($params);)
                    })
                    .collect();
                tokens.append(quote! {
                    abstract class $(&cls_name) {
                        void dispose();
                        $(for m in methods => $m)
                    }
                });
                continue;
            }

            let interface_name = DartCodeOracle::object_interface_name(self.ci, obj);

            let interface_method_sigs: Vec<dart::Tokens> = obj
                .methods()
                .iter()
                .map(|m| {
                    let method_name = DartCodeOracle::fn_name(m.name());
                    let ret = stub_return_type(m.return_type(), m.is_async());
                    let params = stub_method_params(&m.arguments());
                    quote!($ret $method_name($params);)
                })
                .collect();

            tokens.append(quote! {
                abstract class $(&interface_name) {
                    void dispose();
                    $(for m in &interface_method_sigs => $m)
                }
            });

            let unsupported = format!("{cls_name} is not supported on this platform");
            let mut ctor_stubs = Vec::new();
            for ctor in obj.constructors() {
                let ctor_name = ctor.name();
                let params = stub_method_params(&ctor.arguments());
                if ctor.is_async() {
                    if ctor_name == "new" {
                        ctor_stubs.push(quote! {
                            static Future<$(&cls_name)> new_($params) =>
                                throw UnsupportedError($(quoted(&unsupported)));
                        });
                    } else {
                        ctor_stubs.push(quote! {
                            static Future<$(&cls_name)> $(DartCodeOracle::fn_name(ctor_name))($params) =>
                                throw UnsupportedError($(quoted(&unsupported)));
                        });
                    }
                } else if ctor_name == "new" {
                    ctor_stubs.push(quote! {
                        $(&cls_name)($params) { throw UnsupportedError($(quoted(&unsupported))); }
                    });
                } else {
                    ctor_stubs.push(quote! {
                        $(&cls_name).$(DartCodeOracle::fn_name(ctor_name))($params) { throw UnsupportedError($(quoted(&unsupported))); }
                    });
                }
            }

            let method_stubs: Vec<dart::Tokens> = obj
                .methods()
                .iter()
                .map(|m| {
                    let method_name = DartCodeOracle::fn_name(m.name());
                    let ret = stub_return_type(m.return_type(), m.is_async());
                    let params = stub_method_params(&m.arguments());
                    quote!($ret $method_name($params) => throw UnsupportedError($(quoted(&unsupported)));)
                })
                .collect();

            let is_error = self.ci.is_name_used_as_error(obj.name());
            let error_impl = if is_error {
                quote!( implements $(&interface_name), Exception)
            } else {
                quote!( implements $(&interface_name))
            };

            tokens.append(quote! {
                class $(&cls_name) $error_impl {
                    $(for c in ctor_stubs => $c)
                    $(for m in method_stubs => $m)
                    void dispose() => throw UnsupportedError($(quoted(&unsupported)));
                }
            });
        }
        tokens
    }

    fn generate_stub_functions(&self) -> dart::Tokens {
        let mut tokens = quote!();
        for func in self.ci.function_definitions() {
            let fn_name = DartCodeOracle::fn_name(func.name());
            let ret = stub_return_type(func.return_type(), func.is_async());
            let params = stub_method_params(&func.arguments());
            let msg = format!("{fn_name} is not supported on this platform");
            tokens.append(quote! {
                $ret $fn_name($params) => throw UnsupportedError($(quoted(msg)));
            });
        }
        tokens
    }

    fn generate_stub_callback_interfaces(&self) -> dart::Tokens {
        let mut tokens = quote!();
        for cb in self.ci.callback_interface_definitions() {
            let cls_name = DartCodeOracle::class_name(cb.name());
            let methods: Vec<dart::Tokens> = cb
                .methods()
                .iter()
                .map(|m| {
                    let method_name = DartCodeOracle::fn_name(m.name());
                    let ret = stub_return_type(m.return_type(), m.is_async());
                    let params = stub_method_params(&m.arguments());
                    quote!($ret $method_name($params);)
                })
                .collect();
            tokens.append(quote! {
                abstract class $(&cls_name) {
                    $(for m in methods => $m)
                }
            });
        }
        tokens
    }

    fn generate_stub_streams(&self) -> dart::Tokens {
        let mut tokens = quote!();
        for obj in self.ci.object_definitions() {
            if obj.name().contains("StreamExt") {
                let fn_name = DartCodeOracle::fn_name(&obj.name().replace("StreamExt", ""));
                let msg = format!("{fn_name} is not supported on this platform");
                tokens.append(quote! {
                    $fn_name() async* {
                        throw UnsupportedError($(quoted(msg)));
                    }
                });
            }
        }
        tokens
    }
}

fn stub_return_type(
    return_type: Option<&uniffi_bindgen::interface::Type>,
    is_async: bool,
) -> dart::Tokens {
    let base = if let Some(ret) = return_type {
        types::generate_type(ret)
    } else {
        quote!(void)
    };
    if is_async {
        quote!(Future<$base>)
    } else {
        base
    }
}

fn stub_method_params(args: &[&uniffi_bindgen::interface::Argument]) -> dart::Tokens {
    if args.is_empty() {
        return quote!();
    }
    quote!({$(for arg in args =>
        required $(types::generate_type(&arg.as_type())) $(DartCodeOracle::var_name(arg.name())),
    )})
}

pub struct DartBindingGenerator;

fn write_dart_file(path: &Utf8Path, tokens: dart::Tokens, try_format_code: bool) -> Result<()> {
    let file = std::fs::File::create(path)?;
    let mut w = fmt::IoWriter::new(file);
    let mut fmt_cfg = fmt::Config::from_lang::<Dart>();
    if try_format_code {
        fmt_cfg = fmt_cfg.with_indentation(fmt::Indentation::Space(2));
    }
    let dart_cfg = dart::Config::default();
    tokens.format_file(&mut w.as_formatter(&fmt_cfg), &dart_cfg)?;
    Ok(())
}

impl BindingGenerator for DartBindingGenerator {
    type Config = Config;

    fn write_bindings(
        &self,
        settings: &uniffi_bindgen::GenerationSettings,
        components: &[uniffi_bindgen::Component<Self::Config>],
    ) -> Result<()> {
        web::validate_unique_wasm_module_names(
            components
                .iter()
                .filter(|component| component.config.generate_web())
                .map(|component| {
                    (
                        component.ci.namespace().to_string(),
                        component.config.wasm_module_name(component.ci.namespace()),
                    )
                }),
        )?;

        for Component { ci, config, .. } in components {
            let ns = ci.namespace();
            let wrapper = DartWrapper::new(ci, config);
            let web_tokens = if config.generate_web() {
                wrapper.generate_web()?
            } else {
                wrapper.generate_web_placeholder()
            };

            let src_dir = settings.out_dir.join("src");
            std::fs::create_dir_all(&src_dir)?;

            write_dart_file(
                &settings.out_dir.join(format!("{ns}.dart")),
                wrapper.generate_entry_point(),
                settings.try_format_code,
            )?;
            write_dart_file(
                &src_dir.join(format!("{ns}_native.dart")),
                wrapper.generate_native(),
                settings.try_format_code,
            )?;
            write_dart_file(
                &src_dir.join(format!("{ns}_stub.dart")),
                wrapper.generate_stub(),
                settings.try_format_code,
            )?;
            write_dart_file(
                &src_dir.join(format!("{ns}_web.dart")),
                web_tokens,
                settings.try_format_code,
            )?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use genco::fmt;

    fn render_tokens(tokens: dart::Tokens) -> String {
        let mut buf = Vec::new();
        {
            let mut w = fmt::IoWriter::new(&mut buf);
            let fmt_cfg = fmt::Config::from_lang::<Dart>();
            let dart_cfg = dart::Config::default();
            tokens
                .format_file(&mut w.as_formatter(&fmt_cfg), &dart_cfg)
                .expect("failed to render tokens");
        }
        String::from_utf8(buf).expect("non-UTF8 output")
    }

    fn normalize_whitespace(s: &str) -> String {
        s.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn stub_export_statement(output: &str, stub_file: &str) -> String {
        let export_start = output
            .find(&format!("export \"{stub_file}\""))
            .expect("stub export missing");
        let export_end = output[export_start..]
            .find(';')
            .map(|offset| export_start + offset)
            .expect("stub export terminator missing");

        normalize_whitespace(&output[export_start..=export_end])
    }

    fn assert_stub_export_hides(output: &str, stub_file: &str, hidden_names: &[&str]) {
        let statement = stub_export_statement(output, stub_file);
        let hide_list = statement
            .split(" hide ")
            .nth(1)
            .expect("stub export should contain a hide clause")
            .trim_end_matches(';');
        let actual: Vec<_> = hide_list.split(',').map(str::trim).collect();

        assert_eq!(actual, hidden_names, "unexpected stub export hide list");
    }

    #[test]
    fn entry_point_ffi_precedes_js_interop() {
        let ci = ComponentInterface::new("test_ns");
        let config = Config::from(&ci);
        let wrapper = DartWrapper::new(&ci, &config);
        let output = render_tokens(wrapper.generate_entry_point());

        let ffi_pos = output
            .find("dart.library.ffi")
            .expect("ffi selector missing from entry point");
        let js_pos = output
            .find("dart.library.js_interop")
            .expect("js_interop selector missing from entry point");

        assert!(
            ffi_pos < js_pos,
            "ffi must appear before js_interop for native-wins precedence, \
             but ffi at {ffi_pos}, js_interop at {js_pos}.\nGenerated:\n{output}"
        );
    }

    #[test]
    fn web_output_includes_js_interop_runtime() {
        let ci = ComponentInterface::from_webidl(
            r#"
            namespace test_ns {
                string greet(string name);
            };
            "#,
            "test_ns",
        )
        .expect("test UDL should parse");
        let mut config = Config::from(&ci);
        config.generate_web = true;
        config.wasm_module_name = Some("__uniffi_test_ns".to_string());
        let wrapper = DartWrapper::new(&ci, &config);
        let output = render_tokens(wrapper.generate_web().expect("web generation"));

        assert!(output.contains("import \"dart:js_interop\";"));
        assert_stub_export_hides(
            &output,
            "test_ns_stub.dart",
            &["ensureInitialized", "initialize", "greet"],
        );
        assert!(output.contains("__uniffi_test_ns.init"));
        assert!(output.contains("__uniffi_test_ns.test_ns_greet"));
        assert!(output.contains("Future<void> ensureInitialized({String? wasmPath})"));
        assert!(output.contains("uniffi_error"));
        assert!(output.contains("uniffi_internal"));
        assert!(output.contains("uniffi_panic"));
    }

    #[test]
    fn duplicate_wasm_module_names_are_rejected() {
        let result = web::validate_unique_wasm_module_names([
            ("first".to_string(), "__uniffi_shared".to_string()),
            ("second".to_string(), "__uniffi_shared".to_string()),
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn default_wasm_module_name_uses_namespace_not_package_name() {
        let ci = ComponentInterface::from_webidl(
            r#"
            namespace component_ns {
                string greet();
            };
            "#,
            "component_ns",
        )
        .expect("test UDL should parse");
        let mut config = Config::from(&ci);
        config.package_name = Some("shared_package".to_string());

        assert_eq!(
            config.wasm_module_name(ci.namespace()),
            "__uniffi_component_ns"
        );
    }

    #[test]
    fn distinct_wasm_module_names_are_accepted() {
        let result = web::validate_unique_wasm_module_names([
            ("first".to_string(), "__uniffi_first".to_string()),
            ("second".to_string(), "__uniffi_second".to_string()),
        ]);

        assert!(result.is_ok());
    }

    #[test]
    fn web_unsupported_policy_error_fails_generation() {
        let ci = ComponentInterface::from_webidl(
            include_str!("../../fixtures/simple-fns/src/api.udl"),
            "simple_fns",
        )
        .expect("simple-fns UDL should parse");
        let mut config = Config::from(&ci);
        config.generate_web = true;
        config.web_unsupported_policy = "error".to_string();
        let wrapper = DartWrapper::new(&ci, &config);

        assert!(wrapper.generate_web().is_err());
    }

    #[test]
    fn web_unsupported_policy_error_rejects_object_only_components() {
        let ci = ComponentInterface::from_webidl(
            r#"
            namespace test_ns {
            };

            interface Thing {
                constructor();
                void touch();
            };
            "#,
            "test_ns",
        )
        .expect("test UDL should parse");
        let mut config = Config::from(&ci);
        config.generate_web = true;
        config.web_unsupported_policy = "error".to_string();
        let wrapper = DartWrapper::new(&ci, &config);

        assert!(wrapper.generate_web().is_err());
    }

    #[test]
    fn web_output_excludes_64_bit_integer_functions_until_lossless() {
        let ci = ComponentInterface::from_webidl(
            r#"
            namespace test_ns {
                i64 signed_big(i64 value);
                u64 unsigned_big(u64 value);
            };
            "#,
            "test_ns",
        )
        .expect("test UDL should parse");
        let mut config = Config::from(&ci);
        config.generate_web = true;
        let wrapper = DartWrapper::new(&ci, &config);
        let output = render_tokens(wrapper.generate_web().expect("web generation"));

        assert_stub_export_hides(
            &output,
            "test_ns_stub.dart",
            &["ensureInitialized", "initialize"],
        );
        assert!(!output.contains("__uniffi_test_ns.test_ns_signed_big"));
        assert!(!output.contains("__uniffi_test_ns.test_ns_unsigned_big"));
        assert!(!output.contains("JSBigInt"));
    }

    #[test]
    fn web_output_selectively_hides_supported_simple_fns() {
        let ci = ComponentInterface::from_webidl(
            include_str!("../../fixtures/simple-fns/src/api.udl"),
            "simple_fns",
        )
        .expect("simple-fns UDL should parse");
        let mut config = Config::from(&ci);
        config.generate_web = true;
        let wrapper = DartWrapper::new(&ci, &config);
        let output = render_tokens(wrapper.generate_web().expect("web generation"));

        assert_stub_export_hides(
            &output,
            "simple_fns_stub.dart",
            &[
                "ensureInitialized",
                "initialize",
                "byteToU32",
                "getInt",
                "getString",
                "stringIdentity",
            ],
        );
        assert!(output.contains("getString"));
        assert!(output.contains("getInt"));
        assert!(output.contains("stringIdentity"));
        assert!(output.contains("byteToU32"));
        assert!(output.contains("__uniffi_simple_fns.simple_fns_get_string"));
        assert!(output.contains("__uniffi_simple_fns.simple_fns_byte_to_u32"));
        assert!(!output.contains("__uniffi_simple_fns.simple_fns_dummy"));
        assert!(!output.contains("__uniffi_simple_fns.simple_fns_new_set"));
    }
}
