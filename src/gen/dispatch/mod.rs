//! Opt-in generated native adapters for synchronous, 64-bit callback dispatch.
use super::{oracle::DartCodeOracle, Config};
use anyhow::{bail, Result};
use genco::prelude::*;
use std::collections::BTreeMap;
use std::fmt::Write;
use uniffi_bindgen::{
    interface::{AsType, Callable, FfiDefinition, FfiFunction, FfiType, ObjectImpl, Type},
    ComponentInterface,
};

fn native_type(t: &FfiType) -> String {
    match t {
        FfiType::UInt8 => "u8".into(),
        FfiType::Int8 => "i8".into(),
        FfiType::UInt16 => "u16".into(),
        FfiType::Int16 => "i16".into(),
        FfiType::UInt32 => "u32".into(),
        FfiType::Int32 => "i32".into(),
        FfiType::UInt64 | FfiType::Handle => "u64".into(),
        FfiType::Int64 => "i64".into(),
        FfiType::Float32 => "f32".into(),
        FfiType::Float64 => "f64".into(),
        FfiType::RustBuffer(_) => "RustBuffer".into(),
        FfiType::ForeignBytes => "ForeignBytes".into(),
        FfiType::RustCallStatus => "RustCallStatus".into(),
        FfiType::Callback(n) | FfiType::Struct(n) => n.clone(),
        FfiType::Reference(t) => format!("*const {}", native_type(t)),
        FfiType::MutReference(t) => format!("*mut {}", native_type(t)),
        FfiType::VoidPointer => "*mut std::ffi::c_void".into(),
    }
}
fn result_type(t: Option<&FfiType>) -> String {
    t.map(native_type).unwrap_or_else(|| "()".into())
}
fn has_callback(t: &Type) -> bool {
    t.iter_types().any(|t| {
        matches!(
            t,
            Type::CallbackInterface { .. } | Type::Object { imp: ObjectImpl::CallbackTrait, .. }
        )
    })
}
pub fn validate(ci: &ComponentInterface) -> Result<()> {
    if ci
        .function_definitions()
        .iter()
        .any(|f| DartCodeOracle::fn_name(f.name()) == "closeCallbackDispatcher")
    {
        bail!("callback_dispatch reserves the Dart function name closeCallbackDispatcher");
    }
    if ci.iter_external_types().next().is_some() {
        bail!("callback_dispatch does not yet support external component types");
    }
    if ci.iter_callables().any(|c| c.is_async()) || ci.has_async_callback_interface_definition() {
        bail!(
            "callback_dispatch supports synchronous Rust APIs and synchronous Dart callbacks only"
        );
    }
    if ci.object_definitions().iter().any(|o| !o.has_callback_interface()) {
        bail!("callback_dispatch does not yet support Rust object lifetimes; use a function API");
    }
    for record in ci.record_definitions() {
        if record.fields().iter().any(|f| has_callback(&f.as_type())) {
            bail!("callback_dispatch does not yet support callback values inside records");
        }
    }
    for enm in ci.enum_definitions() {
        if enm.variants().iter().flat_map(|v| v.fields()).any(|f| has_callback(&f.as_type())) {
            bail!("callback_dispatch does not yet support callback values inside enums");
        }
    }
    for call in ci.iter_callables() {
        if call.return_type().is_some_and(has_callback) {
            bail!("callback_dispatch does not yet support returning callback values to Dart");
        }
    }
    for cb in ci.callback_interface_definitions().iter().flat_map(|c| c.methods()).chain(
        ci.object_definitions()
            .iter()
            .filter(|o| o.has_callback_interface())
            .flat_map(|o| o.methods()),
    ) {
        for t in cb.arguments().iter().map(|a| a.as_type()).chain(cb.return_type().cloned()) {
            if matches!(
                t,
                Type::Enum { .. } | Type::Duration | Type::Timestamp | Type::Custom { .. }
            ) {
                bail!(
                    "callback_dispatch does not yet support this direct callback wire type: {t:?}"
                );
            }
        }
        if cb.iter_types().any(|t| {
            matches!(
                t,
                Type::CallbackInterface { .. }
                    | Type::Object { imp: ObjectImpl::CallbackTrait, .. }
            )
        }) {
            bail!("callback_dispatch does not yet support passing callback values through callback methods");
        }
    }
    Ok(())
}
pub fn is_adapter(fun: &FfiFunction) -> bool {
    fun.has_rust_call_status_arg() || fun.name().contains("_fn_init_callback_vtable_")
}
pub fn declaration(fun: &FfiFunction, asset: &str) -> dart::Tokens {
    let native_ret = fun
        .return_type()
        .map(|t| DartCodeOracle::ffi_native_type_label(Some(t)).to_string().unwrap())
        .unwrap_or_else(|| "Void".into());
    let dart_ret = DartCodeOracle::ffi_dart_type_label(fun.return_type()).to_string().unwrap();
    let mut native_args = Vec::new();
    let mut dart_args = Vec::new();
    let mut args = Vec::new();
    for (i, arg) in fun.arguments().iter().enumerate() {
        native_args
            .push(DartCodeOracle::ffi_native_type_label(Some(&arg.type_())).to_string().unwrap());
        dart_args.push(format!(
            "{} a{i}",
            DartCodeOracle::ffi_dart_type_label(Some(&arg.type_())).to_string().unwrap()
        ));
        args.push(format!("a{i}"));
    }
    if fun.has_rust_call_status_arg() {
        native_args.push("Pointer<RustCallStatus>".into());
        dart_args.push("Pointer<RustCallStatus> status".into());
        args.push("status".into());
    }
    let sig = format!("{native_ret} Function({})", native_args.join(", "));
    let name = fun.name();
    let extra = if native_args.is_empty() {
        String::new()
    } else {
        format!(", {}", native_args.join(", "))
    };
    let extra_dart =
        if dart_args.is_empty() { String::new() } else { format!(", {}", dart_args.join(", ")) };
    let extra_values =
        if args.is_empty() { String::new() } else { format!(", {}", args.join(", ")) };
    let transfer = if fun.name().contains("_fn_init_callback_vtable_") {
        ""
    } else {
        "_uniffiDispatch.transferred();"
    };
    let source = format!(
        r#"
@Native<{sig}>(assetId: {asset}, symbol: '{name}')
external {dart_ret} _direct_{name}({dart_params});
@Native<{native_ret} Function(Uint64, Pointer<NativeFunction<{sig}>>{extra})>(assetId: _uniffiDispatchAsset, symbol: 'dispatch_{name}')
external {dart_ret} _dispatch_{name}(int endpoint, Pointer<NativeFunction<{sig}>> original{extra_dart});
{dart_ret} {name}({dart_params}) {{
  final endpoint = _uniffiDispatch.id;
  final original = Native.addressOf<NativeFunction<{sig}>>(_direct_{name});
  {transfer}
  return _dispatch_{name}(endpoint, original{extra_values});
}}
"#,
        dart_params = dart_args.join(", ")
    );
    quote!($source)
}
fn callback_names(ci: &ComponentInterface) -> Vec<String> {
    ci.callback_interface_definitions()
        .iter()
        .map(|c| c.name().to_owned())
        .chain(
            ci.object_definitions()
                .iter()
                .filter(|o| o.has_callback_interface())
                .map(|o| o.name().to_owned()),
        )
        .collect()
}
pub fn dart_runtime(ci: &ComponentInterface, config: &Config) -> dart::Tokens {
    let asset = format!("package:{}/{}_dispatch", config.package_name(), config.asset_id());
    let mut source = include_str!("dart.dart.txt").replace("@ASSET@", &asset);
    for name in callback_names(ci) {
        let cls = DartCodeOracle::class_name(&name);
        writeln!(source, "@Native<Uint64 Function(Uint64, Uint64)>(assetId: _uniffiDispatchAsset, symbol: 'register_{name}')\nexternal int _register{cls}(int endpoint, int handle);\n").unwrap();
        writeln!(
            source,
            "int _uniffiRegister{cls}(int handle) {{
  final endpoint = _uniffiDispatch.id;
  final native = _register{cls}(endpoint, handle);
  _uniffiDispatch.onLowered(() {{
    _uniffiDispatchUnregister(endpoint, native);
    FfiConverterCallbackInterface{cls}._handleMap.remove(handle);
  }});
  return native;
}}"
        )
        .unwrap();
    }
    quote!($source)
}
pub fn write_native(
    ci: &ComponentInterface,
    config: &Config,
    out: &camino::Utf8Path,
) -> Result<()> {
    let mut source = include_str!("runtime.rs.txt").to_owned();
    let mut callbacks = BTreeMap::new();
    let mut vtables = Vec::new();
    for def in ci.ffi_definitions() {
        match def {
            FfiDefinition::CallbackFunction(cb) => {
                let mut args: Vec<_> =
                    cb.arguments().iter().map(|a| native_type(&a.type_())).collect();
                if cb.has_rust_call_status_arg() {
                    args.push("*mut RustCallStatus".into());
                }
                writeln!(
                    source,
                    "type {} = unsafe extern \"C\" fn({}) -> {};",
                    cb.name(),
                    args.join(", "),
                    result_type(cb.return_type())
                )?;
                callbacks.insert(cb.name().to_owned(), cb);
            }
            FfiDefinition::Struct(st) => {
                writeln!(source, "#[repr(C)] #[derive(Clone, Copy)] pub struct {} {{", st.name())?;
                for field in st.fields() {
                    writeln!(source, "r#{}: {},", field.name(), native_type(&field.type_()))?;
                }
                writeln!(source, "}}")?;
                if st.name().starts_with("VTableCallbackInterface") {
                    vtables.push(st);
                }
            }
            _ => {}
        }
    }
    for table in &vtables {
        let name = table.name();
        let kind = name.strip_prefix("VTableCallbackInterface").unwrap();
        writeln!(source, "#[no_mangle] pub extern \"C\" fn register_{kind}(ep: u64, local: u64) -> u64 {{ register(ep, local, \"{kind}\") }}")?;
        for field in table.fields() {
            let FfiType::Callback(cbname) = field.type_() else {
                bail!("invalid callback vtable field")
            };
            let cb = &callbacks[&cbname];
            let mut args: Vec<_> = cb.arguments().iter().map(|a| native_type(&a.type_())).collect();
            if cb.has_rust_call_status_arg() {
                args.push("*mut RustCallStatus".into());
            }
            let params = args
                .iter()
                .enumerate()
                .map(|(i, t)| format!("a{i}: {t}"))
                .collect::<Vec<_>>()
                .join(", ");
            let tail = (1..args.len()).map(|i| format!("a{i}")).collect::<Vec<_>>().join(", ");
            let vals = (1..args.len()).map(|i| format!("a{i},")).collect::<Vec<_>>().join("");
            let fieldname = field.name();
            let ret = result_type(cb.return_type());
            writeln!(source, "unsafe extern \"C\" fn proxy_{kind}_{fieldname}({params}) -> {ret} {{\nlet r = route(a0, \"{kind}\"); let ep = endpoint(r.endpoint); let table: {name} = ep.vtable(\"{kind}\"); let input = Transfer(({vals}));\nlet result = dispatch(&ep, move || {{ let ({vals}) = input.take(); Transfer(unsafe {{ (table.r#{fieldname})(r.local{comma}{tail}) }}) }}).take();", comma=if tail.is_empty(){""}else{", "})?;
            if fieldname == "uniffi_clone" {
                writeln!(source, "register(r.endpoint, result, \"{kind}\")")?;
            } else if fieldname == "uniffi_free" {
                writeln!(source, "routes().lock().unwrap().remove(&a0).unwrap(); result")?;
            } else {
                writeln!(source, "result")?;
            }
            writeln!(source, "}}")?;
        }
        writeln!(source, "static PROXY_{kind}: {name} = {name} {{")?;
        for field in table.fields() {
            writeln!(source, "r#{}: proxy_{kind}_{},", field.name(), field.name())?;
        }
        writeln!(source, "}}; static INIT_{kind}: std::sync::Once = std::sync::Once::new();")?;
    }
    for fun in ci.iter_ffi_function_definitions().filter(is_adapter) {
        let name = fun.name();
        let ret = result_type(fun.return_type());
        let mut types: Vec<_> = fun.arguments().iter().map(|a| native_type(&a.type_())).collect();
        if fun.has_rust_call_status_arg() {
            types.push("*mut RustCallStatus".into());
        }
        let params = types
            .iter()
            .enumerate()
            .map(|(i, t)| format!(", a{i}: {t}"))
            .collect::<Vec<_>>()
            .join("");
        writeln!(source, "/// # Safety\n/// The original function and arguments must match this generated UniFFI ABI.\n/// Borrowed pointers must remain valid until this synchronous call returns.")?;
        writeln!(source, "#[no_mangle] pub unsafe extern \"C\" fn dispatch_{name}(ep: u64, original: unsafe extern \"C\" fn({}) -> {ret}{params}) -> {ret} {{", types.join(", "))?;
        if name.contains("_fn_init_callback_vtable_") {
            let FfiType::Reference(inner) = fun.arguments()[0].type_() else {
                bail!("invalid vtable init")
            };
            let FfiType::Struct(table) = *inner else { bail!("invalid vtable init") };
            let kind = table.strip_prefix("VTableCallbackInterface").unwrap();
            writeln!(source, "endpoint(ep).vtables.lock().unwrap().insert(\"{kind}\", Box::new(unsafe {{ *a0 }}));\nINIT_{kind}.call_once(|| unsafe {{ original(&PROXY_{kind}) }});")?;
        } else {
            let vals = (0..types.len()).map(|i| format!("a{i},")).collect::<Vec<_>>().join("");
            writeln!(source, "let input = Transfer(({vals}));\nrun(ep, move || {{ let ({vals}) = input.take(); Transfer(unsafe {{ original({vals}) }}) }}).take()")?;
        }
        writeln!(source, "}}")?;
    }
    let dir = out.join("uniffi_dispatch");
    std::fs::create_dir_all(dir.join("src"))?;
    let crate_name = format!("{}_dart_dispatch", ci.namespace());
    std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"uniffi_dart_dispatch\"\nversion = \"0.0.0\"\nedition = \"2021\"\nrust-version = \"1.85\"\npublish = false\n[lib]\ncrate-type = [\"cdylib\"]\n[workspace]\n".replace("uniffi_dart_dispatch", &crate_name))?;
    std::fs::write(dir.join("src/lib.rs"), source)?;
    std::fs::write(dir.join("README.md"), format!(
        "# Generated callback dispatcher\n\nBuild this crate for the same target as the original native library. It has no third-party dependencies.\n\nLibrary basename: `{crate_name}`.\nNative asset ID: `package:{}/{}_dispatch`.\n\nBundle this native asset alongside the original library. See docs/callback-dispatch.md in uniffi-dart for the opt-in contract and shutdown requirements.\n", config.package_name(), config.asset_id()
    ))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn component(body: &str) -> ComponentInterface {
        ComponentInterface::from_webidl(body, "dispatch_test").unwrap()
    }
    #[test]
    fn reject_async_and_callback_return_paths_before_generation() {
        for udl in [
            "namespace test { [Async] u32 work(); };",
            "namespace test {}; callback interface Sink { [Async] u32 work(); };",
            "namespace test {}; interface Object { constructor(); };",
            "namespace test { boolean close_callback_dispatcher(); };",
            "namespace test { Sink echo(Sink sink); }; callback interface Sink { u32 work(); };",
            "namespace test {}; callback interface Sink { u32 work(); }; dictionary Holder { Sink value; };",
        ] {
            assert!(validate(&component(udl)).is_err(), "accepted unsupported API: {udl}");
        }
    }
    #[test]
    fn accepts_sync_callback_inputs_but_rejects_incompatible_direct_wire_types() {
        validate(&component("namespace test { u32 work(Sink sink); }; callback interface Sink { u32 work(bytes data); };" )).unwrap();
        let ci = component("namespace test {}; callback interface Sink { duration work(); };");
        assert!(validate(&ci).unwrap_err().to_string().contains("wire type"));
    }
}
