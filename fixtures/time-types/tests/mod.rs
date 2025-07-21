#[cfg(test)]
mod tests {
    #[test]
    fn test_time_types() {
        uniffi_dart::testing::run_test("time_types", None).unwrap();
    }
} 