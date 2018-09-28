pub mod cello {
    use operations::Op;

    pub fn cello1() -> Op {
        fn phrase() -> Op {
            Op::Sequence {
                operations: vec![
                    Op::Compose {
                        operations: vec![
                            Op::Sequence {
                                operations: vec![
                                    r![(1, 2, 0.0, 0.35, -0.5)],
                                    r![(3, 4, 0.0, 0.35, 0.0)],
                                    r![(6, 5, 0.0, 0.35, 0.5)],
                                    r![(3, 2, 0.0, 0.35, 0.0)],
                                ],
                            },
                            Op::Length { m: 0.5 },
                        ],
                    },
                    r![(1, 1, 0.0, 0.35, 0.0)],
                    Op::Silence { m: 1.0 },
                    //
                    Op::Compose {
                        operations: vec![
                            Op::Sequence {
                                operations: vec![
                                    r![(1, 2, 0.0, 0.35, -0.5)],
                                    r![(4, 5, 0.0, 0.35, 0.0)],
                                    r![(4, 3, 0.0, 0.35, 0.5)],
                                    r![(8, 5, 0.0, 0.35, 0.0)],
                                ],
                            },
                            Op::Length { m: 0.5 },
                        ],
                    },
                    r![(1, 1, 0.0, 0.35, 0.0)],
                    Op::Silence { m: 1.0 },
                    //
                    Op::Compose {
                        operations: vec![
                            Op::Sequence {
                                operations: vec![
                                    r![(1, 2, 0.0, 0.35, -0.5)],
                                    r![(3, 4, 0.0, 0.35, 0.0)],
                                    r![(15, 16, 0.0, 0.35, 0.5)],
                                    r![(9, 8, 0.0, 0.35, 0.0)],
                                ],
                            },
                            Op::Length { m: 0.5 },
                        ],
                    },
                    r![(3, 4, 0.0, 0.35, 0.0)],
                    Op::Silence { m: 1.0 },
                    //
                    Op::Compose {
                        operations: vec![
                            Op::Sequence {
                                operations: vec![
                                    r![(1, 2, 0.0, 0.35, -0.5)],
                                    r![(3, 4, 0.0, 0.35, 0.0)],
                                    r![(6, 5, 0.0, 0.35, 0.5)],
                                    r![(3, 2, 0.0, 0.35, 0.0)],
                                ],
                            },
                            Op::Length { m: 0.5 },
                        ],
                    },
                    r![(1, 1, 0.0, 0.35, 0.0)],
                    Op::Silence { m: 1.0 },
                    //5
                    Op::Compose {
                        operations: vec![
                            Op::Sequence {
                                operations: vec![
                                    r![(2, 5, 0.0, 0.35, -0.5)],
                                    r![(3, 5, 0.0, 0.35, 0.0)],
                                    r![(1, 1, 0.0, 0.35, 0.5)],
                                    r![(6, 5, 0.0, 0.35, 0.0)],
                                ],
                            },
                            Op::Length { m: 0.5 },
                        ],
                    },
                    r![(4, 5, 0.0, 0.35, 0.0)],
                    Op::Silence { m: 1.0 },
                    //
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
