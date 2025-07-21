use anyhow::Result;

#[test]
fn simple_fns() -> Result<()> {
    uniffi_dart::testing::run_test("simple_fns", "src/api.udl", None)
} 