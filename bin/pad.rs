extern crate num_rational;
extern crate socool_parser;
extern crate weresocool;
use num_rational::{Ratio, Rational64};
use socool_parser::ast::Op::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use weresocool::instrument::oscillator::OscType::{Sine, Noise};
use weresocool::operations::NormalForm;
use weresocool::operations::PointOp;

fn main() {
    println!("\nHello Danny's WereSoCool Scratch Pad");
    let mut a = PointOp::init();

    let mut b = PointOp::init();
    b.l = Rational64::new(2, 1);

    let mut vec_a = vec![a.clone(), a.clone()];
    let mut vec_b = vec![b.clone(), b.clone()];

    let result: Vec<PointOp> = vec_a
        .iter_mut()
        .map(|a| {
            let mut r = a.clone();
            for val in vec_b.iter() {
                r *= b.clone();
            }
            r
        })
        .collect();

    println!("{:#?}", result);
}

#[test]
fn test_point_op() {
    let mut a = PointOp::init();

    let mut b = PointOp::init();
    b.l = Rational64::new(2, 1);
    b.fm = Rational64::new(5, 4);

    let mut c = PointOp::init();
    c.g = Rational64::new(3, 2);
    c.fa = Rational64::new(2, 1);

    let mut d = PointOp::init();
    c.osc_type = Noise;
    c.pa = Rational64::new(1, 1);

    let mut nf_a = vec![vec![a.clone(), a.clone()]];
    let mut nf_b = vec![vec![b.clone(), c.clone(), d.clone()]];

    let mut new_nf = vec![];
    for vec_a in nf_a.iter() {
        for point_op_a in vec_a.iter() {
            for vec_b in nf_b.iter() {
                let mut new_vec: Vec<PointOp> = vec![];
                for point_op_b in vec_b {
                    new_vec.push(point_op_b.clone() * point_op_a.clone())
                }
                new_nf.push(new_vec)
            }
        }
    }

    println!("{:#?}", new_nf);
    let expected = vec![
        vec![
            PointOp {
                fm: Rational64::new(5, 4),
                fa: Rational64::new(0, 1),
                pm: Rational64::new(1, 1),
                pa: Rational64::new(0, 1),
                g: Rational64::new(1, 1),
                l: Rational64::new(2, 1),
                osc_type: Sine,
            },
            PointOp {
                fm: Rational64::new(5, 4),
                fa: Rational64::new(0, 1),
                pm: Rational64::new(1, 1),
                pa: Rational64::new(0, 1),
                g: Rational64::new(1, 1),
                l: Rational64::new(2, 1),
                osc_type: Sine,
            },
            PointOp {
                fm: Rational64::new(1, 1),
                fa: Rational64::new(2, 1),
                pm: Rational64::new(1, 1),
                pa: Rational64::new(0, 1),
                g: Rational64::new(3, 2),
                l: Rational64::new(1, 1),
                osc_type: Sine,
            },
        ],
        vec![
            PointOp {
                fm: Rational64::new(5, 4),
                fa: Rational64::new(0, 1),
                pm: Rational64::new(1, 1),
                pa: Rational64::new(0, 1),
                g: Rational64::new(1, 1),
                l: Rational64::new(2, 1),
                osc_type: Sine,
            },
            PointOp {
                fm: Rational64::new(5, 4),
                fa: Rational64::new(0, 1),
                pm: Rational64::new(1, 1),
                pa: Rational64::new(0, 1),
                g: Rational64::new(1, 1),
                l: Rational64::new(2, 1),
                osc_type: Sine,
            },
            PointOp {
                fm: Rational64::new(1, 1),
                fa: Rational64::new(2, 1),
                pm: Rational64::new(1, 1),
                pa: Rational64::new(0, 1),
                g: Rational64::new(3, 2),
                l: Rational64::new(1, 1),
                osc_type: Sine,
            },
        ],
    ];
    assert_eq!(new_nf, expected)
}

//fn multiply_normal_form() -> NormalForm {
//
//}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
