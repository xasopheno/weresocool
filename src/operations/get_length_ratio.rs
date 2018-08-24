pub mod get_length_ratio {
    use operations::{Apply, GetLengthRatio, Op};

    impl GetLengthRatio for Op {
        fn get_length_ratio(&self) -> f32 {
            match self {
                Op::AsIs {}
                | Op::Reverse {}
                | Op::TransposeM { m: _ }
                | Op::TransposeA { a: _ }
                | Op::PanA { a: _ }
                | Op::PanM { m: _ }
                | Op::Gain { m: _ } => 1.0,

                Op::Repeat { n, operations } => {
                    let mut length_ratio_of_operations = 0.0;
                    for operation in operations {
                        length_ratio_of_operations += operation.get_length_ratio();
                    }

                    length_ratio_of_operations * *n as f32
                }

                Op::Length { m } | Op::Silence { m } => *m,

                Op::Sequence { operations } => {
                    let mut new_total = 0.0;
                    for operation in operations {
                        new_total += operation.get_length_ratio();
                    }
                    new_total
                }
                Op::Compose { operations } => {
                    let mut new_total = 1.0;
                    for operation in operations {
                        new_total *= operation.get_length_ratio();
                    }
                    new_total
                }

                Op::ComposeWithOrder {
                    order_fn,
                    operations,
                } => {
                    let mut new_total = 1.0;
                    for operation in operations {
                        new_total *= operation.get_length_ratio();
                    }
                    new_total
                }

                Op::Fit {
                    with_length_of,
                    main: _,
                    n: _,
                } => with_length_of.get_length_ratio(),

                Op::Overlay { operations } => {
                    let mut max = 0.0;
                    for op in operations {
                        let next = op.get_length_ratio();
                        if next > max {
                            max = next;
                        }
                    }
                    max
                }
            }
        }
    }
}
