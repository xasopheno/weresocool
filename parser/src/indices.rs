use crate::float_to_rational::helpers::f32_to_rational;
use num_rational::Rational64;
use rand::{thread_rng, Rng};
use weresocool_ast::{Op, Term};

pub fn et(d: i64) -> Vec<Term> {
    let mut ops = vec![Term::Op(Op::TransposeM {
        m: Rational64::from_integer(1),
    })];
    for i in 1..d as usize {
        let m = 2.0_f32.powf(i as f32 / d as f32);
        ops.push(Term::Op(Op::TransposeM {
            m: f32_to_rational(format!("{:.8}", m)),
        }))
    }

    ops
}

pub fn random_seed() -> i64 {
    thread_rng().gen::<i64>()
}
