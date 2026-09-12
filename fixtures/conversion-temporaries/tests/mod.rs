#[test]
fn conversion_temporaries() -> anyhow::Result<()> {
    uniffi_dart::testing::run_test("conversion_temporaries", "src/api.udl", None)
}
