#[cfg(test)]
mod expect_tests {
    use crate::testing::expect::expect;
    use test_generator::test_resources;

    #[test_resources("src/testing/mocks/*.socool")]
    fn expect_test(resource: &str) {
        expect(resource);
    }
}
