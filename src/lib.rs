#[cfg(feature = "build")]
mod build;
#[cfg(feature = "bindgen-tests")]
pub mod testing;
#[cfg(feature = "build")]
pub use build::generate_scaffolding;

pub mod gen;

// The bindgen CLI, exposed as `uniffi_dart::main()` so downstream
// `uniffi-bindgen.rs` helpers can forward argv to it (the same convention as
// `uniffi::uniffi_bindgen_main()` / `uniffi_bindgen_cs::main()`), instead of
// calling the generator with positional args.
#[cfg(feature = "binary")]
mod cli;
#[cfg(feature = "binary")]
pub use cli::main;

pub use uniffi_dart_macro::*;
