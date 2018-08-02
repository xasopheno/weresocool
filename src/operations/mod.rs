use ratios::R;

pub enum Operation {
    AsIs,
    Transpose,
    MutLength,
    MutGain,
    MutRatios,
    DoManyOperations,
    ConcatManyOperations
}

pub struct AsIs {
    index: int
}

pub struct Transpose {
    mul: f32,
    add: f32,
}


pub struct MutLength {
    mul: f32,
    add: f32,
}


pub struct MutGain {
    mul: f32,
    add: f32,
}

pub struct MutRatios {
    ratios: Vec<R>
}

pub struct DoManyOperations {
    operations: vec<Operation>
}

pub struct ConcatManyOperations {
    operations: vec<Operations>
}

Operator(e, operations);


let ops =
    ConcatManyOperations([
      AsIs,
      DoManyOperations([Transpose, MutRatios]),
      DoManyOperations([
        ConcatManyOperations[AsIs(i), Transpose],
        Transpose(...))
      ])
    ])

//    [
//        Transpose,
//        Multiple { [...] }
//        [MutLength, Transpose {mul: 1.5, add: 0.0}],
//        []
//    ]
