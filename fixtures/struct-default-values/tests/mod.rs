use anyhow::Result;

#[test]
fn struct_default_values() -> Result<()> {
    uniffi_dart::testing::run_test("struct_default_values", "src/api.udl", None)
} 