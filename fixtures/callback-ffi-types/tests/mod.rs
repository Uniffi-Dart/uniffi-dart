#[test]
fn callback_ffi_types() -> anyhow::Result<()> {
    uniffi_dart::testing::run_test("callback_ffi_types", "src/api.udl", None)
}
