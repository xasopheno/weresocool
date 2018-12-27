pub fn get_filepath_and_import_name(s: String) -> (String, String) {
    let s = s.replace("import", "");
    let filepath = get_filepath(s.clone());
    let import_name = get_import_name(s.clone());
    (filepath, import_name)
}

pub fn is_import(s: String) -> bool {
    s.trim().starts_with("import ")
}

pub fn is_as_import(s: String) -> bool {
    s.trim();
    s.contains(" as ")
}

fn get_import_name_from_path(s: String) -> String {
    let s = s.replace("import ", "");
    let split: Vec<&str> = s.split('/').collect();
    let result = split[split.len() - 1].to_string();
    result.trim().to_string()
}

pub fn get_filepath(s: String) -> String {
    let s = s.replace("import ", "");
    if !is_as_import(s.clone()) {
        s
    } else {
        let split: Vec<&str> = s.split('/').collect();
        let split_len = split.len();
        let mut path = split[0..split_len - 1].join("/").to_string();
        let filename_with_as = get_import_name_from_path(s.clone());
        let filename_vec: Vec<&str> = filename_with_as.split(" as ").collect();
        let filename = filename_vec[0].clone();
        path.push('/');
        path.push_str(&filename);
        path.trim().to_string()
    }
}

fn get_as_name(s: String) -> String {
    let split: Vec<&str> = s.split(" as ").collect();
    let result = split[split.len() - 1].to_string();
    result
}

pub fn get_import_name(s: String) -> String {
    let filename_or_as = get_import_name_from_path(s.to_string());
    if is_as_import(filename_or_as.clone()) {
        let filename = get_as_name(filename_or_as);
        filename
    } else {
        let filename = filename_or_as.replace(".socool", "");
        filename
    }
}


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
