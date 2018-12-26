pub fn is_import(s: String) -> bool {
    s.trim().starts_with("import")
}

pub fn is_as_import(s: String) -> bool {
    s.trim();
    s.contains(" as ")
}

fn test_data() -> Vec<String> {
    let import_str = "   import standard ".to_string();
    let import_as_str = "import  standard as std".to_string();
    let not_import_as_str = "import standardasstd  ".to_string();
    let not_import = "not an import".to_string();
    vec![import_str, import_as_str, not_import_as_str, not_import]
}

//fn get_import_name(s: String) {
//
//}

#[test]
fn test_import_strings() {
    let lines = test_data();
    let starts_with_import: Vec<bool> = lines
        .iter()
        .map(|line| is_import(line.to_string()))
        .collect();

    let is_as_import: Vec<bool> = lines
        .iter()
        .map(|line| is_as_import(line.to_string()))
        .collect();

    assert_eq!(starts_with_import, vec![true, true, true, false]);
    assert_eq!(is_as_import, vec![false, true, false, false]);
}

#[test]
fn test_import_strings2() {
    let lines = test_data();
    let w_o_import: Vec<String> = lines
        .iter()
        .filter(|line| is_import(line.to_string()))
        .map(|line| line.replace("import ", ""))
        .collect();

    let trimmed: Vec<String> = w_o_import
        .iter()
        .map(|line| line.trim().to_string())
        .collect();

    assert_eq!(
        trimmed,
        vec!["standard", "standard as std", "standardasstd"]
    )
}
