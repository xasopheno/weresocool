pub mod material3 {
    use operations::{Op};

    fn overtones() -> Op {
        r![
            (4, 1, 0.0, 0.04, -0.2),
            (3, 1, 7.0, 0.04, 0.2),
            (9, 4, 5.0, 0.05, 1.0),
            (9, 4, 0.0, 0.05, 1.0),
            (5, 2, 4.0, 0.25, -1.0),
            (5, 2, 0.0, 0.25, -1.0),
            (7, 4, 3.0, 0.05, 0.7),
            (7, 4, 0.0, 0.05, 0.7),
            (3, 2, 4.0, 0.25, -0.7),
            (3, 2, 0.0, 0.25, -0.7),
            (2, 1, 0.0, 0.75, 0.4),
            (2, 1, 0.0, 0.75, 0.4),
        ]
    }

    pub fn s5_melody() -> Op {
        Op::Compose { operations: vec![
            Op::Sequence { operations: vec![

            ]}
        ]}
    }

    pub fn s5_bass() -> Op {
        Op::Compose { operations: vec![
            Op::Sequence {
                operations: vec![
                    Op::AsIs,

                ],
            },
            Op::TransposeM {m: 3.0/2.0}
        ]}
    }
}




