#[test]
fn test_time_types() -> anyhow::Result<()> {
    uniffi_dart::testing::run_test("time-types", "src/api.udl", None)
}
