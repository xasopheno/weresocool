extern crate weresocool;

fn main() {
    println!("\nHello Danny's WereSoCool Scratch Pad");

    let string_list: Vec<String> = vec!["this", "thing"]
        .iter_mut()
        .map(|string| string.to_string())
        .collect();

    let joined = string_list.join(".");
    assert_eq!(joined, "this.thing".to_string());

    test_filepath();
}

pub fn is_as_import(s: String) -> bool {
    s.trim();
    s.contains(" as ")
}

fn get_filename_or_as_from_path(s: String) -> String {
    let split: Vec<&str> = s.split('/').collect();
    let result = split[split.len() - 1].to_string();
    result
}

fn get_filepath(s: String) -> String {
    if !is_as_import(s.clone()) {
        s
    } else {
        let split: Vec<&str> = s.split('/').collect();
        let split_len = split.len();
        let mut path = split[0..split_len - 1].join("/").to_string();
        let filename_with_as = get_filename_or_as_from_path(s.clone());
        let filename_vec: Vec<&str> = filename_with_as.split(" as ").collect();
        let filename = filename_vec[0].clone();
        path.push('/');
        path.push_str(&filename);
        path
    }
}

#[test]
fn test_filepath() {
    let tests = vec![
        "songs/wip/test.socool as test".to_string(),
        "../songs/test.socool".to_string(),
        "test.socool".to_string(),
    ];

    let result: Vec<String> = tests
        .iter()
        .map(|test| get_filepath(test.to_string()))
        .collect();

    println!("{:?}", result);

    assert_eq!(
        result,
        vec![
            "songs/wip/test.socool",
            "../songs/test.socool",
            "test.socool"
        ]
    )
}
