use anyhow::Result;

#[test]
fn proc_macro() -> Result<()> {
    uniffi_dart::testing::run_test("proc_macro", "src/api.udl", None)
} 