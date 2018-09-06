pub mod horns {
    use operations::Op;

    pub fn horns1() -> Op {
        fn phrase() -> Op {
            Op::Sequence {
                operations: vec![
                    Op::Compose {
                        operations: vec![
                            r![(4, 1, 0.0, 0.2, -0.5), (2, 1, 0.0, 0.2, 0.5)],
                            Op::Length { m: 8.0 },
                        ],
                    },
                    Op::Compose {
                        operations: vec![
                            r![(4, 2, 0.0, 0.2, 1.0), (2, 1, 0.0, 0.2, -1.0)],
                            Op::Length { m: 4.0 },
                        ],
                    },
                    Op::Compose {
                        operations: vec![
                            r![(12, 5, 0.0, 0.2, -1.0), (2, 1, 0.0, 0.2, 1.0)],
                            Op::Length { m: 4.0 },
                        ],
                    },
                    Op::Compose {
                        operations: vec![
                            r![(12, 5, 0.0, 0.2, 1.0), (2, 1, 0.0, 0.2, -1.0)],
                            Op::Length { m: 4.0 },
                        ],
                    },
                    Op::Compose {
                        operations: vec![
                            r![(9, 4, 0.0, 0.2, -1.0), (3, 2, 0.0, 0.2, 1.0)],
                            Op::Length { m: 4.0 },
                        ],
                    },
                    Op::Compose {
                        operations: vec![
                            r![(8, 3, 0.0, 0.2, -1.0), (9, 4, 0.0, 0.2, 1.0)],
                            Op::Length { m: 4.0 },
                        ],
                    },
                ],
            }
        }

        fn result() -> Op {
            Op::Sequence {
                operations: vec![Op::Silence { m: 1.0 }, phrase()],
            }
        }

        result()
    }
}
