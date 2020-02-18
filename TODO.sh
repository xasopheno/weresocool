[x] OpOrNf => Term
[x] OpOrNfTable => TermTable
[x] Op::FunctionDef => Term::FunDef
[x] Op::Fid => Op::Id

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
| ListDef of ListDef

Definition =
  TermDef of Term

list melody (scale) = scale @ [ 1, 2, 3, 4, 5, 4, 5]

melody (ET12)

melody (ET17)


Term =
Id(string)
| Op(op)
| Nf(nf)
| Fop(FunDef | FunCall)
# | Lop(ListDef | ListCall)


     
