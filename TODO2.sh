[x] OpOrNf => Term
[x] OpOrNfTable => TermTable
[x] Op::FunctionDef => Term::FunDef
[] Op::Fid => Op::Id

Op =
  Seq of ListOp
| Overlay of List<Op>
| Compose of Op, Op
| ...

list melody (scale) = scale @ [ 1, 2, 3, 4, 5, 4, 5]

melody (ET12)

melody (ET17)

pub enum Term {
    Id(String),
    Op(Op),
    Nf(NormalForm),
    FunDef(FunDef),
}

p10.1.2.93ub struct FunDef {
    pub name: String,
    pub vars: Vec<String>,
    pub term: Box<Term>,
}

fn1(a, b) = {
  Seq [a, b, a]
}

fn2(c, d) {
  Seq [c, d, c]

main = {
  fn2(fn1(Tm 1, Tm 3/2), fn1(Tm 9/8, Tm 4/3))
}

pub trait Substitute {
    fn substitute(&self, normal_form: &mut NormalForm, table: &TermTable, arg_map: &ArgMap)
        -> Term;
}

Term ->
Nf 
  - Cool!
Op 
  - Normalize it.
  - Store it. 
FunDef
  - (Simplify it)
  - Store it. 

Id(name)
  - Look it up
  - Replace it

Term =
| Op(op)
| Nf(nf)
| FunDef(FunDef)
#| ListDef(Lop)
#| ListDef(List(Nf))


Lop =
  List<Op>
| List<Op> [ List<Indices> ]
| ET(Int)

list l = [Tm 1, Tm 9/8, Tm 5/4, Tm 4/3, Tm 3/2, Tm 5/3, Tm 15/8]
list x = l[3, 2, 7, 9, 11]
list et12 = ET(12)

Seq [x[2, 4, 3, 3, 4, 1], Tm 9/8]

