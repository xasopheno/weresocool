pub mod bach {
    use operations::{Op, Operate};
    use ratios::{R, Pan};

    pub fn ratios(r: Vec<R>) -> Op {
        Op::Ratios {
            ratios: r
        }
    }

    pub fn maker(
        a: Vec<usize>,
        b: Vec<usize>,
        c: Vec<usize>,
        d: Vec<usize>,
    ) -> Op {
        Op::Compose {
            operations: vec![
                Op::Sequence { operations: vec![
                        ratios(r![
                                (a[0], a[1], 0.0, 1.0, 0.0),
                                (a[2], a[3], 0.0, 1.0, 0.0),
                            ]
                        ),
                        ratios(r![
                                (b[0], b[1], 0.0, 1.0, 0.0),
                                (b[2], b[3], 0.0, 1.0, 0.0),
                            ]
                        ),
                        ratios(r![
                                (c[0], c[1], 0.0, 1.0, 0.0),
                                (c[2], c[3], 0.0, 1.0, 0.0),
                            ]
                        ),
                        ratios(r![
                                (b[0], b[1], 0.0, 1.0, 0.0),
                                (b[2], b[3], 0.0, 1.0, 0.0),
                            ]
                        ),
                        ratios(r![
                                (d[0], d[1], 0.0, 1.0, 0.0),
                                (d[2], d[3], 0.0, 1.0, 0.0),
                            ]
                        ),
                        ratios(r![
                                (b[0], b[1], 0.0, 1.0, 0.0),
                                (b[2], b[3], 0.0, 1.0, 0.0),
                            ]
                        ),
                        ratios(r![
                                (c[0], c[1], 0.0, 1.0, 0.0),
                                (c[2], c[3], 0.0, 1.0, 0.0),
                            ]
                        ),
                        ratios(r![
                               (b[0], b[1], 0.0, 1.0, 0.0),
                               (b[2], b[3], 0.0, 1.0, 0.0),
                            ]
                        ),
                    ],
                },
                Op::Sequence {
                    operations: vec![
                        Op::AsIs,
                        Op::AsIs
                    ]
                }
            ]
        }
    }

    pub fn bach1() -> Op {
        let sequence1 = Op::Sequence {
            operations: vec![
                maker(
                    vec![2, 1, 1, 2],
                    vec![6, 5, 3, 4],
                    vec![9, 8, 2, 3],
                    vec![1, 1, 4, 5],
                ),
                maker(
                    vec![8, 5, 1, 2],
                    vec![4, 3, 4, 5],
                    vec![5, 4, 3, 4],
                    vec![1, 1, 2, 3],
                ),
                maker(
                    vec![15, 8, 1, 2],
                    vec![4, 3, 4, 5],
                    vec![6, 5, 3, 4],
                    vec![9, 8, 2, 3],
                ),
                maker(
                    vec![2, 1, 1, 2],
                    vec![3, 2, 3, 5],
                    vec![4, 3, 9, 16],
                    vec![6, 5, 3, 4],
                ),
                maker(
                    vec![5, 2, 1, 2],
                    vec![8, 5, 1, 1],
                    vec![3, 2, 7, 8],
                    vec![6, 5, 4, 5],
                ),
//                maker(
//                    vec![9, 4, 1, 2],
//                    vec![11, 8, 5, 6],
//                    vec![5, 4, 7, 8],
//                    vec![9, 8, 4, 5],
//                ),

            ],

        };

        sequence1
    }
}