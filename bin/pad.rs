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

    let mut filepath = "../songs/wip/test.socool".to_string();
    let stuff: Vec<&str> = filepath
        .split('/')
        .collect();

    println!("{:?}", stuff);

//    unsafe {
//        let filename = filename.as_mut_vec();
//    }

//    filename.trim
}
