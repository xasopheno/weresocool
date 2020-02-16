#[macro_use]
extern crate serde;
pub mod ast;
pub mod operations;
pub use crate::{
    ast::{FunDef, ListOp, Op, Op::*, OscType, Term, TermTable, ASR},
    operations::{GetLengthRatio, NameSet, NormalForm, Normalize, PointOp, Substitute},
};
