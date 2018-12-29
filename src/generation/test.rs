mod parse_tests {
    extern crate socool_parser;
    use socool_parser::parser::parse_file;
    #[test]
    fn import_test() {
        let filename = &"songs/test/import_test.socool".to_string();
        let parsed = parse_file(filename, None);
        let mut result: Vec<String> = parsed
            .table
            .iter()
            .map(|(name, _)| name.to_string())
            .collect();

        result.sort();

        let mut expected: Vec<String> = vec![
            "danny",
            "import_test_2.main",
            "import_test_2.root_fifth",
            "import_test_2.std_test.fade_out",
            "import_test_2.std_test.main",
            "main",
            "standard.fade_out",
            "standard.import_test_2.main",
            "standard.import_test_2.root_fifth",
            "standard.import_test_2.std_test.fade_out",
            "standard.import_test_2.std_test.main",
            "standard.main",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        expected.sort();

        assert_eq!(expected, result)
    }
}
