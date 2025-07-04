use anyhow::Result;

#[test]
fn test_external_types_provider() -> Result<()> {
    uniffi_dart::testing::run_test("external_types_provider", "src/lib.udl", None)
} 