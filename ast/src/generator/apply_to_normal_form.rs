use crate::{
    generator::error_non_generator, handle_id_error, join_list_nf, Defs, GenOp, NormalForm,
    Normalize, Term,
};
use weresocool_error::Error;

impl Normalize for GenOp {
    fn apply_to_normal_form(&self, input: &mut NormalForm, defs: &Defs) -> Result<(), Error> {
        match self {
            GenOp::Named { name, seed } => {
                let term = handle_id_error(name.to_string(), defs, None)?;
                match term {
                    Term::Gen(gen) => {
                        gen.to_owned().set_seed(*seed);
                        gen.apply_to_normal_form(input, defs)
                    }
                    _ => Err(error_non_generator()),
                }
            }
            GenOp::Const { gen, seed } => {
                let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(*seed);
                *input = join_list_nf(gen.to_owned().generate(
                    input,
                    gen.lcm_length(),
                    defs,
                    &mut rng,
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
