use std::collections::HashMap;
#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub socool); // synthesized by LALRPOP
pub mod ast;
use crate::ast::*;

fn main() {
    let mut table = make_table();
    println!("{:?}", socool::SoCoolParser::new().parse(
        &mut table,
        "let thing =
            Tm 3/2
            | Gain 0.3
        "));
    println!("{:?}", table);

//    println!(
//        "{:?}",
//        socool::SoCoolParser::new().parse(
//        "o[(3/2, 3.0, 1.0, 0.0),
//           (3/2, 0.0, 1.0, 0.0),
//           (1, 0.0, 1.0, 0.0)]"
//        ).unwrap()
//    );
//    println!("{:?}", socool::SoCoolParser::new().parse(
//        "Tm 3/2
//        | Gain 0.5
//        | Length 0.5
//        "
//    ).unwrap());
//    println!("{:?}", socool::SoCoolParser::new().parse(
//        "Tm 3/2"
//    ).unwrap());
}

fn make_table() -> HashMap<String, Let> {
    let table: HashMap<String, Let> = HashMap::new();
    table
}

#[cfg(test)]
mod tests {
    use crate::ast::{Op, Let};
    use super::*;
    lalrpop_mod!(pub socool); // synthesized by LALRPOP
    #[test]
    fn ops() {
        let mut table = make_table();
        assert_eq!(
            socool::SoCoolParser::new().parse(&mut table, "  Tm 3/2").unwrap(),
            Op::TransposeM { m: 1.5 }
        );
        assert_eq!(
            socool::SoCoolParser::new().parse(&mut table, "Ta 3.0").unwrap(),
            Op::TransposeA { a: 3.0 }
        );
        assert_eq!(
            socool::SoCoolParser::new().parse(&mut table, "PanM   3.0").unwrap(),
            Op::PanM { m: 3.0 }
        );
        assert_eq!(
            socool::SoCoolParser::new().parse(&mut table, "PanA 3.0").unwrap(),
            Op::PanA { a: 3.0 }
        );
        assert_eq!(
            socool::SoCoolParser::new().parse(&mut table, "Gain 3.0").unwrap(),
            Op::Gain { m: 3.0 }
        );
        assert_eq!(
            socool::SoCoolParser::new().parse(&mut table, "Length 3.0").unwrap(),
            Op::Length { m: 3.0 }
        );
        assert_eq!(
            socool::SoCoolParser::new().parse(&mut table, "Reverse").unwrap(),
            Op::Reverse
        );
        assert_eq!(
            socool::SoCoolParser::new().parse(&mut table, "AsIs").unwrap(),
            Op::AsIs
        );
        assert_eq!(
            socool::SoCoolParser::new()
                .parse(&mut table, "
                Sequence [
                    AsIs,
                    Tm 3/2,
                ]
                ")
                .unwrap(),
            Op::Sequence {
                operations: vec![Op::AsIs, Op::TransposeM { m: 3.0 / 2.0 }]
            }
        );
        assert_eq!(
            socool::SoCoolParser::new()
                .parse(&mut table, "
                Overlay [
                    AsIs,
                    Tm 3/2,
                ]
                ")
                .unwrap(),
            Op::Overlay {
                operations: vec![Op::AsIs, Op::TransposeM { m: 3.0 / 2.0 }]
            }
        );
        assert!(
            socool::SoCoolParser::new()
                .parse(&mut table,
                       "o[(3/2, 3.0, 1.0, 0.0),
                       (3/2, 0.0, 1.0, 0.0),
                       (1, 0.0, 1.0, 0.0)]"
                )
                .is_ok()
        )
    }
    #[test]
    fn let_test() {
        let mut table = make_table();
        socool::SoCoolParser::new().parse(
            &mut table,
            "let thing =
                Tm 3/2
                | Gain 0.3
            ").unwrap();
        assert_eq!(
            table[0],
            Let { name: "thing".to_string(), operation: Op::Compose { operations: vec![Op::TransposeM { m: 1.5 }, Op::Gain { m: 0.3 }] } }
        )
    }
}
