pub mod bass {
    use operations::Op;

    pub fn bass1() -> Op {
        fn phrase() -> Op {
            Op::Sequence {
                operations: vec![
                    Op::Compose {
                        operations: vec![
                            r![(1, 1, 0.0, 0.35, -0.5), (1, 2, 0.0, 0.35, 0.5)],
                            Op::Length { m: 2.0 },
                        ],
                    },
                    Op::Silence { m: 2.0 },
                ],
            }
        }

        fn phrase1() -> Op {
            Op::Sequence {
                operations: vec![
                    Op::Repeat {
                        operations: vec![phrase()],
                        n: 4,
                    },
                    Op::Compose {
                        operations: vec![
                            r![(4, 5, 0.0, 0.40, -0.5), (2, 5, 0.0, 0.40, 0.5)],
                            Op::Length { m: 2.0 },
                        ],
                    },
                    Op::Silence { m: 2.0 },
                    Op::Compose {
                        operations: vec![
                            r![(3, 10, 0.0, 0.45, -0.5), (3, 5, 0.0, 0.45, 0.5)],
                            Op::Length { m: 2.0 },
                        ],
                    },
                    Op::Silence { m: 2.0 },
                ],
            }
        }

        fn result() -> Op {
            Op::Sequence {
                operations: vec![
                    Op::Silence { m: 1.0 },
                    Op::Compose {
                        operations: vec![phrase1(), Op::Gain { m: 1.3 }],
                    },
                ],
            }
        }

        result()
    }
}
