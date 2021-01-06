use crate::{
    generator::error_non_generator, handle_id_error, join_list_nf, Defs, GenOp, NormalForm,
    Normalize, Term,
};
use weresocool_error::Error;

impl Normalize for GenOp {
    fn apply_to_normal_form(&self, input: &mut NormalForm, defs: &Defs) -> Result<(), Error> {
        match self {
            GenOp::Named(name) => {
                let term = handle_id_error(name.to_string(), defs, None)?;
                match term {
                    Term::Gen(gen) => gen.apply_to_normal_form(input, defs),
                    _ => Err(error_non_generator()),
                }
            }
            GenOp::Const(gen) => {
                *input = join_list_nf(gen.to_owned().generate(input, gen.lcm_length(), defs)?);
                Ok(())
            }
            GenOp::Taken { n, gen } => {
                *input = join_list_nf(gen.to_owned().generate_from_genop(input, Some(*n), defs)?);
                Ok(())
            }
        }
    }
}
