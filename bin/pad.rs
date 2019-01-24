//extern crate socool_parser;
extern crate num_rational;
use num_rational::Rational64;
extern crate socool_ast;
//use socool_parser::parser::*;
use socool_ast::{
    ast::Op::*,
    operations::{NormalForm, Normalize as NormalizeOp},
};

fn main() {
    println!("\nHello Danny's WereSoCool Scratch Pad");

    let mut a = NormalForm::init();
    let mut b = NormalForm::init();

    Sequence {
        operations: vec![
            TransposeM {
                m: Rational64::new(3, 2),
            },
            TransposeM {
                m: Rational64::new(5, 4),
            },
            Length {
                m: Rational64::new(2, 1),
            },
        ],
    }
    .apply_to_normal_form(&mut a);

    Sequence {
        operations: vec![
            AsIs,
            TransposeA {
                a: Rational64::new(2, 1),
            },
            Length {
                m: Rational64::new(2, 1),
            },
        ],
    }
    .apply_to_normal_form(&mut b);

    let c = a * b;

    println!("{:#?}", c);
}
