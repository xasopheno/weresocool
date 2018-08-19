macro_rules! r {
    ($(($num:expr,$den:expr,$offset:expr,$gain:expr,$pan:expr)),*$(,)*) => {
        Op::Overlay { operations: vec![$(
                Op::Compose { operations: vec! [
                    Op::TransposeM { m: $num as f32 /$den as f32 },
                    Op::TransposeA { a: $offset },
                    Op::Gain { m: $gain },
                    Op::Pan { a: $pan },
                ]},
            )*]
        }
    }
}

#[cfg(test)]
pub mod tests {
    use operations::Op;
    #[test]
    fn test_r_macro() {
        let r_macro = r![
            (1, 1, 3.0, 1.0, 1.0),
            (3, 2, 0.0, 0.6, -1.0),
            (5, 4, 1.5, 0.5, 0.5),
        ];

        let macro_test = Op::Overlay {
            operations: vec![
                Op::Compose {
                    operations: vec![
                        Op::TransposeM { m: 1.0 / 1.0 },
                        Op::TransposeA { a: 3.0 },
                        Op::Gain { m: 1.0 },
                        Op::Pan { a: 1.0 },
                    ],
                },
                Op::Compose {
                    operations: vec![
                        Op::TransposeM { m: 3.0 / 2.0 },
                        Op::TransposeA { a: 0.0 },
                        Op::Gain { m: 0.6 },
                        Op::Pan { a: -1.0 },
                    ],
                },
                Op::Compose {
                    operations: vec![
                        Op::TransposeM { m: 5.0 / 4.0 },
                        Op::TransposeA { a: 1.5 },
                        Op::Gain { m: 0.5 },
                        Op::Pan { a: 0.5 },
                    ],
                },
            ],
        };

        assert_eq!(r_macro, macro_test);
    }
}
