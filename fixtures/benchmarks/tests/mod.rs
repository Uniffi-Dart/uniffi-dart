#[cfg(test)]
mod tests {
    #[test]
    fn test_benchmarks() {
        uniffi_dart::testing::run_test("benchmarks", None).unwrap();
    }
} 