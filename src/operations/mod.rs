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
    Sum( operations: [
      AsIs,

      Product( operations: [
        T { m: 2.0, a: 0.0},
        R { r: ratios }
      ]),

      Product([
        Add[
            AsIs,
            T { m: 2.0, a: 0.0}
        ],
        T { m: 1.5, a: 0.0} )
      ])
    ])

let ops =
    Sum
        Asis,
        Product
            T 2.0, 0.0
            R ratios
        Product
            Add
                AsIs
                T 2.0 0.0

            Transpose 1.5, 0.0
