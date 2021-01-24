use crate::{
    generator::error_non_generator, handle_id_error, ArgMap, Defs, GenOp, NormalForm, Substitute,
    Term,
};
use weresocool_error::Error;

impl Substitute for GenOp {
    fn substitute(
        &self,
        normal_form: &mut NormalForm,
        defs: &Defs,
        arg_map: &ArgMap,
    ) -> Result<Term, Error> {
        match self {
            GenOp::Named { name } => {
                let term = handle_id_error(name.to_string(), defs, Some(arg_map))?;
                match term {
                    Term::Gen(_) => Ok(term),
                    _ => Err(error_non_generator()),
                }
            }
            GenOp::Const { .. } => Ok(Term::Gen(self.to_owned())),
            GenOp::Taken { n, gen } => {
                let term = gen.substitute(normal_form, defs, arg_map)?;
                match term {
                    Term::Gen(gen) => Ok(Term::Gen(GenOp::Taken {
                        n: *n,
                        gen: Box::new(gen),
                    })),

                    _ => Err(error_non_generator()),
                }
            }
        }
    }
}
