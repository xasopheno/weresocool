use crate::operations::{helpers::handle_id_error, ArgMap, NormalForm, Substitute};
use crate::substitute_operations;
use crate::{Defs, ListOp, Term};

impl Substitute for ListOp {
    fn substitute(&self, normal_form: &mut NormalForm, defs: &Defs, arg_map: &ArgMap) -> Term {
        match self {
            ListOp::Const(terms) => Term::Lop(ListOp::Const(substitute_operations(
                terms.to_vec(),
                normal_form,
                defs,
                arg_map,
            ))),
            ListOp::Named(name) => {
                let term = handle_id_error(name.to_string(), defs, Some(arg_map));

                match term {
                    Term::Lop(lop) => lop.substitute(normal_form, defs, arg_map),
                    _ => unimplemented!(),
                }
            }
            ListOp::ListOpIndexed { list_op, indices } => {
                let term_vectors = list_op.term_vectors(defs, Some(arg_map));
                //let index_vectors = indices.get_indices_and_terms(term_vectors.len());
                unimplemented!()
            }
        }
    }
}
