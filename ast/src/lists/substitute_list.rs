use crate::operations::{helpers::handle_id_error, ArgMap, NormalForm, Normalize, Substitute};
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
            ListOp::ListOpIndexed {
                list_op: _,
                indices: _,
            } => {
                let mut result = vec![];

                self.term_vectors(defs, Some(arg_map))
                    .iter_mut()
                    .for_each(|term_vector| {
                        let mut nf = normal_form.clone();
                        term_vector.term.apply_to_normal_form(&mut nf, defs);
                        term_vector
                            .index_terms
                            .iter()
                            .for_each(|index_term| index_term.apply_to_normal_form(&mut nf, defs));
                        result.push(Term::Nf(nf))
                    });
                Term::Lop(ListOp::Const(result))
            }
        }
    }
}
