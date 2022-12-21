#[cfg(test)]
mod test_csv_gen {
    use hamcrest2::prelude::*;
    use num_rational::Rational64;

    use crate::{datagen::*, NameSet, NormalForm, PointOp};

    #[test]
    fn test_get_data_2d() {
        let result = get_data("./src/datagen/2d_test_data.csv".to_string()).unwrap();

        let expected = vec![vec![2.5, 1.0], vec![1.0, 2.0], vec![1.5, 2.0]];

        assert_that!(&result, contains(expected).exactly());
    }

    #[test]
    fn test_point_to_point_op() {
        let mut names = NameSet::new();
        names.insert("2d_test_data.csv".to_string());
        let result = point_to_point_op(
            &vec![1.0, 2.0],
            // Buffers { fa: None, lm: None },
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
        let expected = PointOp {
            fa: Rational64::new(2, 1),
            l: Rational64::new(1, 1),
            names,
            ..PointOp::default()
        };
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
        let expected = NormalForm {
            operations: vec![vec![
                PointOp {
                    fa: Rational64::new(2, 1),
                    l: Rational64::new(1, 2),
                    names: names.clone(),
                    ..PointOp::default()
                },
                PointOp {
                    fa: Rational64::new(2, 1),
                    l: Rational64::new(1, 2),
                    names: names.clone(),
                    ..PointOp::default()
                },
                PointOp {
                    fa: Rational64::new(2, 1),
                    l: Rational64::new(1, 2),
                    names,
                    ..PointOp::default()
                },
            ]],
            length_ratio: Rational64::new(3, 2),
        };
        assert_eq!(result, expected);
    }
    #[test]

    fn test_csv_to_normalform() {
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

        let result = csv_to_normalform("./src/datagen/2d_test_data.csv", scales).unwrap();

        let mut names = NameSet::new();
        names.insert("2d_test_data.csv".to_string());
        let expected = NormalForm {
            operations: vec![vec![
                PointOp {
                    fa: Rational64::new(5, 1),
                    l: Rational64::new(1, 2),
                    names: names.clone(),
                    ..PointOp::default()
                },
                PointOp {
                    fa: Rational64::new(2, 1),
                    l: Rational64::new(1, 1),
                    names: names.clone(),
                    ..PointOp::default()
                },
                PointOp {
                    fa: Rational64::new(3, 1),
                    l: Rational64::new(1, 1),
                    names,
                    ..PointOp::default()
                },
            ]],
            length_ratio: Rational64::new(5, 2),
        };
        assert_eq!(result, expected);
    }
}
