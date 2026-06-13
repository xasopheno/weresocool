#[macro_use]
extern crate serde;
pub mod ast;
pub mod color;
pub mod datagen;
pub mod drum_presets;
pub mod follow;
pub mod generator;
pub mod lists;
pub mod nameset;
pub mod operations;
pub mod rand_ctx;
pub mod term;
pub mod wgsl;
pub use crate::{
    ast::{
        ClapParams, Distortion, FmOscDef, FunDef, HiHatParams, KickParams, Op, Op::*, OscType,
        RimshotParams, SnareParams, ASR,
    },
    color::{
        CssOrHex, Color, ColorMap, ColorSet, ColorSets, ColorValue, GradientColor, GenColor,
        RandColor, RandColorSet,
    },
    datagen::Scale,
    generator::{
        coefs::{Coef, Coefs},
        Axis, CoefState, GenOp, Generator,
    },
    lists::{
        normalize_listop::join_list_nf, Direction, Index, IndexVector, Indices, ListOp, TermVector,
    },
    nameset::NameSet,
    operations::{
        helpers::{handle_id_error, join_sequence},
        substitute::substitute_operations,
        GetLengthRatio, NormalForm, Normalize, PointOp, Substitute,
        Defs, Span, SpanMap,
    },
    term::Term,
    wgsl::{WgslMap, validate_wgsl},
};
