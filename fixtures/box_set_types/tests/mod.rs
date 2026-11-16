use anyhow::Result;

#[test]
fn box_set_types() -> Result<()> {
    uniffi_dart::testing::run_test("box_set_types", "src/api.udl", None)
}
