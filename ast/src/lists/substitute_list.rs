use crate::operations::{helpers::handle_id_error, NormalForm, Normalize, Substitute};
use crate::substitute_operations;
use crate::{ListOp, Term};
use scop::Defs;
use weresocool_error::Error;

impl Substitute<Term> for ListOp {
    fn substitute(
        &self,
        normal_form: &mut NormalForm,
        defs: &mut Defs<Term>,
    ) -> Result<Term, Error> {
        match self {
            ListOp::Const { terms } => Ok(Term::Lop(ListOp::Const {
                terms: substitute_operations(terms.to_vec(), normal_form, defs)?,
            })),
            ListOp::Named { name } => {
                let term = handle_id_error(name, defs)?;

                match term {
                    Term::Lop(lop) => lop.substitute(normal_form, defs),
                    _ => Err(Error::with_msg("List.substitute() on called non-list")),
                }
            }
            ListOp::ListOpIndexed { .. } => Ok(Term::Lop(ListOp::Const {
                terms: self
                    .term_vectors(defs)?
                    .iter_mut()
                    .map(|term_vector| {
                        let mut nf = normal_form.clone();

                        term_vector.term.apply_to_normal_form(&mut nf, defs)?;
                        term_vector.index_terms.iter().try_for_each(|index_term| {
                            index_term.apply_to_normal_form(&mut nf, defs)
                        })?;

                        Ok(Term::Nf(nf))
                    })
                    .collect::<Result<Vec<Term>, Error>>()?,
            })),
            ListOp::Concat { listops } => {
                let mut result = vec![];
                for list in listops {
                    result.push(list.substitute(normal_form, defs)?)
                }

                Ok(Term::Lop(ListOp::Const { terms: result }))
            }
            ListOp::GenOp { gen, .. } => gen.substitute(normal_form, defs),
        }
    }
}
