use anyhow::Result;

/// Library mode must read the crate's committed `uniffi.toml` (no `--config`
/// given), so the generated asset id uses the custom `package_name` rather than
/// the default `uniffi`. Guards the CrateConfigSupplier fix — the only in-repo
/// test that exercises the library-mode generation path.
#[test]
fn library_mode_reads_uniffi_toml() -> Result<()> {
    uniffi_dart::testing::assert_library_mode_asset_id("library_config", "custom_lib_pkg")
}
