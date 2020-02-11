#[macro_use]
extern crate serde;
pub mod ast;
pub mod operations;
pub use crate::{
    ast::{Op, Op::*, OpOrNfTable, OscType, Term, ASR},
    operations::{GetLengthRatio, NameSet, NormalForm, Normalize, PointOp, Substitute},
};
