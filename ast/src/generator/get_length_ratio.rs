use crate::generator::{
    error_non_generator,
    generate::{eval_polynomial, parse_expr},
    Axis, Coefs, GenOp, Generator,
};
use crate::operations::helpers::handle_id_error;
use crate::{Defs, GetLengthRatio, Term};
use num_integer::lcm;
use num_rational::Rational64;
use weresocool_error::Error;

impl GetLengthRatio for GenOp {
    fn get_length_ratio(&self, defs: &Defs) -> Result<Rational64, Error> {
        match self {
            GenOp::Named(name) => {
                let generator = handle_id_error(name.to_string(), defs, None)?;
                match generator {
                    Term::Gen(gen) => gen.get_length_ratio_genop(None, defs),
                    _ => Err(error_non_generator()),
                }
            }
            GenOp::Const(gen) => {
                let n = gen.lcm_length();
                Ok(gen.get_length(n)?)
            }
            GenOp::Taken { n, gen } => gen.get_length_ratio_genop(Some(*n), defs),
        }
    }
}

impl GenOp {
    pub fn length(&self, defs: &Defs) -> Result<usize, Error> {
        match self {
            GenOp::Named(name) => {
                let generator = handle_id_error(name.to_string(), defs, None)?;
                match generator {
                    Term::Gen(gen) => gen.length(defs),
                    _ => Err(error_non_generator()),
                }
            }
            GenOp::Const(generator) => Ok(generator.lcm_length()),
            GenOp::Taken { n, .. } => Ok(*n),
        }
    }

    pub fn get_length_ratio_genop(
        &self,
        n: Option<usize>,
        defs: &Defs,
    ) -> Result<Rational64, Error> {
        match self {
            GenOp::Named(name) => {
                let generator = handle_id_error(name.to_string(), defs, None)?;
                match generator {
                    Term::Gen(gen) => gen.get_length_ratio_genop(n, defs),
                    _ => Err(error_non_generator()),
                }
            }
            GenOp::Const(gen) => {
                let n = if let Some(n) = n { n } else { gen.lcm_length() };
                Ok(gen.get_length(n)?)
            }
            GenOp::Taken { n, gen } => gen.get_length_ratio_genop(Some(*n), defs),
        }
    }
}

impl Generator {
    pub fn get_length(&self, n: usize) -> Result<Rational64, Error> {
        let mut lengths = vec![Rational64::new(1, 1); n];
        for coef in self.coefs.iter() {
            if let Axis::L = coef.axis {
                let mut state = coef.state_bak;
                match &coef.coefs {
                    Coefs::Const(c) => {
                        lengths[0] *= coef.axis.at_least_axis_minimum(
                            Rational64::new(state, coef.div as i64),
                            coef.div,
                        );
                        for (i, length) in lengths.iter_mut().enumerate().take(n).skip(1) {
                            state += c[(i - 1) % coef.coefs.len()];
                            *length *= coef.axis.at_least_axis_minimum(
                                Rational64::new(state, coef.div as i64),
                                coef.div,
                            );
                        }
                    }

                    Coefs::Poly(poly) => {
                        let r = eval_polynomial(poly, state, coef.div).unwrap();
                        lengths[0] *= coef.axis.at_least_axis_minimum(r, coef.div);
                        for length in lengths.iter_mut().take(n).skip(1) {
                            state += 1;
                            let r = eval_polynomial(poly, state, coef.div).unwrap();
                            *length *= coef.axis.at_least_axis_minimum(r, coef.div);
                        }
                    }

                    Coefs::Expr { expr_str, .. } => {
                        let parsed = parse_expr(expr_str)?;
                        let r = coef
                            .axis
                            .evaluate_expr(state, coef.div, &parsed, expr_str)?;
                        lengths[0] *= coef.axis.at_least_axis_minimum(r, coef.div);
                        for length in lengths.iter_mut().take(n).skip(1) {
                            state += 1;
                            let r = coef
                                .axis
                                .evaluate_expr(state, coef.div, &parsed, expr_str)?;
                            *length *= coef.axis.at_least_axis_minimum(r, coef.div);
                        }
                    }
                };
            }
        }
        Ok(lengths
            .iter()
            .fold(Rational64::from_integer(0), |current, val| current + *val))
    }

    pub fn lcm_length(&self) -> usize {
        let lengths: Vec<usize> = self
            .coefs
            .iter()
            .map(|coef| match &coef.coefs {
                Coefs::Const(c) => c.len(),
                Coefs::Poly(_) => coef.div - 1,
                Coefs::Expr { .. } => coef.div - 1,
            })
            .collect();
        1 + lengths
            .iter()
            .fold(1usize, |current, val| lcm(current, *val))
    }
}
