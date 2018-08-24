pub mod violins {
    use operations::{Op, Apply};

    pub fn violins1() -> Op {
        fn sound() -> Op {
            r![(2, 1, 3.0, 0.8, 0.0), (1, 1, 0.0, 1.0, 0.0)]
        }

        fn violins1(m: Vec<(usize, usize)>) -> Op {
            Op::Sequence {
                operations: vec![
                    r![(m[0].0, m[0].1, 0.0, 1.0, 0.6)],
                    Op::Compose {
                        operations: vec![
                            r![(m[1].0, m[1].1, 0.0, 1.0, -0.1)],
                            Op::Length { m: 2.0 }
                        ]
                    },
                    Op::Silence { m: 1.0 }
                ],
            }
        };



        fn violins_phrase1() -> Op {
            Op::Sequence { operations: vec![
                violins1(vec![(3, 2), (6, 5)]),
                violins1(vec![(1, 1), (8, 5)]),
                violins1(vec![(4, 3), (9, 8)]),
                violins1(vec![(15, 16), (3, 2)]),
                violins1(vec![(2, 1), (1, 1)]),
                violins1(vec![(6, 5), (3, 2)]),
//                violins1(vec![(7, 4), (7, 8)]),
            ]}
        }

        fn result() -> Op {
            Op::Compose {
                operations: vec![
                    sound(),
                    violins_phrase1(),
                ]
            }
        }

        result()
    }
}
