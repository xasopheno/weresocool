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
                let arg_term = arg_map.get(name);
                let term = match arg_term {
                    Some(arg_term) => arg_term.clone(),
                    None => handle_id_error(name.to_string(), defs),
                };

                match term {
                    Term::Lop(lop) => lop.substitute(normal_form, defs, arg_map),
                    _ => unimplemented!(),
                }
            }
            ListOp::ListOpIndexed {
                list_op: _,
                indices: _,
            } => unimplemented!(),
        }
    }
}
