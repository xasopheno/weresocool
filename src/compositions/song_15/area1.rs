pub mod material {
    use operations::{Op};

    pub fn overtones() -> Op {
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
            (1, 1, 0.0, 0.75, 0.0),
            (1, 1, 0.0, 0.75, 0.0),
        ]
    }

    fn overtones2() -> Op {
        r![
//            (2, 1, 3.0, 0.1, 0.6),
//            (2, 1, 0.0, 0.1, -0.6),
//            (3, 2, 1.0, 0.10, 0.2),
//            (3, 2, 0.0, 0.10, -0.2),
            (1, 1, 0.0, 0.75, 0.0),
            (1, 1, 0.0, 0.75, 0.0),
//            (1, 2, 0.0, 0.05, 0.0),
//            (1, 2, 4.0, 0.05, 0.0),
        ]
    }

    pub fn sequence1() -> Op {
        Op::Sequence {
            operations: vec![
                Op::AsIs,
                Op::Compose { operations: vec![
                    Op::Sequence { operations: vec![
                        Op::TransposeM { m: 3.0/2.0 },
                        Op::TransposeM { m: 8.0/5.0 },
                        Op::TransposeM { m: 3.0/2.0 },
                    ]},
                    Op::Gain {m: 0.5},
                    Op::Length { m: 0.333333 }
                ]},
                Op::Compose { operations: vec![
                    Op::AsIs,
                    Op::Gain {m: 1.05}
                ]}
            ],
        }
    }

    pub fn sequence2() -> Op {
        Op::Sequence {
            operations: vec![
                Op::AsIs,
                Op::TransposeM { m: 7.0/8.0 },
                Op::TransposeM { m: 4.0/5.0 },
                Op::Compose { operations: vec![
                    Op::TransposeM { m: 3.0/4.0 },
                    Op::Gain {m: 1.5}
                ]}
            ],
        }
    }

    pub fn sequence3() -> Op {
        Op::Sequence {
            operations: vec![
                Op::AsIs,
                Op::TransposeM { m: 2.0/3.0 },
            ],
        }
    }

    pub fn with_overtones() -> Op {
        Op::Compose {
            operations: vec![
                overtones(),
                sequence1(),
            ],
        }
    }

    pub fn melody() -> Op {
        Op::Compose {
            operations: vec![
                sequence1(),
                sequence2(),
                sequence3(),
            ],
        }
    }

    pub fn fit() -> Op {
        Op::Fit {
            n: 2,
            with_length_of: Box::new(melody()),
            main:
            Box::new(
                Op::Compose { operations: vec![
                    with_overtones(),
                    melody(),
                    Op::TransposeM { m: 3.0/1.0 },
                    Op::Gain { m: 0.5 }
                ]
                })
        }
    }

    pub fn fit_again() -> Op {
        Op::Fit {
            n: 2,
            with_length_of: Box::new(melody()),
            main:
            Box::new(
                Op::Compose { operations: vec![
                    fit(),
                    Op::TransposeM { m: 3.0/2.0 },
                    Op::Gain { m: 0.5 }
                ]
                })
        }
    }

    pub fn overlay() -> Op {
        Op::Overlay { operations: vec![
            fit_again(),
            fit(),
            Op::Compose { operations: vec![
//                overtones2(),
                melody(),
                Op::Gain { m: 0.7 }
            ]}
        ]}
    }

    pub fn repeat() -> Op {
        Op::Repeat {
            operations: vec![overlay()],
            n: 3
        }
    }
}
