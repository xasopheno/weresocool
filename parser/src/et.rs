use crate::float_to_rational::helpers::f32_to_rational;
use num_rational::Rational64;
use weresocool_ast::{ListOp, Op, Term};

pub fn et(d: i64) -> ListOp {
    let mut ops = vec![];
    ops.push(Term::Op(Op::TransposeM {
        m: Rational64::from_integer(1),
    }));
    for i in 1..d as usize {
        let m = 2.0_f32.powf(i as f32 / d as f32);
        ops.push(Term::Op(Op::TransposeM {
            m: f32_to_rational(m.to_string()),
        }))
    }

    ListOp::List(ops)
}
