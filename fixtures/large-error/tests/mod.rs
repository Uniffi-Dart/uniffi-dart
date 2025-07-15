use anyhow::Result;

#[test]
fn large_error() -> Result<()> {
    uniffi_dart::testing::run_test("large_error", "src/api.udl", None)
} 