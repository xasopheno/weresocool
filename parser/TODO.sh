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
