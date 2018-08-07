use ratios::{R, Pan};
use operations::{Op, Operate};
use event::Event;

pub fn generate_composition() -> (){
    let rs = r![
        (1, 1, 0.0, 0.6, -1.0),
    ];
    let e = vec![Event::new(100.0, rs, 1.0, 1.0)];

    let ops = Op::Sequence {
        operations: vec![
            Op::AsIs,
            Op::Transpose { m: 3.0/2.0, a: 0.0 },
            Op::Transpose { m: 5.0/4.0, a: 0.0 },
            Op::Compose {
                operations: vec![
                    Op::Transpose { m: 3.0/4.0, a: 0.0 },
                    Op::Length { m: 2.0, a: 0.0 },
                ]
            }
        ]
    };

    let ops2 = Op::Compose {
        operations: vec![
            ops.clone(),
            Op::Transpose { m: 1.5, a: 1.0 }
        ]
    };

    println!("{:?}", ops.apply(e.clone()));
    println!("{:?}", ops2.apply(e.clone()));
}
