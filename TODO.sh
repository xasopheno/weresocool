[x] OpOrNf => Term
[x] OpOrNfTable => TermTable
[x] Op::FunctionDef => Term::FunDef
[] Op::Fid => Op::Id

Op =
  Seq of ListOp
| Overlay of List<Op>
| Compose of Op, Op
| ...

ListOp =
  StandardList of List<Op>
| IndexInto of ListOp, List<Indices>
| ET of Int
| ListVar of String

Indices =
| IntWithOp of Int, Op

Term =
  Op of Op
| Nf of Nf
| Lop of ListOp
| LNf of List<Nf>
| FunDef of FunctionDef

Definition =
  TermDef of Term

list melody (scale) = scale @ [ 1, 2, 3, 4, 5, 4, 5]

melody (ET12)

melody (ET17)

#pub enum Term {
    #Op(Op),
    #Nf(NormalForm),
    #FunDef(FunDef),
#}

pub struct FunDef {
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
Id(string)
| Op(op)
| Nf(nf)
| Fop(FunDef | FunCall)
# | Lop(ListDef | ListCall)


     
