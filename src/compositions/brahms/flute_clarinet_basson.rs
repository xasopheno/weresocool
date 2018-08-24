pub mod winds {
    use operations::{Op, Apply};

    pub fn winds1() -> Op {
        fn sound() -> Op {
            r![
                (4, 1, 0.0, 0.15, -0.7),
                (4, 1, 0.0, 0.15, 0.7),
                (2, 1, 0.0, 0.15, -0.5),
                (2, 1, 0.0, 0.15, 0.5),
                (1, 1, 0.0, 0.15, -0.9),
                (1, 1, 0.0, 0.15, 0.9),
            ]
        }

        fn phrase() -> Op {
            Op::Sequence {
                operations: vec![
                    r![(3, 2, 0.0, 1.0, 0.3), (6, 5, 0.0, 1.0, 0.0)],
                    r![(6, 5, 0.0, 1.0, 0.0), (1, 1, 0.0, 1.0, 0.3)],
                    r![(8, 5, 0.0, 1.0, 0.3), (4, 3, 0.0, 1.0, 0.0)],
                    r![(4, 3, 0.0, 1.0, 0.0), (9, 8, 0.0, 1.0, 0.3)],
                    r![(9, 8, 0.0, 1.0, 0.3), (15, 16, 0.0, 1.0, 0.0)],
                    r![(15, 16, 0.0, 1.0, 0.0), (3, 4, 0.0, 1.0, 0.3)],
                ]
            }
        };

        fn phrase_with_space() -> Op {
            Op::Compose { operations: vec![
                Op::Sequence { operations: vec![
                    Op::AsIs,
                    Op::Silence {m: 1.0}
                ]},
                phrase(),
            ]}
        }

        fn result() -> Op {
            Op::Sequence {operations: vec! [
                Op::Silence {m: 2.0},
                Op::Compose {
                    operations: vec![
                        sound(),
                        phrase_with_space(),
                    ]
                }
            ]}

        }

        result()
    }
}
