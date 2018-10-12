use std::collections::HashMap;
#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub socool); // synthesized by LALRPOP
pub mod ast;
use crate::ast::*;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let mut table = make_table();

    let filename = "working.socool";

    let mut f = File::open(filename).expect("file not found");

    let mut composition = String::new();
    f.read_to_string(&mut composition)
        .expect("something went wrong reading the file");

    println!(
        "\n Settings: {:?}",
        socool::SoCoolParser::new()
            .parse(&mut table, &composition)
            .unwrap()
    );

    for (key, val) in table.iter() {
        println!("\n name: {:?} op: {:?}", key, val);
    }

    println!("\n Main: {:?}", table.get("main").unwrap());
}

fn make_table() -> HashMap<String, Op> {
    let table: HashMap<String, Op> = HashMap::new();
    table
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Init, Op};
    lalrpop_mod!(pub socool);

    fn mock_init() -> (String) {
        "{ f: 200, l: 1.0, g: 1.0, p: 0.0 }
        let main = {"
            .to_string()
    }

    fn test_parsed_operation(mut parse_str: String, expected: Op) {
        let mut table = make_table();

        parse_str.push_str("}");

        let _result = socool::SoCoolParser::new().parse(&mut table, &parse_str);

        let main = table.get(&"main".to_string()).unwrap();
        assert_eq!(*main, expected);
    }

    #[test]
    fn tm_test() {
        let mut parse_str = mock_init();
        parse_str.push_str("Tm 3/2");
        test_parsed_operation(parse_str, Op::TransposeM { m: 1.5 });
    }

    #[test]
    fn ta_test() {
        let mut parse_str = mock_init();
        parse_str.push_str("Ta 2.0");
        test_parsed_operation(parse_str, Op::TransposeA { a: 2.0 });
    }

    #[test]
    fn pan_a_test() {
        let mut parse_str = mock_init();
        parse_str.push_str("PanA 2.0");
        test_parsed_operation(parse_str, Op::PanA { a: 2.0 });
    }

    #[test]
    fn pan_m_test() {
        let mut parse_str = mock_init();
        parse_str.push_str("PanM 3.0/2.0");
        test_parsed_operation(parse_str, Op::PanM { m: 1.5 });
    }

    #[test]
    fn gain_test() {
        let mut parse_str = mock_init();
        parse_str.push_str("Gain 0.25");
        test_parsed_operation(parse_str, Op::Gain { m: 0.25 });
    }

    #[test]
    fn length_test() {
        let mut parse_str = mock_init();
        parse_str.push_str("Length 0.5");
        test_parsed_operation(parse_str, Op::Length { m: 0.5 });
    }

    #[test]
    fn reverse_test() {
        let mut parse_str = mock_init();
        parse_str.push_str("Reverse");
        test_parsed_operation(parse_str, Op::Reverse);
    }

    #[test]
    fn asis_test() {
        let mut parse_str = mock_init();
        parse_str.push_str("AsIs");
        test_parsed_operation(parse_str, Op::AsIs);
    }

    #[test]
    fn sequence_test() {
        let mut parse_str = mock_init();
        parse_str.push_str(
            "
            Sequence [
                AsIs,
                Tm 3/2,
            ]
        ",
        );
        test_parsed_operation(
            parse_str,
            Op::Sequence {
                operations: vec![Op::AsIs, Op::TransposeM { m: 3.0 / 2.0 }],
            },
        );
    }

    #[test]
    fn overlay_test() {
        let mut parse_str = mock_init();
        parse_str.push_str(
            "
            Overlay [
                AsIs,
                Tm 3/2,
            ]
        ",
        );
        test_parsed_operation(
            parse_str,
            Op::Overlay {
                operations: vec![Op::AsIs, Op::TransposeM { m: 3.0 / 2.0 }],
            },
        );
    }

    //        assert!(
    //            socool::SoCoolParser::new()
    //                .parse(&mut table,
    //                       "o[(3/2, 3.0, 1.0, 0.0),
    //                       (3/2, 0.0, 1.0, 0.0),
    //                       (1, 0.0, 1.0, 0.0)]"
    //                )
    //                .is_ok()
    //        )
    //    }
    //    #[test]
    //    fn let_insert() {
    //        let mut table = make_table();
    //        socool::SoCoolParser::new().parse(
    //            &mut table,
    //            "let thing =
    //                Tm 3/2
    //                | Gain 0.3
    //            ").unwrap();
    //        assert_eq!(
    //            table[thing],
    //            Let { name: "thing".to_string(), operation: Op::Compose { operations: vec![Op::TransposeM { m: 1.5 }, Op::Gain { m: 0.3 }] } }
    //        )
    //    }
    //    #[test]
    //    fn let_get() {
    //        let mut table = make_table();
    //        socool::SoCoolParser::new().parse(
    //            &mut table,
    //            "let thing =
    //                    Tm 3/2
    //                    | Gain 0.3
    //
    //            Sequence[
    //                thing
    //            ]
    //            ").unwrap();
    //        assert_eq!(
    //            table[thing],
    //            Let { name: "thing".to_string(), operation: Op::Compose { operations: vec![Op::TransposeM { m: 1.5 }, Op::Gain { m: 0.3 }] } }
    //        )
    //    }
    //
    //    #[test]
    //    fn fit_length_test() {
    //        let mut table = make_table();
    //        let result = socool::SoCoolParser::new().parse(
    //            &mut table,
    //            "
    //                { f: 200, l: 1.0, g: 1.0, p: 0.0 }
    //
    //                let thing = {
    //                    Sequence [
    //                     Tm 3/2
    //                    ]
    //                }
    //                let thing2 = {
    //                    Tm 5/4
    //                    | Repeat 5
    //                    > fitLength thing
    //                }
    //                let main = {
    //                    Overlay [
    //                        Sequence[
    //                            thing,
    //                            thing2
    //                        ]
    //                        > fitLength thing
    //                    ]
    //                }
    //            ");
    //        assert!(
    //          table.len() == 0;
    //        );
}
