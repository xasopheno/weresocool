use num_rational::Rational64;
use weresocool::{
    generation::{RenderReturn, RenderType},
    interpretable::{InputType::Filename, Interpretable},
    ui::were_so_cool_logo,
};
use weresocool_ast::{NormalForm, PointOp};

use weresocool_error::Error;

fn main() -> Result<(), Error> {
    were_so_cool_logo();
    let filename = "songs/template_1.socool";

    let render_return = Filename(filename).make(RenderType::NfBasisAndTable, None)?;
    let (mut nf, _basis, _table) = match render_return {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => panic!("huh"),
    };

    let lengths = vec![
        Rational64::from_integer(2),
        Rational64::from_integer(1),
        Rational64::from_integer(1),
    ];

    let sum_lrs: Rational64 = lengths.iter().sum();
    let scaled_lrs: Vec<Rational64> = lengths
        .iter()
        .map(|l| l / sum_lrs * nf.length_ratio)
        .collect();
    // dbg!(&scaled_lrs);

    let result = nf.divide_into_parts(scaled_lrs);
    // dbg!(result);
    Ok(())
}
