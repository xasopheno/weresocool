pub mod get_operations {
    use operations::GetOperations;
    use socool_parser::ast::Op;

    impl GetOperations for Op {
        fn get_operations(&self) -> Option<Vec<Op>> {
            match self {
                Op::AsIs {}
                | Op::Reverse {}
                | Op::TransposeM { m: _ }
                | Op::TransposeA { a: _ }
                | Op::PanA { a: _ }
                | Op::PanM { m: _ }
                | Op::Gain { m: _ } => None,

                Op::Length { m } | Op::Silence { m } => None,

                Op::Sequence { operations } => None,
                Op::Compose { operations } => None,

                Op::WithLengthRatioOf {
                    with_length_of: _,
                    main: _,
                } => None,

                Op::Overlay { operations } => { Result(operations) }
            }
        }
    }
}
