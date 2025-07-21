#[cfg(test)]
mod tests {
    #[test]
    fn test_docstring_proc_macro() {
        uniffi_dart::testing::run_test("docstring_proc_macro", None).unwrap();
    }
} 