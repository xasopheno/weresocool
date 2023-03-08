#[cfg(test)]
mod csv2d_tests {
    use hamcrest2::prelude::*;
    use num_rational::Rational64;

    use crate::{datagen::*, NameSet, NormalForm, PointOp};

    #[test]
    fn test_get_data_2d() {
        let result = get_data2d("./src/datagen/2d_test_data.csv".to_string()).unwrap();

        let expected = vec![vec![2.5, 1.0], vec![1.0, 2.0], vec![1.5, 2.0]];

        assert_that!(&result, contains(expected).exactly());
    }

    #[test]
    fn test_point_to_point_op() {
        let mut names = NameSet::new();
        names.insert("2d_test_data.csv".to_string());

        let result = point_to_point_op(
            &vec![1.0, 2.0],
            None,
            &vec![
                Scale {
                    axis: Axis::F,
                    value: Rational64::new(2, 1),
                },
                Scale {
                    axis: Axis::L,
                    value: Rational64::new(1, 2),
                },
            ],
            "2d_test_data.csv",
        );
        let expected = Term::Op(Op::Compose {
            operations: vec![
                Term::Op(Op::TransposeA {
                    a: Rational64::new(2, 1),
                }),
                Term::Op(Op::Length {
                    m: Rational64::new(2, 1),
                }),
            ],
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_csv_data_to_normal_form() {
        let csv_data = vec![vec![1.0, 1.0], vec![1.0, 1.0], vec![1.0, 1.0]];

        let mut names = NameSet::new();
        let filename = "2d_test_data.csv";
        let scales = vec![
            Scale {
                axis: Axis::F,
                value: Rational64::new(2, 1),
            },
            Scale {
                axis: Axis::L,
                value: Rational64::new(1, 2),
            },
        ];
        names.insert(filename.to_string());

        let result = csv_data_to_normal_form(&csv_data, scales, "2d_test_data.csv");
        let expected = Term::Op(Op::Sequence {
            operations: vec![
                Term::Op(Op::Compose {
                    operations: vec![
                        Term::Op(Op::TransposeA {
                            a: Rational64::new(2, 1),
                        }),
                        Term::Op(Op::Length {
                            m: Rational64::new(1, 1),
                        }),
                    ],
                }),
                Term::Op(Op::Compose {
                    operations: vec![
                        Term::Op(Op::TransposeA {
                            a: Rational64::new(2, 1),
                        }),
                        Term::Op(Op::Length {
                            m: Rational64::new(1, 1),
                        }),
                    ],
                }),
                Term::Op(Op::Compose {
                    operations: vec![
                        Term::Op(Op::TransposeA {
                            a: Rational64::new(2, 1),
                        }),
                        Term::Op(Op::Length {
                            m: Rational64::new(1, 1),
                        }),
                    ],
                }),
            ],
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_csv1d_to_normalform() {
        let scales = vec![
            Scale {
                axis: Axis::F,
                value: Rational64::new(2, 1),
            },
            Scale {
                axis: Axis::L,
                value: Rational64::new(1, 2),
            },
        ];

        let result = csv2d_to_normalform("./src/datagen/2d_test_data.csv", scales).unwrap();

        let mut names = NameSet::new();
        names.insert("2d_test_data.csv".to_string());
        let expected = Term::Op(Op::Sequence {
            operations: vec![
                Term::Op(Op::Compose {
                    operations: vec![
                        Term::Op(Op::TransposeA {
                            a: Rational64::new(5, 1),
                        }),
                        Term::Op(Op::Length {
                            m: Rational64::new(1, 1),
                        }),
                    ],
                }),
                Term::Op(Op::Compose {
                    operations: vec![
                        Term::Op(Op::TransposeA {
                            a: Rational64::new(2, 1),
                        }),
                        Term::Op(Op::Length {
                            m: Rational64::new(2, 1),
                        }),
                    ],
                }),
                Term::Op(Op::Compose {
                    operations: vec![
                        Term::Op(Op::TransposeA {
                            a: Rational64::new(3, 1),
                        }),
                        Term::Op(Op::Length {
                            m: Rational64::new(2, 1),
                        }),
                    ],
                }),
            ],
        });
        assert_eq!(result, expected);
    }
}
