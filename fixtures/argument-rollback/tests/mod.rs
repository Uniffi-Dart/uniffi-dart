#[test]
fn argument_rollback() -> anyhow::Result<()> {
    uniffi_dart::testing::run_test("argument_rollback", "src/api.udl", None)
}
