use anyhow::Result;

#[test]
fn map_type() -> Result<()> {
    uniffi_dart::testing::run_test("map_type", "src/api.udl", None)
}
