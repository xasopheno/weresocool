#[cfg(test)]
mod eeg_test {
    use num_rational::Rational64;

    use crate::{
        datagen::{csv_to_normalform, eeg_data_to_normal_form, eeg_datum_to_point_op, CsvData},
        NameSet, NormalForm, PointOp,
    };
    #[test]
    fn test_eeg_datum_to_point_op() {
        let mut names = NameSet::new();
        names.insert("data.csv".to_string());
        let result = eeg_datum_to_point_op(1.0e-14, None, 2.0e14, "data.csv");
        let expected = PointOp {
            fa: Rational64::new(2, 1),
            l: Rational64::new(1, 50),
            names,
            ..PointOp::default()
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_eeg_datum_to_normal_form() {
        let eeg_data = CsvData {
            data: vec![0.5e-14, 1.0e-14, 1.5e-14],
        };
        let mut names = NameSet::new();
        names.insert("data.csv".to_string());
        let result = eeg_data_to_normal_form(&eeg_data, 2.0e14, "data.csv");
        let expected = NormalForm {
            operations: vec![vec![
                PointOp {
                    fa: Rational64::new(1, 1),
                    l: Rational64::new(1, 50),
                    names: names.clone(),
                    ..PointOp::default()
                },
                PointOp {
                    fa: Rational64::new(3, 2),
                    l: Rational64::new(1, 50),
                    names: names.clone(),
                    ..PointOp::default()
                },
                PointOp {
                    fa: Rational64::new(2, 1),
                    l: Rational64::new(1, 50),
                    names,
                    ..PointOp::default()
                },
            ]],
            length_ratio: Rational64::new(3, 50),
        };
        assert_eq!(result, expected);
    }
    #[test]
    fn test_csv_to_normalform() {
        let result = csv_to_normalform(
            "./src/datagen/test_data.csv",
            Some(Rational64::new(200_000_000_000_000, 1)),
        )
        .unwrap();

        let mut names = NameSet::new();
        names.insert("test_data.csv".to_string());
        let expected = NormalForm {
            operations: vec![vec![
                PointOp {
                    fa: Rational64::new(1, 1),
                    l: Rational64::new(1, 50),
                    names: names.clone(),
                    ..PointOp::default()
                },
                PointOp {
                    fa: Rational64::new(3, 2),
                    l: Rational64::new(1, 50),
                    names: names.clone(),
                    ..PointOp::default()
                },
                PointOp {
                    fa: Rational64::new(2, 1),
                    l: Rational64::new(1, 50),
                    names,
                    ..PointOp::default()
                },
            ]],
            length_ratio: Rational64::new(3, 50),
        };
        assert_eq!(result, expected);
    }
}
