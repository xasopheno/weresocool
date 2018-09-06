macro_rules! r {
    ($(($num:expr,$den:expr,$offset:expr,$gain:expr,$pan:expr)),*$(,)*) => {
        Op::Overlay { operations: vec![$(
                Op::Compose { operations: vec! [
                    Op::TransposeM { m: $num as f32 /$den as f32 },
                    Op::TransposeA { a: $offset },
                    Op::Gain { m: $gain },
                    Op::PanA { a: $pan },
                ]},
            )*]
        }
    }
}

macro_rules! compose {
    ( $( $operation:expr ),*$(,)* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($operation);
            )*

            Op::Compose { operations: temp_vec }
        };
    };
}

macro_rules! sequence {
    ( $( $operation:expr ),*$(,)* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($operation);
            )*
            Op::Sequence { operations: temp_vec }
        };
    };
}

macro_rules! overlay {
    ( $( $operation:expr ),*$(,)* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($operation);
            )*
            Op::Overlay { operations: temp_vec }
        };
    };
}

macro_rules! fit {
    ($main:expr => $with_length_of:expr, $n:expr) => {
        Op::Fit {
            n: $n,
            with_length_of: Box::new($with_length_of),
            main: Box::new($main),
        }
    };
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
                        Op::PanA { a: 1.0 },
                    ],
                },
                Op::Compose {
                    operations: vec![
                        Op::TransposeM { m: 3.0 / 2.0 },
                        Op::TransposeA { a: 0.0 },
                        Op::Gain { m: 0.6 },
                        Op::PanA { a: -1.0 },
                    ],
                },
                Op::Compose {
                    operations: vec![
                        Op::TransposeM { m: 5.0 / 4.0 },
                        Op::TransposeA { a: 1.5 },
                        Op::Gain { m: 0.5 },
                        Op::PanA { a: 0.5 },
                    ],
                },
            ],
        };

        assert_eq!(r_macro, macro_test);
    }
}
