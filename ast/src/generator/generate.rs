use crate::{
    generator::error_non_generator, handle_id_error, Axis, CoefState, Coefs, Defs, GenOp,
    Generator, NormalForm, Normalize, Op, Term,
};
use num_rational::Rational64;
use polynomials::Polynomial;
use weresocool_error::Error;
use weresocool_shared::helpers::{et_to_rational, f32_to_rational, r_to_f64};

impl CoefState {
    pub fn generate(&mut self, mut rng: &mut rand::rngs::StdRng) -> Result<Op, Error> {
        match &mut self.coefs {
            Coefs::Const(coefs) => {
                let result = self.axis.generate_const(self.state, self.div);
                self.state += coefs[self.idx].get_value(&mut rng);
                self.idx += 1;
                self.idx %= coefs.len();
                Ok(result)
            }
            Coefs::Poly(poly) => {
                let result = self.axis.generate_poly(self.state, self.div, poly)?;
                self.state += 1;
                Ok(result)
            }
            Coefs::Expr { expr_str, parsed } => {
                if parsed.is_none() {
                    *parsed = Some(parse_expr(expr_str)?);
                };
                let result = self.axis.generate_expr(
                    self.state,
                    self.div,
                    parsed.as_ref().unwrap(),
                    expr_str,
                )?;
                self.state += 1;
                Ok(result)
            }
        }
    }
}

pub fn bind_x(e: &meval::Expr, s: &str) -> Result<impl Fn(f64) -> f64, Error> {
    let func = e.to_owned().bind("x");
    match func {
        Ok(f) => Ok(f),
        Err(err) => {
            println!("{}", err);
            Err(Error::with_msg(format!(
                "Unable to parse expression: {}",
                s
            )))
        }
    }
}

pub fn parse_expr(s: &str) -> Result<meval::Expr, Error> {
    let expr_parse: Result<meval::Expr, meval::Error> = s.parse();
    match expr_parse {
        Ok(e) => Ok(e),
        Err(err) => {
            println!("{}", err);
            Err(Error::with_msg(format!(
                "Unable to parse expression: {}",
                s
            )))
        }
    }
}

pub fn eval_polynomial(
    polynomial: &Polynomial<Rational64>,
    state: i64,
    div: usize,
) -> Result<Rational64, Error> {
    let result = polynomial.eval(Rational64::new(state, div as i64));
    if let Some(value) = result {
        Ok(value)
    } else {
        println!("Error: Polynomials must have at least one value.");
        Err(Error::with_msg("Polynomial must have at least one value."))
    }
}

impl Axis {
    pub fn at_least_axis_minimum(&self, r: Rational64, div: usize) -> Rational64 {
        match self {
            Axis::F => std::cmp::max(r, Rational64::from_integer(0)),
            Axis::G => std::cmp::max(r, Rational64::from_integer(0)),
            Axis::L => std::cmp::max(r, Rational64::new(1, div as i64)),
            Axis::P => r,
        }
    }

    pub fn evaluate_expr(
        &self,
        state: i64,
        div: usize,
        expr: &meval::Expr,
        s: &str,
    ) -> Result<Rational64, Error> {
        let func = bind_x(expr, s)?;
        let eval = func(state as f64 / div as f64);
        dbg!(f32_to_rational(eval as f32));
        Ok(f32_to_rational(eval as f32))
    }

    pub fn generate_expr(
        &self,
        state: i64,
        div: usize,
        expr: &meval::Expr,
        s: &str,
    ) -> Result<Op, Error> {
        let func = bind_x(expr, s)?;
        let eval = func(state as f64 / div as f64);
        dbg!(f32_to_rational(eval as f32));

        match self {
            Axis::F => Ok(Op::TransposeM {
                m: self.at_least_axis_minimum(f32_to_rational(2.0_f64.powf(eval) as f32), div),
            }),
            Axis::L => Ok(Op::Length {
                m: self.at_least_axis_minimum(f32_to_rational(eval as f32), div),
            }),
            Axis::G => Ok(Op::Gain {
                m: self.at_least_axis_minimum(f32_to_rational(eval as f32), div),
            }),
            Axis::P => Ok(Op::PanA {
                a: self.at_least_axis_minimum(f32_to_rational(eval as f32), div),
            }),
        }
    }

