use crate::{
    generator::error_non_generator, handle_id_error, join_list_nf, GenOp, NormalForm, Normalize,
    Term,
};
use rand::SeedableRng;
use scop::Defs;
use weresocool_error::Error;

impl Normalize<Term> for GenOp {
    fn apply_to_normal_form(&self, input: &mut NormalForm, defs: &Defs<Term>) -> Result<(), Error> {
        match self {
            GenOp::Named { name, seed } => {
                let term = handle_id_error(name, defs)?;
                match term {
                    Term::Gen(gen) => {
                        gen.to_owned().set_seed(*seed);
                        gen.apply_to_normal_form(input, defs)
                    }
                    _ => Err(error_non_generator()),
                }
            }
            GenOp::Const { gen, seed } => {
                *input = join_list_nf(gen.to_owned().generate(
                    input,
                    gen.lcm_length(),
                    defs,
                    &mut SeedableRng::seed_from_u64(*seed),
                )?);
                Ok(())
            }
            GenOp::Taken { n, gen, seed } => {
                gen.to_owned().set_seed(*seed);
                *input = join_list_nf(gen.to_owned().generate_from_genop(input, Some(*n), defs)?);
                Ok(())
            }
        }
    }
}
