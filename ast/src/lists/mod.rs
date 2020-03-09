pub mod indices;
pub mod normalize_listop;
pub mod substitute_list;
use indices::Indices;
use rand::{rngs::StdRng, thread_rng, Rng, SeedableRng};

use crate::Term;

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum ListOp {
    Const(Vec<Term>),
    Named(String),
    ListOpIndexed {
        list_op: Box<ListOp>,
        indices: Indices,
    },
}

