#[macro_use]
extern crate serde;
pub mod ast;
pub mod list;
pub mod operations;
pub mod term;
pub use crate::{
    ast::{is_choice_op, Defs, FunDef, Op, Op::*, OscType, ASR},
    list::{Index, IndexList, Indices, ListNf, ListOp},
    operations::{ArgMap, GetLengthRatio, NameSet, NormalForm, Normalize, PointOp, Substitute},
    term::Term,
};
