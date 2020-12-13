use rand::{thread_rng, Rng};
use weresocool_ast::{Op, Term};
use weresocool_shared::cool_ratio::*;

pub fn et(d: i64) -> Vec<Term> {
    let mut ops = vec![];
    ops.push(Term::Op(Op::TransposeM {
        m: CoolRatio::from_int(1),
    }));
    for i in 1..d as usize {
        let m = 2.0_f32.powf(i as f32 / d as f32);
        ops.push(Term::Op(Op::TransposeM {
            m: CoolRatio::from_float_string(m.to_string()),
        }))
    }

    ops
}

pub fn random_seed() -> i64 {
    thread_rng().gen::<i64>()
}