    fn generate_poly(
        &self,
        state: i64,
        div: usize,
        poly: &Polynomial<Rational64>,
    ) -> Result<Op, Error> {
        let eval = eval_polynomial(poly, state, div)?;

        match self {
            Axis::F => {
                let eval_f64 = r_to_f64(eval);
                let eval_in_log = 2.0_f32.powf(eval_f64 as f32);
                let rational = f32_to_rational(eval_in_log);
                Ok(Op::TransposeM {
                    m: self.at_least_axis_minimum(rational, div),
                })
            }
            Axis::L => Ok(Op::Length {
                m: self.at_least_axis_minimum(eval, div),
            }),
            Axis::G => Ok(Op::Gain {
                m: self.at_least_axis_minimum(eval, div),
            }),
            Axis::P => Ok(Op::PanA {
                a: self.at_least_axis_minimum(eval, div),
            }),
        }
    }
    fn generate_const(&self, state: i64, div: usize) -> Op {
        match self {
            Axis::F => Op::TransposeM {
                m: self.at_least_axis_minimum(et_to_rational(state, div), div),
            },
            Axis::L => Op::Length {
                m: self.at_least_axis_minimum(Rational64::new(state, div as i64), div),
            },
            Axis::G => Op::Gain {
                m: self.at_least_axis_minimum(Rational64::new(state, div as i64), div),
            },
            Axis::P => Op::PanA {
                a: Rational64::new(state, div as i64),
            },
        }
    }
}

impl Generator {
    pub fn term_vectors(
        &mut self,
        n: usize,
        rng: &mut rand::rngs::StdRng,
    ) -> Result<Vec<Op>, Error> {
        let mut result: Vec<Op> = vec![];
        let mut coefs = self.coefs.clone();

        for _ in 0..n {
            let mut operations: Vec<Term> = vec![];
            for coef in coefs.iter_mut() {
                operations.push(Term::Op(coef.generate(rng)?))
            }
            result.push(Op::Compose { operations })
        }

        Ok(result)
    }

    pub fn generate(
        &mut self,
        nf: &NormalForm,
        n: usize,
        defs: &Defs,
        mut rng: &mut rand::rngs::StdRng,
    ) -> Result<Vec<NormalForm>, Error> {
        let mut result: Vec<NormalForm> = vec![];
        let mut coefs = self.coefs.clone();

        for _ in 0..n {
            let mut nf: NormalForm = nf.clone();
            for coef in coefs.iter_mut() {
                coef.generate(&mut rng)?
                    .apply_to_normal_form(&mut nf, defs)?;
            }
            result.push(nf)
        }

        Ok(result)
    }
}

impl GenOp {
    pub fn term_vectors_from_genop(self, n: Option<usize>, defs: &Defs) -> Result<Vec<Op>, Error> {
        match self {
            GenOp::Named { name, seed } => {
                let generator = handle_id_error(name, defs, None)?;
                match generator {
                    Term::Gen(gen) => {
                        gen.to_owned().set_seed(seed);
                        gen.term_vectors_from_genop(n, defs)
                    }
                    _ => Err(error_non_generator()),
                }
            }
            GenOp::Const { mut gen, seed } => {
                let length = if let Some(n) = n { n } else { gen.lcm_length() };
                let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed);
                gen.term_vectors(length, &mut rng)
            }

            GenOp::Taken { gen, n, seed } => {
                gen.to_owned().set_seed(seed);
                gen.term_vectors_from_genop(Some(n), defs)
            }
        }
    }
    pub fn generate_from_genop(
        self,
        input: &mut NormalForm,
        n: Option<usize>,
        defs: &Defs,
    ) -> Result<Vec<NormalForm>, Error> {
        match self {
            GenOp::Named { name, seed } => {
                let generator = handle_id_error(name, defs, None)?;
                match generator {
                    Term::Gen(gen) => {
                        gen.to_owned().set_seed(seed);
                        gen.generate_from_genop(input, n, defs)
                    }
                    _ => Err(error_non_generator()),
                }
            }
            GenOp::Const { mut gen, seed } => {
                let length = if let Some(n) = n { n } else { gen.lcm_length() };

                let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed);
                gen.generate(input, length, defs, &mut rng)
            }

            GenOp::Taken { gen, n, seed } => {
                gen.to_owned().set_seed(seed);
                gen.generate_from_genop(input, Some(n), defs)
            }
        }
    }
}
