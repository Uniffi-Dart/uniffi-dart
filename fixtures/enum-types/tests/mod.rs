use anyhow::Result;

#[test]
fn enum_types() -> Result<()> {
    uniffi_dart::testing::run_test("enum_types", "src/api.udl", None)
} 