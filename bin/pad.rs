use weresocool::{
    generation::{RenderReturn, RenderType},
    interpretable::{InputType::Filename, Interpretable},
    ui::were_so_cool_logo,
};
// use num_rational::Rational64;
// use weresocool_ast::{NormalForm, PointOp};

use weresocool_error::Error;

fn main() -> Result<(), Error> {
    were_so_cool_logo();
    let filename = "songs/template_1.socool";

    let render_return = Filename(filename).make(RenderType::NfBasisAndTable, None)?;
    let (mut nf, _basis, _table) = match render_return {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => panic!("huh"),
    };
    let result = nf.divide_into_n_equal_lengths(3);
    dbg!(result);
    Ok(())
}
