use anyhow::Result;

#[test]
fn type_limits() -> Result<()> {
    uniffi_dart::testing::run_test("type_limits", "src/api.udl", None)
} 