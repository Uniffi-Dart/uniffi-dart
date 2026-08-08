use std::process::exit;

use camino::Utf8PathBuf;

use uniffi_dart::gen::generate_dart_bindings;

const USAGE: &str = "\
uniffi_bindgen_dart — generate Dart bindings from a compiled UniFFI library

USAGE:
    uniffi_bindgen_dart --library <LIB> --out-dir <DIR> [--config <TOML>] [--udl <UDL>]

OPTIONS:
    --library <LIB>   Path to the compiled cdylib/dylib (.so/.dylib/.dll) whose
                      embedded UniFFI metadata is read (library mode). This is how
                      `setup_scaffolding!()` (proc-macro, no UDL) crates are consumed.
    --out-dir <DIR>   Directory the generated `.dart` binding is written into.
    --config <TOML>   Optional `uniffi.toml` config file.
    --udl <UDL>       Optional UDL path; only consulted for UDL-based crates,
                      and ignored when --config is given.
    -h, --help        Print this help.
";

fn main() {
    let mut library: Option<Utf8PathBuf> = None;
    let mut out_dir: Option<Utf8PathBuf> = None;
    let mut config: Option<Utf8PathBuf> = None;
    let mut udl: Option<Utf8PathBuf> = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut take = |name: &str| {
            args.next().map(Utf8PathBuf::from).unwrap_or_else(|| {
                eprintln!("error: `{name}` requires a value\n\n{USAGE}");
                exit(2);
            })
        };
        match arg.as_str() {
            "--library" => library = Some(take("--library")),
            "--out-dir" => out_dir = Some(take("--out-dir")),
            "--config" => config = Some(take("--config")),
            "--udl" => udl = Some(take("--udl")),
            "-h" | "--help" => {
                print!("{USAGE}");
                return;
            }
            other => {
                eprintln!("error: unknown argument `{other}`\n\n{USAGE}");
                exit(2);
            }
        }
    }

    let (Some(library), Some(out_dir)) = (library, out_dir) else {
        eprintln!("error: --library and --out-dir are required\n\n{USAGE}");
        exit(2);
    };

    // In library mode the interface is read from the compiled library's embedded
    // metadata; a UDL path is only consulted for UDL-based crates. Default it to
    // the library path (never read for proc-macro crates) so pure
    // `setup_scaffolding!()` crates need no `--udl`.
    let udl = udl.unwrap_or_else(|| library.clone());

    if let Err(e) = generate_dart_bindings(
        udl.as_path(),
        config.as_deref(),
        Some(out_dir.as_path()),
        library.as_path(),
        true,
    ) {
        eprintln!("error: failed to generate Dart bindings: {e:?}");
        exit(1);
    }

    eprintln!("Generated Dart bindings from {library} into {out_dir}");
}
