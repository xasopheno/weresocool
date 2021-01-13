use crate::{
    generator::error_non_generator, handle_id_error, Axis, CoefState, Coefs, Defs, GenOp,
    Generator, NormalForm, Normalize, Op, Term,
};
use num_rational::Rational64;
use polynomials::Polynomial;
use weresocool_error::Error;
use weresocool_shared::helpers::et_to_rational;

impl CoefState {
    fn generate(&mut self) -> Op {
        match &self.coefs {
            Coefs::Const(coefs) => {
                let result = self.axis.generate_const(self.state, self.div);
                self.state += coefs[self.idx];
                self.idx += 1;
                self.idx %= coefs.len();
                result
            }
            Coefs::Poly(poly) => {
                let result = self.axis.generate_poly(self.state, self.div, poly);
                self.state += 1;
                result
            }
        }
    }
}

impl Axis {
    fn generate_poly(&self, state: i64, div: usize, poly: &Polynomial<Rational64>) -> Op {
        let m = poly.eval(Rational64::new(state, div as i64)).unwrap();
        match self {
            Axis::F => Op::TransposeM { m },
            Axis::L => Op::Length { m },
            Axis::G => Op::Gain { m },
            Axis::P => Op::PanA { a: m },
        }
    }
    fn generate_const(&self, state: i64, div: usize) -> Op {
        match self {
            Axis::F => Op::TransposeM {
                m: et_to_rational(state, div),
            },
            Axis::L => Op::Length {
                m: Rational64::new(std::cmp::max(1, state), div as i64),
            },
            Axis::G => Op::Gain {
                m: Rational64::new(std::cmp::max(0, state), div as i64),
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
