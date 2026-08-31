use anyhow::Result;

#[test]
fn enum_variant_collision() -> Result<()> {
    uniffi_dart::testing::run_test("enum_variant_collision", "src/api.udl", None)
}
