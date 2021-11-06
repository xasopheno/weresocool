#[cfg(test)]
mod eeg_test {
    use num_rational::Rational64;

    use crate::{
        datagen::{csv_to_normalform, eeg_data_to_normal_form, eeg_datum_to_point_op, EEGData},
        NameSet, NormalForm, PointOp,
    };
    #[test]
    fn test_eeg_datum_to_point_op() {
        let mut names = NameSet::new();
        names.insert(format!("eeg_{}", 1));
        let result = eeg_datum_to_point_op(0.01, 1, 200.0);
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
        let eeg_data = EEGData {
            data: vec![0.01, 0.02, 0.03],
        };
        let mut names = NameSet::new();
        names.insert(format!("eeg_{}", 1));
        let result = eeg_data_to_normal_form(&eeg_data, 100.0);
        let expected = NormalForm {
            operations: vec![vec![
                PointOp {
                    fa: Rational64::new(1, 1),
                    l: Rational64::new(1, 50),
                    names: names.clone(),
                    ..PointOp::default()
                },
                PointOp {
                    fa: Rational64::new(2, 1),
                    l: Rational64::new(1, 50),
                    names: names.clone(),
                    ..PointOp::default()
                },
                PointOp {
                    fa: Rational64::new(3, 1),
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
        let result = csv_to_normalform("./src/datagen/test_data.csv", 100.0);

        let mut names = NameSet::new();
        names.insert(format!("eeg_{}", 1));
        let expected = NormalForm {
            operations: vec![vec![
                PointOp {
                    fa: Rational64::new(1, 1),
                    l: Rational64::new(1, 50),
                    names: names.clone(),
                    ..PointOp::default()
                },
                PointOp {
                    fa: Rational64::new(2, 1),
                    l: Rational64::new(1, 50),
                    names: names.clone(),
                    ..PointOp::default()
                },
                PointOp {
                    fa: Rational64::new(3, 1),
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
