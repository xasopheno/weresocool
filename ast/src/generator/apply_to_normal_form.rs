use crate::{
    generator::error_non_generator, handle_id_error, join_list_nf, operations::Defs, GenOp, NormalForm, Normalize, Term
};
use rand::SeedableRng;
use weresocool_error::Error;

impl Normalize for GenOp {
    fn apply_to_normal_form(
        &self,
        input: &mut NormalForm,
        defs: &mut Defs,
    ) -> Result<(), Error> {
        match self {
            GenOp::Named { name, seed } => {
                let term = handle_id_error(name, defs)?;
                match term {
                    Term::Gen(generator) => {
                        generator.to_owned().set_seed(*seed);
                        generator.apply_to_normal_form(input, defs)
                    }
                    _ => Err(error_non_generator()),
                }
            }
            GenOp::Const { generator, seed } => {
                *input = join_list_nf(generator.to_owned().generate(
                    input,
                    generator.lcm_length(),
                    defs,
                    &mut SeedableRng::seed_from_u64(*seed),
                )?);
                Ok(())
            }
            GenOp::Taken { n, generator, seed } => {
                generator.to_owned().set_seed(*seed);
                *input = join_list_nf(generator.to_owned().generate_from_genop(input, Some(*n), defs)?);
                Ok(())
            }
        }
    }
}
