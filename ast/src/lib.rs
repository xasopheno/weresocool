#[macro_use]
extern crate serde;
pub mod ast;
pub mod list;
pub mod operations;
pub use crate::{
    ast::{is_choice_op, Defs, FunDef, Op, Op::*, OscType, Term, ASR},
    list::{Index, IndexList, Indices, ListNf, ListOp},
    operations::{GetLengthRatio, NameSet, NormalForm, Normalize, PointOp, Substitute},
};
