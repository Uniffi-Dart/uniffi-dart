#[test]
fn incoming_buffers() -> anyhow::Result<()> {
    uniffi_dart::testing::run_test("incoming_buffers", "src/api.udl", None)
}
