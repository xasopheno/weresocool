pub mod normalize_nf {
    extern crate num_rational;
    extern crate rand;

    use crate::ast::{OpTable, OpOrNf};
    use crate::operations::{GetLengthRatio, NormalForm, Normalize};
    use num_rational::Rational64;

    impl Normalize for OpOrNf {
        fn apply_to_normal_form(&self, input: &mut NormalForm, table: &OpTable) {
            match self {
               OpOrNf::Op(op) => {
                   op.apply_to_normal_form(input, table)
               },
               OpOrNf::Nf(nf) => {
                   nf.apply_to_normal_form(input, table)
               }
            }
        }
    }

    impl GetLengthRatio for OpOrNf {
        fn get_length_ratio(&self, table: &OpTable) -> Rational64 {
            match self {
                OpOrNf::Op(op) => {
                    op.get_length_ratio(table)
                },
                OpOrNf::Nf(nf) => {
                    nf.get_length_ratio(table)
                }
            }
        }
    }
}
