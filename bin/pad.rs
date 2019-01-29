//extern crate socool_parser;
//extern crate num_rational;
extern crate socool_ast;
//use num_rational::{Ratio, Rational64};
//use socool_parser::parser::*;
use socool_ast::operations::{
    NormalForm,
    //        Normalize as NormalizeOp,
    PointOp,
};
use std::collections::hash_map::DefaultHasher;
//use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};

fn main() {
    println!("\nHello Danny's WereSoCool Scratch Pad");

    //    let mut btree = BTreeSet::new();
    //
    //    println!("{:?}", calculate_hash(&btree));
    //
    //    btree.insert("asdf".to_string());
    //
    //    println!("{:?}", calculate_hash(&btree));
    //
    //    btree.insert("asdf".to_string());
    //
    //    println!("{:?}", calculate_hash(&btree));
    //
    //    btree.insert("wasm".to_string());
    //
    //    println!("{:?}", calculate_hash(&btree));

    let mut nf1 = NormalForm::init();
    let mut nf2 = NormalForm::init();
    let mut p_op1 = PointOp::init();
    p_op1.names.insert("asdf".to_string());
    let p_op2 = PointOp::init();
    nf1.operations.push(vec![p_op1]);
    nf2.operations.push(vec![p_op2]);

    println!("{:?}", calculate_hash(&nf1));
    println!("{:?}", calculate_hash(&nf2));

    //    foo = { foo_op }
    //    table.insert('foo', foo_op.Normalize())
    //    foo = foo_op.normalize()

    //      bar = {
    //          Sequence [
    //              AsIs = AsIs.normalize,
    //              Tm 3/2 = Tm3/2.normalize,
    //              foo => Id('foo'),
    //              AsIs = AsIs.normalize,
    //          ].normalize
    //          > FitLength Length 2 => (Length lr_input/lr_foo)
    //      }.normalize

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

    //      Normalize:
    //          NormalizeForm -> self
    //          Op -> self.normalize()
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
