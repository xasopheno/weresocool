//extern crate socool_parser;
extern crate num_rational;
use num_rational::{Ratio, Rational64};
extern crate socool_ast;
//use socool_parser::parser::*;
use socool_ast::{
    ast::{Op::*, OscType::Sine},
    operations::{NormalForm, Normalize as NormalizeOp, PointOp},
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

    //    println!("{:#?}", c);

    let expected = NormalForm {
        operations: vec![
            vec![
                PointOp {
                    fm: Ratio::new(3, 2),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(1, 1),
                    osc_type: Sine,
                },
                PointOp {
                    fm: Ratio::new(5, 4),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(1, 1),
                    osc_type: Sine,
                },
                PointOp {
                    fm: Ratio::new(1, 1),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(2, 1),
                    osc_type: Sine,
                },
            ],
            vec![
                PointOp {
                    fm: Ratio::new(3, 2),
                    fa: Ratio::new(2, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(1, 1),
                    osc_type: Sine,
                },
                PointOp {
                    fm: Ratio::new(5, 4),
                    fa: Ratio::new(2, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(1, 1),
                    osc_type: Sine,
                },
                PointOp {
                    fm: Ratio::new(1, 1),
                    fa: Ratio::new(2, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(2, 1),
                    osc_type: Sine,
                },
            ],
            vec![
                PointOp {
                    fm: Ratio::new(3, 2),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(2, 1),
                    osc_type: Sine,
                },
                PointOp {
                    fm: Ratio::new(5, 4),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(2, 1),
                    osc_type: Sine,
                },
                PointOp {
                    fm: Ratio::new(1, 1),
                    fa: Ratio::new(0, 1),
                    pm: Ratio::new(1, 1),
                    pa: Ratio::new(0, 1),
                    g: Ratio::new(1, 1),
                    l: Ratio::new(4, 1),
                    osc_type: Sine,
                },
            ],
        ],
        length_ratio: Ratio::new(8, 1),
    };

    assert_eq!(c, expected)
}
