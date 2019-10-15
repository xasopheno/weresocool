use error::Error;
use failure::Fail;
//use num_rational::Rational64;
//use socool_ast::{NormalForm, PointOp};
use weresocool::generation::{
    filename_to_render, parsed_to_render::r_to_f64, renderable::nf_to_vec_renderable, RenderReturn,
    RenderType,
};

fn main() {
    match run() {
        Ok(_) => {}
        e => {
            for cause in Fail::iter_causes(&e.unwrap_err()) {
                println!("Failure caused by: {}", cause);
            }
        }
    }
}

#[allow(unused_variables)]
fn run() -> Result<(), Error> {
    let (nf, basis, table) =
        match filename_to_render("songs/test/live.socool", RenderType::NfBasisAndTable)? {
            RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
            _ => panic!("Error. Unable to generate NormalForm"),
        };
    let renderables = nf_to_vec_renderable(&nf, &table, &basis);
    let basis_f = r_to_f64(basis.f);

    Ok(())
}
