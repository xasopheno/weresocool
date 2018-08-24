pub mod violins {
    use operations::{Op};

    pub fn violins1() -> Op {
        fn sound() -> Op {
            r![(4, 1, 3.0, 0.25, 0.0), (2, 1, 0.0, 0.8, 0.0)]
        }

        fn violins1(m: Vec<(usize, usize)>, d: f32) -> Op {
            Op::Sequence {
                operations: vec![
                    r![(m[0].0, m[0].1, 0.0, 1.0, 0.9 * d)],
                    r![(m[1].0, m[1].1, 0.0, 1.0, 0.0)],
                    r![(m[1].0, m[1].1, 0.0, 1.0, -0.9 * d)],
                    Op::Silence { m: 1.0 }
                ],
            }
        };



        fn violins_phrase1() -> Op {
            Op::Sequence { operations: vec![
                violins1(vec![(3, 2), (6, 5)], -1.0),
                violins1(vec![(1, 1), (8, 5)], 1.0),
                violins1(vec![(4, 3), (9, 8)], -1.0),
                violins1(vec![(15, 16), (3, 2)], 1.0),
                violins1(vec![(2, 1), (1, 1)], -1.0),
                violins1(vec![(6, 5), (3, 2)], 1.0),
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
