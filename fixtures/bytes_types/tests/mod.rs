use anyhow::Result;

#[test]
fn bytes_types() -> Result<()> {
    uniffi_dart::testing::run_test("bytes_types", "src/api.udl", None)
}
