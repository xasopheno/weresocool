use weresocool_parser::parser::*;
use weresocool_ast::{Defs, Op, Term};

fn main() {
    let parse_str = r#"{ f: 200, l: 1.0, g: 1.0, p: 0.0 }
main = {
  Choose[
    Tm 1/1,
    Tm 5/4,
    Tm 3/2,
    Tm 2/1
  ] | Repeat 4
}"#;

    let mut defs: Defs = Default::default();

    match socool::SoCoolParser::new().parse(&mut defs, parse_str) {
        Ok(_init) => {
            let main = defs.ops.get("main").unwrap();
            println!("Parsed main:");
            println!("{:#?}", main);
        }
        Err(e) => {
            println!("Parse error: {:?}", e);
        }
    }
}
