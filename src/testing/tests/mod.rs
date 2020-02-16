#[cfg(test)]
mod expect_tests {
    use crate::testing::expect::expect;

    #[test]
    fn test_functions() {
        expect("src/testing/mocks/simple_fun.socool");
        expect("src/testing/mocks/fun_nested.socool");
    }

    #[test]
    fn test_lists() {
        expect("src/testing/mocks/simple_list.socool");
        expect("src/testing/mocks/list_id.socool");
    }
}
