extern crate weresocool;
use std::collections::HashMap;

fn main() {
    println!("\nHello Danny's WereSoCool Scratch Pad");

    let mut hm: HashMap<String, usize> = HashMap::new();
    hm.insert("this.thing".to_string(), 1);
    println!("{:#?}", hm);
    let result = hm.get(&"this.thing".to_string());
    println!("{:#?}", result);
    assert_eq!(result, Some(&1));
    let nope = hm.get(&"this".to_string());
    println!("{:?}", nope);
    assert_eq!(nope, None);

    let string_list: Vec<String> = vec!["this", "thing"]
        .iter_mut()
        .map(|string| string.to_string())
        .collect();

    let joined = string_list.join(".");
    println!("{:#?}", joined);
    assert_eq!(joined, "this.thing".to_string());

    let import_str = "   import standard ".to_string();
    let import_as_str = "import  standard as std".to_string();
    let not_import_as_str = "import standardasstd  ".to_string();
    let not_import = "not an import".to_string();
    let lines = vec![import_str, import_as_str, not_import_as_str, not_import];

    fn is_import(s: String) -> bool {
       s.trim().starts_with("import")
    };

    fn is_as_import(s: String) -> bool {
        s.trim();
        s.contains(" as ")
    }

    #[test]
    fn test_import_strings() {
        let starts_with_import: Vec<bool> =
            lines
                .iter()
                .map(|line| is_import(line.to_string()))
                .collect();

        let is_as_import: Vec<bool> =
            lines
                .iter()
                .map(|line| is_as_import(line.to_string()))
                .collect();

        assert_eq!(starts_with_import, vec![true, true, true, false]);
        assert_eq!(is_as_import, vec![false, true, false, false]);
    }

    let w_o_import: Vec<String> =
        lines
            .iter()
            .filter(|line| is_import(line.to_string()))
            .map(|line| {
                line
                    .replace("import ", "")
            })
            .collect();

    let trimmed: Vec<String> = w_o_import
        .iter()
        .map(|line| line.trim().to_string())
        .collect();
    println!("{:#?}", trimmed);




}
