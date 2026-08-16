use std::process::exit;

use camino::Utf8PathBuf;

use uniffi_dart::gen::generate_dart_bindings;

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

fn main() {
    let mut source: Option<Utf8PathBuf> = None;
    let mut out_dir: Option<Utf8PathBuf> = None;
    let mut config: Option<Utf8PathBuf> = None;
    let mut crate_name: Option<String> = None;
    let mut try_format = true;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        // Take the value for an option, rejecting a missing value or one that is
        // itself an option (so `--out-dir --config x` errors instead of silently
        // using `--config` as the out-dir).
        let mut take = |name: &str| match args.next() {
            Some(v) if !v.starts_with("--") => v,
            _ => {
                eprintln!("error: `{name}` requires a value\n\n{USAGE}");
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
            "--library" => { /* library mode is implied; accepted for compatibility */ }
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
            "--no-format" => try_format = false,
            "--version" => {
                println!("uniffi_bindgen_dart {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "-h" | "--help" => {
                print!("{USAGE}");
                return;
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

    if let Err(e) = generate_dart_bindings(
        // In library mode the UDL is resolved per-crate from cargo metadata, so the
        // `udl_file` argument is unused — pass the library path as a harmless stand-in.
        source.as_path(),
        config.as_deref(),
        Some(out_dir.as_path()),
        source.as_path(),
        true,
        crate_name,
        try_format,
    ) {
        eprintln!("error: failed to generate Dart bindings: {e:?}");
        exit(1);
    }

    eprintln!("Generated Dart bindings from {source} into {out_dir}");
}
