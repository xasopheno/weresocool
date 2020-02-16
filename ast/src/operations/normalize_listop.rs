use crate::ast::{Defs, ListOp};
use crate::operations::{helpers::join_sequence, NormalForm, Normalize};

impl Normalize for ListOp {
    fn apply_to_normal_form(&self, input: &mut NormalForm, defs: &Defs) {
        match self {
            ListOp::List(operations) => {
                let mut result = NormalForm::init_empty();
                for op in operations {
                    let mut input_clone = input.clone();
                    op.apply_to_normal_form(&mut input_clone, defs);
                    result = join_sequence(result, input_clone);
                }

                *input = result
            }
        }
    }
}
