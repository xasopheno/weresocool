use event::Event;
use ratios::R;

pub enum Operation {
    AsIs {},
    Transpose {mul: f32, add: f32},
    Length {mul: f32, add: f32},
    Gain {mul: f32, add: f32},
    Ratios {ratios: Vec<R>},
    Sequence {operations: Vec<Operation>},
    Compose {operations: Vec<Operation>},
}

//pub struct AsIs {}
//
//pub struct Transpose {
//    mul: f32,
//    add: f32,
//}
//
//pub struct Length {
//    mul: f32,
//    add: f32,
//}
//
//pub struct Gain {
//    mul: f32,
//    add: f32,
//}
//
//pub struct Ratios {
//    ratios: Vec<R>,
//}
//
//pub struct Sequence {
//    operations: Vec<Operation>,
//}
//
//pub struct Compose {
//    operations: Vec<Operation>,
//}

//pub trait Operate {
//    fn apply(&self, e: &Event) -> Vec<Operation>;
//}
//
//
impl Operate for Sequence {
    fn apply(&self, e: &Event) -> Vec<Event> {
        let mut container = vec![];
        for operation in self.operations.iter() {
            container.push(operation.apply(e))
        }
        container
    }
}


impl Operate<Compose> for Compose {
    fn apply(&self, e: Vec<Event>) -> Vec<Event> {
        let mut container = vec![];
        let result;
        for operation in self.operations.iter() {
            let events = vec![e.clone()];
            for event in events.iter() {
            newEvents.appendList(operation.apply(event))
    }
            event = operation.apply(event)
            container.push(operation.apply(event))
        }
        container
    }
}

//Operator(e, operations);
fn test() -> Operation {
    let ratios = r![1, 1, 0.0, 0.0, 0.0];
    let ops = Compose {
        operations: vec![
            Transpose {mul: 2.0, add: 0.0}
            Length {mul: 2.0, add: 0.0}
        ]

        operations: vec![
            Compose { operations: AsIs, AsIs }
            Sequence { operations: AsIs, Transpose {mul: 1.5, add: 0.0}}
        ]

        operations: vec![
            Sequence { operations: AsIs, AsIs }
            Compose { operations: AsIs, Transpose {mul: 1.5, add: 0.0}}
        ]

        operations: vec![
            Sequence { operations: AsIs, AsIs }
            Sequence { operations: AsIs, Transpose {mul: 1.5, add: 0.0}}
        ]
    }
//        Operation::Sequence {
//            operations: vec![
//              Opertation::AsIs {},
//              Operation::Compose {
//                operations: vec![
//                    Operation::Transpose { mul: 2.0, add: 0.0},
//                    Operation::Ratios { ratios }
//                  ]
//                },
//              Operation::Compose {
//                operations: vec![
//                    Operation::Sequence {
//                        operations:
//                    vec![
//                        Operation::AsIs {},
//                        Operation::Transpose { mul: 2.0, add: 0.0}
//                    ]},
//                    Operation::Transpose { mul: 1.5, add: 0.0}
//                  ]
//                }
//            ]
//        };
    ops
}



//
//            ]
//        ]
//    }


//    ops.with(event)
//thing1 = [
//    _;
//    ^ 3.0/2.0;
//    compose(
//         r[
//            (1, 2, 0.0, 0.3, 0.0)
//            (1, 2, 1.0, 0.3, 0.0)
//         ],
//         l 2.0, 0.0
//    );
//    compose(
//         r
//            (3, 2, 0.0, 0.5, 0.3)
//            (3, 2, 4.0, 0.5, -0.3)
//         l 1.0, 2.0
//         ^ 9.0/8.0
//    );
//     r[
//        (1, 2, 0.0, 0.3, 0.0)
//        (1, 2, 1.0, 0.3, 0.0)
//      ];
//]
