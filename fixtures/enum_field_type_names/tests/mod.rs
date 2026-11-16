use anyhow::Result;

#[test]
fn enum_field_type_names() -> Result<()> {
    uniffi_dart::testing::run_test("enum_field_type_names", "src/api.udl", None)
}
