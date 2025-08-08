use crate::{generator::error_non_generator, handle_id_error, operations::Defs, GenOp, NormalForm, Substitute, Term};
use weresocool_error::Error;

impl Substitute for GenOp {
    fn substitute(
        &self,
        _normal_form: &mut NormalForm,
        defs: &mut Defs,
    ) -> Result<Term, Error> {
        match self {
            GenOp::Named { name, seed } => {
                let term = handle_id_error(name, defs)?;
                match term {
                    Term::Gen(r#gen) => {
                        r#gen.to_owned().set_seed(*seed);
                        Ok(Term::Gen(r#gen))
                    }
                    _ => Err(error_non_generator()),
                }
            }
            GenOp::Const { .. } => Ok(Term::Gen(self.to_owned())),
            GenOp::Taken { n, generator, seed } => {
                let term = generator.substitute(_normal_form, defs)?;
                match term {
                    Term::Gen(gen_op) => Ok(Term::Gen(GenOp::Taken {
                        n: *n,
                        seed: *seed,
                        generator: Box::new(gen_op),
                    })),
                    _ => Ok(term),
                }
            }
        }
    }
}
