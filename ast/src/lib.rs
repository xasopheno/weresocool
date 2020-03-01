#[macro_use]
extern crate serde;
pub mod ast;
pub mod lists;
pub mod operations;
pub mod term;
pub use crate::{
    ast::{is_choice_op, Defs, FunDef, Op, Op::*, OscType, ASR},
    lists::{Index, IndexVector, Indices, ListNf, ListOp},
    operations::{
        substitute::substitute_operations, ArgMap, GetLengthRatio, NameSet, NormalForm, Normalize,
        PointOp, Substitute,
    },
    term::Term,
};
