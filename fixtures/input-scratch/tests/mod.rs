#[test]
fn input_scratch() -> anyhow::Result<()> {
    uniffi_dart::testing::run_test("input_scratch", "src/api.udl", None)
}
