#[macro_use]
extern crate serde;
pub mod ast;
pub mod operations;
pub use crate::ast::{Op, Op::*, OpOrNf, OpOrNfTable, OscType};
pub use crate::operations::{GetLengthRatio, NameSet, NormalForm, Normalize, PointOp, Substitute};
