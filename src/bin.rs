//! Thin binary wrapper. The CLI implementation lives in `uniffi_dart::main`
//! (see `src/cli.rs`) so downstream `uniffi-bindgen.rs` helpers can forward argv
//! to the same entry point as a library call, rather than invoking the generator
//! function with positional arguments.

fn main() {
    if let Err(e) = uniffi_dart::main() {
        eprintln!("error: failed to generate Dart bindings: {e:?}");
        std::process::exit(1);
    }
}
