#[cfg(test)]
mod tests {
    #[test]
    fn test_proc_macro_no_implicit_prelude() {
        uniffi_dart::testing::run_test("proc_macro_no_implicit_prelude", None).unwrap();
    }
} 