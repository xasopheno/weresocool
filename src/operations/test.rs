//pub mod tests {
//    use operations::{Op, Operate};
//    use ratios::{Pan, R};
//
//    #[test]
//    fn get_length_test() {
//        let rs = r![(1, 1, 0.0, 1.0, 0.0),];
//
//        let as_is = Op::AsIs {};
//        let transpose = Op::Transpose { m: 1.5, a: 1.0 };
//        let gain = Op::Gain { m: 1.5 };
//        let ratios = Op::Ratios { ratios: rs.clone() };
//        let silence = Op::Silence { m: 1.5 };
//        let length = Op::Length { m: 1.5 };
//
//        assert_eq!(as_is.get_length_ratio(), 1.0);
//        assert_eq!(transpose.get_length_ratio(), 1.0);
//        assert_eq!(gain.get_length_ratio(), 1.0);
//        assert_eq!(ratios.get_length_ratio(), 1.0);
//        assert_eq!(silence.get_length_ratio(), 1.5);
//        assert_eq!(length.get_length_ratio(), 1.5);
//
//
//        let sequence1 = Op::Sequence {
//            operations: vec![
//                Op::AsIs,
//                Op::Transpose { m: 2.0, a: 0.0 },
//                Op::Ratios { ratios: rs.clone() },
//                Op::Length { m: 2.0 },
//            ],
//        };
//
//        let length1 = sequence1.get_length_ratio();
//        assert_eq!(length1, 5.0);
//
//        let sequence2 = Op::Compose {
//            operations: vec![sequence1.clone(), Op::Length { m: 2.0 }],
//        };
//
//        let length2 = sequence2.clone().get_length_ratio();
//        assert_eq!(length2, 10.0);
//
//        let sequence3 = Op::Compose {
//            operations: vec![sequence1.clone(), sequence2.clone()],
//        };
//
//        let sequence_with_sequence_length = sequence3.clone().get_length_ratio();
//
//        assert_eq!(sequence_with_sequence_length, 50.0);
//
//        let fit = Op::Fit {
//            with_length_of: Box::new(sequence1.clone()),
//            main: Box::new(sequence3.clone())
//        };
//
//        let fit_length = fit.get_length_ratio();
//
//        assert_eq!(fit_length, 5.0);
//
//    }
//}
