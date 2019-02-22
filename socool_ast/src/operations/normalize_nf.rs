pub mod normalize_nf {
    extern crate num_rational;
    extern crate rand;

    use crate::ast::{OpOrNf, OpOrNfTable};
    use crate::operations::{ArgMap, GetLengthRatio, NormalForm, Normalize, Substitute};
    use num_rational::Rational64;

    impl Normalize for OpOrNf {
        fn apply_to_normal_form(&self, input: &mut NormalForm, table: &OpOrNfTable) {
            match self {
                OpOrNf::Op(op) => op.apply_to_normal_form(input, table),
                OpOrNf::Nf(nf) => nf.apply_to_normal_form(input, table),
            }
        }
    }

    impl Substitute for OpOrNf {
        fn substitute(
            &self,
            normal_form: &mut NormalForm,
            table: &OpOrNfTable,
            arg_map: &ArgMap,
        ) -> OpOrNf {
            match self {
                OpOrNf::Op(op) => op.substitute(normal_form, table, arg_map),
                OpOrNf::Nf(nf) => nf.substitute(normal_form, table, arg_map),
            }
        }
    }

    impl GetLengthRatio for OpOrNf {
        fn get_length_ratio(&self, table: &OpOrNfTable) -> Rational64 {
            match self {
                OpOrNf::Op(op) => op.get_length_ratio(table),
                OpOrNf::Nf(nf) => nf.get_length_ratio(table),
            }
        }
    }
}
