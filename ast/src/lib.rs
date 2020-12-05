#[macro_use]
extern crate serde;
pub mod ast;
pub mod generator;
pub mod lists;
pub mod operations;
pub mod term;
pub use crate::{
    ast::{Defs, FunDef, Op, Op::*, OscType, ASR},
    generator::{Axis, Coefs, GenOp, Generator},
    lists::{Index, IndexVector, Indices, ListOp, TermVector},
    operations::{
        substitute::substitute_operations, ArgMap, GetLengthRatio, NameSet, NormalForm, Normalize,
        PointOp, Substitute,
    },
    term::Term,
};
