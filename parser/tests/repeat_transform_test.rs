use weresocool_parser::parser::*;
use weresocool_ast::{Defs, Op, Term};

#[test]
fn test_repeat_transformation() {
    let parse_str = r#"{ f: 200, l: 1.0, g: 1.0, p: 0.0 }
main = {
  Tm 1/1 | Repeat 4
}"#;

    let mut defs: Defs = Default::default();

    let _init = socool::SoCoolParser::new().parse(&mut defs, parse_str).unwrap();

    let main = defs.ops.get("main").unwrap();

    // The transformation should convert Compose[Tm 1/1, Sequence[AsIs×4]] to Repeat{[Tm 1/1], 4}
    match main {
        Term::Op(Op::Compose { operations }) => {
            println!("Compose operations: {:#?}", operations);

            // After transformation, there should be one Repeat operation
            assert_eq!(operations.len(), 1, "Expected one Repeat operation after transformation");

            match &operations[0] {
                Term::Op(Op::Repeat { operations: repeat_ops, count }) => {
                    println!("Found Repeat with {} operations and count {}", repeat_ops.len(), count);
                    assert_eq!(*count, 4, "Expected Repeat count to be 4");
                    assert_eq!(repeat_ops.len(), 1, "Expected Repeat to contain 1 operation");
                }
                other => {
                    panic!("Expected Repeat operation, got: {:#?}", other);
                }
            }
        }
        other => {
            panic!("Expected Compose operation, got: {:#?}", other);
        }
    }
}

#[test]
fn test_choose_with_repeat_transformation() {
    let parse_str = r#"{ f: 200, l: 1.0, g: 1.0, p: 0.0 }
main = {
  Choose[Tm 1/1, Tm 5/4] | Repeat 2
}"#;

    let mut defs: Defs = Default::default();

    let _init = socool::SoCoolParser::new().parse(&mut defs, parse_str).unwrap();

    let main = defs.ops.get("main").unwrap();

    match main {
        Term::Op(Op::Compose { operations }) => {
            println!("Compose operations for Choose + Repeat: {:#?}", operations);

            // Should be transformed to Repeat{[Choose[...]], 2}
            assert_eq!(operations.len(), 1, "Expected one Repeat operation after transformation");

            match &operations[0] {
                Term::Op(Op::Repeat { operations: repeat_ops, count }) => {
                    assert_eq!(*count, 2, "Expected Repeat count to be 2");
                    assert_eq!(repeat_ops.len(), 1, "Expected Repeat to contain 1 operation (Choose)");

                    // Verify the inner operation is a Choose
                    match &repeat_ops[0] {
                        Term::Op(Op::Choose { .. }) => {
                            println!("Successfully found Choose inside Repeat");
                        }
                        other => {
                            panic!("Expected Choose inside Repeat, got: {:#?}", other);
                        }
                    }
                }
                other => {
                    panic!("Expected Repeat operation, got: {:#?}", other);
                }
            }
        }
        other => {
            panic!("Expected Compose operation, got: {:#?}", other);
        }
    }
}
