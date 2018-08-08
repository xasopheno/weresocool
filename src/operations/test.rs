pub mod tests {
    use event::Event;
    use operations::{Op, Operate};
    use ratios::{Pan, R};
    use settings::get_test_settings;
    #[test]
    fn get_length_test() {
        let rs = r![
        (1, 1, 0.0, 1.0, 0.0),
    ];

        let e = vec![Event::new(120.0, rs.clone(), 1.0, 1.0)];
        let sequence1 = Op::Sequence {
            operations: vec![
                Op::AsIs, //1
                Op::Transpose {m: 2.0, a: 0.0}, //1
                Op::Ratios {ratios: rs.clone()}, //1
                Op::Length {m: 2.0}, //2
            ],
        };

        let mut length1 = sequence1.get_length_ratio();
        assert_eq!(5.0, length1);

        let sequence2 = Op::Compose {
            operations: vec![
                sequence1.clone(),
                Op::Length {m: 2.0}
             ],
        };//10

        let mut length2 = sequence2.clone().get_length_ratio();
        assert_eq!(10.0, length2);

        let sequence3 = Op::Compose {
            operations: vec![
                sequence1.clone(),
                sequence2.clone(),
            ] //40
        };

        let mut length3 = sequence3.clone().get_length_ratio();

        assert_eq!(50.0, length3);
    }
}
