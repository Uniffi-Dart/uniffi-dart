#[cfg(test)]
mod tests {
    #[test]
    fn test_docstring() {
        uniffi_dart::testing::run_test("docstring", None).unwrap();
    }
} 