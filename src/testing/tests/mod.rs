#[cfg(test)]
mod expect_tests {
    use crate::testing::expect::expect_eq;

    #[test]
    fn test_function() {
        let _should_match = expect_eq(
            "src/testing/mocks/simple_fun.socool",
            "src/testing/mocks/simple.socool",
        );
    }

    #[test]
    fn test_function_overlay() {
        let _should_match = expect_eq(
            "src/testing/mocks/fun_nested.socool",
            "src/testing/mocks/fun_nested_expected.socool",
        );
    }
}
