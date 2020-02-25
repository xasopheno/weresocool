use crate::operations::{helpers::handle_id_error, ArgMap, NormalForm, Substitute};
use crate::substitute_operations;
use crate::{Defs, ListOp, Term};

impl Substitute for ListOp {
    fn substitute(&self, normal_form: &mut NormalForm, defs: &Defs, arg_map: &ArgMap) -> Term {
        unimplemented!()
        //match self {
        //ListOp::IndexedNamedList { name, indices } => {
        //let value = arg_map.get(&name.clone());
        //let term = match value {
        //Some(sub) => sub.clone(),
        //None => handle_id_error(name.to_string(), defs),
        //};
        //match term {
        //Term::Lop(list_op) => match list_op {
        //ListOp::List(list) => {
        //let new_lop = ListOp::IndexedList {
        //terms: list,
        //indices: indices.clone(),
        //};
        //new_lop.substitute(normal_form, defs, arg_map)
        //}
        //_ => unimplemented!(),
        //},
        //_ => unimplemented!(),
        //}
        //}
        //_ => {
        //let vec_nf = self.to_list_nf(normal_form, defs);
        //let vec_terms = vec_nf.iter().map(|t| Term::Nf(t.clone())).collect();
        //let terms = substitute_operations(vec_terms, normal_form, defs, arg_map);
        //Term::Lop(ListOp::List(terms))
        //}
        //}
    }
}
