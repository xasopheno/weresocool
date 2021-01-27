use num_rational::Rational64;
use polynomials::*;
use rand::{rngs::StdRng, seq::SliceRandom, Rng};
use std::hash::{Hash, Hasher};

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Coef {
    Int(i64),
    RandRange(std::ops::RangeInclusive<i64>),
    RandChoice(Vec<i64>),
}

impl Coef {
    pub fn get_value(&self, mut rng: &mut StdRng) -> i64 {
        let result = match self {
            Self::Int(v) => *v,
            Self::RandRange(range) => rng.gen_range(range.to_owned()),
            Self::RandChoice(choices) => *choices.as_slice().choose(&mut rng).unwrap(),
        };
        result
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Coefs {
    Const(Vec<Coef>),
    Poly(Polynomial<Rational64>),
    Expr {
        expr_str: String,
        parsed: Option<meval::Expr>,
    },
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for Coefs {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Coefs::Const(c) => c.hash(state),
            Coefs::Poly(p) => p.iter().copied().collect::<Vec<Rational64>>().hash(state),
            _ => unimplemented!(),
        }
    }
}

impl Coefs {
    pub fn init_const(coefs: Vec<Vec<Coef>>) -> Coefs {
        Self::Const(coefs.into_iter().flatten().collect())
    }

    pub fn init_expr(expr_str: String) -> Coefs {
        Self::Expr {
            expr_str,
            parsed: None,
        }
    }

    pub fn init_polynomial(coefs: Vec<Rational64>) -> Coefs {
        let mut poly = Polynomial::new();
        for p in coefs {
            poly.push(p)
        }
        Coefs::Poly(poly)
    }

    pub fn len(&self) -> usize {
        match self {
            Coefs::Const(coefs) => coefs.len(),
            Coefs::Poly { .. } => unimplemented!(),
            _ => unimplemented!(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Coefs::Const(coefs) => coefs.is_empty(),
            Coefs::Poly(_coefs) => unimplemented!(),
            _ => unimplemented!(),
        }
    }
}
