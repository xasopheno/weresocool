pub mod material3 {
    use operations::{Op};

    fn overtones() -> Op {
        r![
            (8, 1, 5.0, 0.1, 0.8),
            (8, 1, 0.0, 0.1, -0.8),
            (2, 1, 5.0, 0.25, 0.8),
            (2, 1, 0.0, 0.25, -0.8),
            (3, 1, 7.0, 0.1, 0.4),
            (3, 1, 0.0, 0.1, -0.4),
            (1, 1, 3.0, 0.75, 0.3),
            (1, 1, 0.0, 0.75, -0.3),
        ]
    }

    pub fn s5_h1() -> Op {
        Op::Compose { operations: vec![
            Op::Sequence { operations: vec![
                Op::TransposeM {m: 3.0/2.0},
                Op::TransposeM {m: 3.0/2.0},
                Op::TransposeM {m: 2.0/1.0},
                Op::TransposeM {m: 2.0/1.0},
                Op::TransposeM {m: 9.0/4.0},
                Op::TransposeM {m: 9.0/4.0},
                Op::TransposeM {m: 9.0/4.0},
                Op::TransposeM {m: 9.0/4.0},
                Op::TransposeM {m: 7.0/4.0},
                Op::TransposeM {m: 7.0/4.0},
                Op::TransposeM {m: 9.0/4.0},
                Op::TransposeM {m: 9.0/4.0},
                Op::TransposeM {m: 2.0/1.0},
                Op::TransposeM {m: 2.0/1.0},
                Op::TransposeM {m: 2.0/1.0},
                Op::TransposeM {m: 2.0/1.0},
                Op::TransposeM {m: 7.0/4.0},
                Op::TransposeM {m: 7.0/4.0},
                Op::TransposeM {m: 3.0/2.0},
                Op::TransposeM {m: 3.0/2.0},
                Op::TransposeM {m: 4.0/3.0},
                Op::TransposeM {m: 4.0/3.0},
                Op::TransposeM {m: 3.0/2.0},
                Op::TransposeM {m: 3.0/2.0},
                Op::TransposeM {m: 3.0/2.0},
                Op::TransposeM {m: 1.0/1.0},
                Op::TransposeM {m: 1.0/1.0},
                Op::TransposeM {m: 3.0/2.0},
                Op::TransposeM {m: 2.0/1.0},
                Op::TransposeM {m: 2.0/1.0},
                Op::TransposeM {m: 2.0/1.0},
                Op::TransposeM {m: 5.0/4.0},
                Op::TransposeM {m: 5.0/4.0},
                Op::TransposeM {m: 9.0/2.0},
                Op::TransposeM {m: 9.0/2.0},
            ]},
            Op::Length { m: 0.5},
            Op::TransposeM { m: 2.0},
            Op::Gain { m: 1.2},
        ]}
    }

    pub fn s5_melody() -> Op {
        Op::Compose { operations: vec![
            Op::Sequence { operations: vec![
                Op::TransposeM {m: 7.0/8.0},
                Op::TransposeM {m: 4.0/3.0},
                Op::TransposeM {m: 6.0/5.0},
                Op::TransposeM {m: 6.0/5.0},
                Op::TransposeM {m: 1.0/1.0},
                Op::TransposeM {m: 3.0/2.0},
                Op::TransposeM {m: 4.0/3.0},
                Op::TransposeM {m: 4.0/3.0},
                Op::TransposeM {m: 3.0/2.0},
                Op::TransposeM {m: 4.0/3.0},
                Op::TransposeM {m: 9.0/8.0},
                Op::TransposeM {m: 1.0/1.0},
                Op::TransposeM {m: 1.0/1.0},
//
                Op::TransposeM {m: 3.0/2.0},
                Op::TransposeM {m: 2.0/1.0},
                Op::TransposeM {m: 2.0/1.0},
                Op::TransposeM {m: 9.0/4.0},
                Op::TransposeM {m: 9.0/4.0},
//
                Op::TransposeM {m: 5.0/2.0},
                Op::TransposeM {m: 8.0/3.0},
                Op::TransposeM {m: 9.0/4.0},
                Op::TransposeM {m: 2.0/1.0},
                Op::TransposeM {m: 2.0/1.0},
                Op::TransposeM {m: 4.0/1.0},
                Op::TransposeM {m: 7.0/2.0},
                Op::TransposeM {m: 3.0/1.0},
                Op::TransposeM {m: 3.0/1.0},
                Op::TransposeM {m: 9.0/2.0},
                Op::TransposeM {m: 9.0/2.0},
                Op::TransposeM {m: 9.0/2.0},
                Op::TransposeM {m: 9.0/2.0},
                Op::TransposeM {m: 4.0/2.0},
                Op::TransposeM {m: 4.0/2.0},
                Op::TransposeM {m: 3.0/2.0},
                Op::TransposeM {m: 3.0/2.0},
                Op::TransposeM {m: 9.0/8.0},
                Op::TransposeM {m: 9.0/8.0},
                Op::TransposeM {m: 9.0/8.0},
                Op::TransposeM {m: 1.0/1.0},
                Op::TransposeM {m: 1.0/1.0},
                Op::TransposeM {m: 1.0/1.0},
                Op::TransposeM {m: 1.0/1.0},
            ]},
            Op::Length { m: 0.3333},
            Op::TransposeM { m: 4.0},
            Op::Gain { m: 2.0},
        ]}
    }

    pub fn s5_bass() -> Op {
        Op::Compose { operations: vec![
            Op::Sequence {
                operations: vec![
                    Op::TransposeM {m: 1.0/1.0},
                    Op::TransposeM {m: 9.0/8.0},
                    Op::TransposeM {m: 6.0/5.0},
                    Op::TransposeM {m: 9.0/8.0},
                    Op::TransposeM {m: 9.0/4.0},
                    Op::TransposeM {m: 2.0/1.0},
                    Op::TransposeM {m: 2.0/1.0},
                    Op::TransposeM {m: 2.0/1.0},
                    Op::TransposeM {m: 7.0/4.0},
                    Op::TransposeM {m: 7.0/4.0},
                    Op::TransposeM {m: 7.0/4.0},
                    Op::TransposeM {m: 5.0/3.0},
                    Op::TransposeM {m: 5.0/3.0},
                    Op::TransposeM {m: 4.0/3.0},
                    Op::TransposeM {m: 5.0/4.0},
                    Op::TransposeM {m: 7.0/6.0},
                    Op::TransposeM {m: 5.0/4.0},
                    Op::TransposeM {m: 1.0/1.0},
                    Op::TransposeM {m: 15.0/16.0},
                    Op::TransposeM {m: 9.0/8.0},
                    Op::TransposeM {m: 3.0/2.0},
                    Op::TransposeM {m: 4.0/3.0},
                    Op::TransposeM {m: 5.0/4.0},
                    Op::TransposeM {m: 9.0/8.0},
                ],
            },
            overtones(),
//            Op::TransposeM {m: 3.0/2.0}
        ]}
    }

    pub fn sequence5() -> Op {
        Op::Repeat {
            n: 1,
            operations: vec![
            Op::Overlay {
                operations: vec![
                s5_melody(),
                s5_h1(),
                s5_bass()
            ]}
        ]}
    }
}




