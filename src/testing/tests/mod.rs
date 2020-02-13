use crate::testing::expect::expect_eq;

#[cfg(test)]
mod expect_tests {
    use super::*;

    #[test]
    fn test_expect_function() {
        let _should_match = expect_eq(
            "src/testing/mocks/simple_fun.socool",
            "src/testing/mocks/simple.socool",
        );
    }
}
