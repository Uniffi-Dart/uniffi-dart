//! The `uniffi_bindgen_dart` command-line interface, exposed as a library entry
//! point ([`main`]) so downstream `uniffi-bindgen.rs` helpers can forward argv to
//! it — the same convention as `uniffi::uniffi_bindgen_main()` and
//! `uniffi_bindgen_cs::main()` — instead of calling the generator function with
//! positional arguments (which breaks whenever a new option is added).

use std::process::exit;

use camino::Utf8PathBuf;

use crate::gen::{generate_dart_bindings_with_options, DartBindgenOptions};

const USAGE: &str = "\
uniffi_bindgen_dart — generate Dart bindings from a compiled UniFFI library

USAGE:
    uniffi_bindgen_dart [OPTIONS] <SOURCE>

ARGS:
    <SOURCE>          Path to the compiled cdylib/dylib (.so/.dylib/.dll) whose
                      embedded UniFFI metadata is read (library mode). This is how
                      `setup_scaffolding!()` (proc-macro, no UDL) crates are consumed.

OPTIONS:
    --library         Treat <SOURCE> as a compiled library. Accepted for parity
                      with `uniffi-bindgen`; library mode is the only mode this
                      tool supports, so the flag is optional.
    --out-dir <DIR>   Directory the generated `.dart` binding is written into.
    --config <TOML>   Optional `uniffi.toml` whose values are merged into each
                      crate's config (its values take precedence).
    --crate <NAME>    Generate bindings for only this crate, when the library
                      bundles more than one UniFFI component.
    --no-format       Skip running `dart format` on the generated output.
    --version         Print the version and exit.
    -h, --help        Print this help.

Runs `cargo metadata` in the current directory to resolve each crate's
`uniffi.toml`/UDL, so it must be run inside the Cargo workspace.
";

/// Parse process arguments and generate Dart bindings — the CLI entry point.
///
/// Downstream build helpers should forward argv here (`uniffi_dart::main()`)
/// rather than calling [`crate::gen::generate_dart_bindings`] with positional
/// args. Argument errors print usage and exit the process; generation errors are
/// returned to the caller.
pub fn main() -> anyhow::Result<()> {
    let mut source: Option<Utf8PathBuf> = None;
    let mut out_dir: Option<Utf8PathBuf> = None;
    let mut config: Option<Utf8PathBuf> = None;
    let mut crate_name: Option<String> = None;
    let mut try_format = true;

    let mut args = std::env::args().skip(1);
    while let Some(raw) = args.next() {
        // Accept both `--opt value` and `--opt=value`, matching the clap-based
        // uniffi-bindgen / -cs / -go. Only split a leading `--flag=…`; a positional
        // path that happens to contain `=` is left intact.
        let (arg, inline): (String, Option<String>) = match raw.split_once('=') {
            Some((flag, value)) if flag.starts_with("--") => {
                (flag.to_string(), Some(value.to_string()))
            }
            _ => (raw, None),
        };
        // Take an option's value: the attached `=value` if present, otherwise the
        // next argument — rejected if missing or itself an option (so
        // `--out-dir --config x` errors instead of silently using `--config`).
        let mut take = |name: &str| {
            if let Some(value) = &inline {
                return value.clone();
            }
            match args.next() {
                Some(v) if !v.starts_with("--") => v,
                _ => {
                    eprintln!("error: `{name}` requires a value\n\n{USAGE}");
                    exit(2);
                }
            }
        };
        // A boolean flag rejects an attached `=value` rather than ignoring it.
        let reject_inline = |name: &str| {
            if inline.is_some() {
                eprintln!("error: `{name}` takes no value\n\n{USAGE}");
                exit(2);
            }
        };
        // Reject a repeated path option rather than silently keeping the last value.
        let set_once = |slot: &mut Option<Utf8PathBuf>, name: &str, value: String| {
            if slot.is_some() {
                eprintln!("error: `{name}` given more than once\n\n{USAGE}");
                exit(2);
            }
            *slot = Some(Utf8PathBuf::from(value));
        };
        match arg.as_str() {
            "--library" => reject_inline("--library"), // implied; accepted for compatibility
            "--out-dir" => {
                let v = take("--out-dir");
                set_once(&mut out_dir, "--out-dir", v);
            }
            "--config" => {
                let v = take("--config");
                set_once(&mut config, "--config", v);
            }
            "--crate" => {
                if crate_name.is_some() {
                    eprintln!("error: `--crate` given more than once\n\n{USAGE}");
                    exit(2);
                }
                crate_name = Some(take("--crate"));
            }
            "--no-format" => {
                reject_inline("--no-format");
                try_format = false;
            }
            "--version" => {
                reject_inline("--version");
                println!("uniffi_bindgen_dart {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "-h" | "--help" => {
                print!("{USAGE}");
                return Ok(());
            }
            other if other.starts_with('-') => {
                eprintln!("error: unknown option `{other}`\n\n{USAGE}");
                exit(2);
            }
            _ => {
                if source.is_some() {
                    eprintln!("error: more than one source path given\n\n{USAGE}");
                    exit(2);
                }
                source = Some(Utf8PathBuf::from(arg));
            }
        }
    }

    let (Some(source), Some(out_dir)) = (source, out_dir) else {
        eprintln!("error: a source library path and --out-dir are required\n\n{USAGE}");
        exit(2);
    };

    generate_dart_bindings_with_options(
        // In library mode the UDL is resolved per-crate from cargo metadata, so the
        // `udl_file` argument is unused — pass the library path as a harmless stand-in.
        source.as_path(),
        config.as_deref(),
        Some(out_dir.as_path()),
        source.as_path(),
        true,
        &DartBindgenOptions { crate_name, try_format },
    )?;

    eprintln!("Generated Dart bindings from {source} into {out_dir}");
    Ok(())
}
