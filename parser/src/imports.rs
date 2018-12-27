pub fn is_import(s: String) -> bool {
    s.trim().starts_with("import")
}

fn get_filename_or_as_from_path(s: String) -> String {
    let split: Vec<&str> = s.split('/').collect();
    let result = split[split.len() - 1].to_string();
    result
}

//fn get_file_path_from_import(s: String) -> String {
//
//}

pub fn is_as_import(s: String) -> bool {
    s.trim();
    s.contains(" as ")
}

fn get_as_name(s: String) -> String {
    let split: Vec<&str> = s.split(" as ").collect();
    let result = split[split.len() - 1].to_string();
    result
}

pub fn get_import_name(s: String) -> String {
    let filename_or_as = get_filename_or_as_from_path(s.to_string());
    if is_as_import(filename_or_as.clone()) {
        let filename = get_as_name(filename_or_as);
        filename
    } else {
        let filename = filename_or_as.replace(".socool", "");
        filename
    }
}

 #[test]
fn test_filename() {
    let tests = vec![
        "songs/wip/test.socool as test".to_string(),
        "../songs/test.socool".to_string(),
        "test.socool".to_string(),
    ];

    let result: Vec<String> = tests
        .iter()
        .map(|test| {
            get_import_name(test.to_string())
        })
        .collect();
    println!("{:?}", result);
    assert_eq!(result, vec!["test", "test", "test",])
}

#[test]
fn test_data() -> Vec<String> {
    let import_str = "   import standard ".to_string();
    let import_as_str = "import  standard as std".to_string();
    let not_import_as_str = "import standardasstd  ".to_string();
    let not_import = "not an import".to_string();
    vec![import_str, import_as_str, not_import_as_str, not_import]
}

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
