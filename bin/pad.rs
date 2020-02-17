use failure::Fail;
use num_rational::Rational64;
use weresocool_ast::{ListOp, Op, Term};
use weresocool_error::Error;
use weresocool_parser::float_to_rational::helpers::f32_to_rational;

fn main() {
    match run() {
        Ok(_) => {}
        e => {
            for cause in Fail::iter_causes(&e.unwrap_err()) {
                println!("Failure caused by: {}", cause);
            }
        }
    }
}

fn et(d: usize) -> ListOp {
    let mut ops = vec![];
    ops.push(Term::Op(Op::TransposeM {
        m: Rational64::from_integer(1),
    }));
    for i in 1..d {
        let m = 2.0_f32.powf(i as f32 / d as f32);
        dbg!(m, i);
        ops.push(Term::Op(Op::TransposeM {
            m: f32_to_rational(m.to_string()),
        }))
    }

    ListOp::List(ops)
}

#[allow(unused_variables)]
fn run() -> Result<(), Error> {
    let result = et(12);
    dbg!(result);

    Ok(())
}
