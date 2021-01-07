use crate::generator::{error_non_generator, Axis, GenOp, Generator};
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
                Ok(gen.get_length(n))
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
                let n = if n.is_some() {
                    n.unwrap()
                } else {
                    gen.lcm_length()
                };
                Ok(gen.get_length(n))
            }
            GenOp::Taken { n, gen } => gen.get_length_ratio_genop(Some(*n), defs),
        }
    }
}

impl Generator {
    pub fn get_length(&self, n: usize) -> Rational64 {
        let mut lengths = vec![Rational64::new(1, 1); n];
        for coef in self.coefs.iter() {
            if let Axis::L = coef.axis {
                let mut state = coef.state_bak;
                lengths[0] *= Rational64::new(state, coef.div as i64);
                for (i, length) in lengths.iter_mut().enumerate().take(n).skip(1) {
                    state += coef.coefs[(i - 1) % coef.coefs.len()];
                    state = std::cmp::max(1, state);
                    *length *= Rational64::new(state, coef.div as i64);
                }
            }
        }
        let result = lengths
            .iter()
            .fold(Rational64::from_integer(0), |current, val| current + *val);

        result
    }

    pub fn lcm_length(&self) -> usize {
        let lengths: Vec<usize> = self.coefs.iter().map(|coef| coef.coefs.len()).collect();
        1 + lengths
            .iter()
            .fold(1usize, |current, val| lcm(current, *val))
    }
}
