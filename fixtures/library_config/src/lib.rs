// Minimal library-mode fixture. Its committed `uniffi.toml` sets a non-default
// `package_name`, so the bindgen must read that config (via the cargo-metadata
// crate config supplier) for the generated asset id to be correct — see the
// library-mode test in `tests/mod.rs`.

#[uniffi::export]
fn ping() -> String {
    "pong".to_string()
}

uniffi::include_scaffolding!("api");
