//extern crate socool_parser;
//extern crate num_rational;
//extern crate socool_ast;
//use num_rational::{Ratio, Rational64};
//use socool_parser::parser::*;
//use socool_ast::{
//    ast::{Op::*, OscType::Sine},
//    operations::{NormalForm, Normalize as NormalizeOp, PointOp},
//};

fn main() {
    println!("\nHello Danny's WereSoCool Scratch Pad");

    //    foo = { foo_op }
    //    table.insert('foo', foo_op.Normalize())
    //    foo = NormalForm(Op, LR)

    //      bar = {
    //          Sequence [
    //          AsIs,
    //          Tm 3/2,
    //          foo => Id('foo'),
    //          AsIs,
    //          ]
    //          > FitLength foo => (Length lr_input/lr_foo)
    //      }

    //      table.insert('bar', bar.Normalize())
    //
    //      main = {
    //          Sequence [
    //              bar | bar => Id(bar) * Id(bar)
    //              Overlay [bar, bar] => Id(bar).join(Id(bar))
    //              Sequence [bar, bar] = Id(bar).overlay(Id(bar))
    //                  foo > Fit FitWord bar => fn((FitLength)(foo, bar)) => NormalForm
    //                  bar > @foo | baz => fn(bar, foo, baz) => NormalForm(Overlay(bar - foo, foo * baz))
    //          ]

    //      }
}
