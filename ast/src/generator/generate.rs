use crate::{
    generator::error_non_generator, handle_id_error, Axis, Coefs, Defs, GenOp, Generator,
    NormalForm, Normalize, Op, Term,
};
use num_rational::Rational64;
use weresocool_error::Error;
use weresocool_shared::helpers::et_to_rational;
impl Coefs {
    fn generate(&mut self) -> Op {
        let result = self.axis.generate(self.state, self.div);
        self.state += self.coefs[self.idx];
        self.idx += 1;
        self.idx %= self.coefs.len();
        result
    }
}

impl Axis {
    fn generate(&self, state: i64, div: usize) -> Op {
        match self {
            Axis::F => Op::TransposeM {
                m: et_to_rational(state, div),
            },
            Axis::L => Op::Length {
                m: Rational64::new(std::cmp::max(1, state), div as i64),
            },
            Axis::G => Op::Gain {
                m: Rational64::new(std::cmp::max(1, state), div as i64),
            },
            Axis::P => Op::PanA {
                a: Rational64::new(state, div as i64),
            },
        }
    }
}

impl Generator {
    pub fn generate(
        &mut self,
        nf: &NormalForm,
        n: usize,
        defs: &Defs,
    ) -> Result<Vec<NormalForm>, Error> {
        let mut result: Vec<NormalForm> = vec![];
        let mut coefs = self.coefs.clone();

        for _ in 0..n {
            let mut nf: NormalForm = nf.clone();
            for coef in coefs.iter_mut() {
                coef.generate().apply_to_normal_form(&mut nf, defs)?;
            }
            result.push(nf)
        }

        Ok(result)
    }
}

impl GenOp {
    pub fn generate_from_genop(
        self,
        input: &mut NormalForm,
        n: Option<usize>,
        defs: &Defs,
    ) -> Result<Vec<NormalForm>, Error> {
        match self {
            GenOp::Named(name) => {
                let generator = handle_id_error(name, defs, None)?;
                match generator {
                    Term::Gen(gen) => gen.generate_from_genop(input, n, defs),
                    _ => Err(error_non_generator()),
                }
            }
            GenOp::Const(mut gen) => {
                let length = if let Some(n) = n { n } else { gen.lcm_length() };
                gen.generate(input, length, defs)
            }

            GenOp::Taken { gen, n } => gen.generate_from_genop(input, Some(n), defs),
        }
    }
}
