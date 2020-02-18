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
        expect("src/testing/mocks/list_with_indicies.socool");
        expect("src/testing/mocks/list_named_with_indicies.socool");
        expect("src/testing/mocks/list_composed_with_op.socool");
        expect("src/testing/mocks/list_fit_length.socool");
        expect("src/testing/mocks/list_named_indexed_fit_length.socool");
        expect("src/testing/mocks/list_indexed_fit_length.socool");
        expect("src/testing/mocks/et_list.socool");
        expect("src/testing/mocks/list_with_random.socool");
    }
}
