extern crate weresocool;

fn main() {
    println!("\nHello Danny's WereSoCool Scratch Pad");

    let string_list: Vec<String> = vec!["this", "thing"]
        .iter_mut()
        .map(|string| string.to_string())
        .collect();

    let joined = string_list.join(".");
    println!("{:#?}", joined);
    assert_eq!(joined, "this.thing".to_string());
}
