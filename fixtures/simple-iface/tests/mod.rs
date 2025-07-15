use anyhow::Result;

#[test]
fn simple_iface() -> Result<()> {
    uniffi_dart::testing::run_test("simple_iface", "src/api.udl", None)
} 