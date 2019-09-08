use weresocool::generation::{filename_to_render, RenderReturn, RenderType};
use error::Error;

struct CsvOp {

}

fn main() -> Result<(), Error> {
    println!("Hello Scratch Pad");
    let filename = "songs/template.socool";

    match filename_to_render(filename, RenderType::NfBasisAndTable)? {
        RenderReturn::NfBasisAndTable(nf, _basis, _table) => {
            for (i, voice) in nf.operations.iter().enumerate() {
                dbg!(i);
                for (j, op) in voice.iter().enumerate() {
                    dbg!(j, op);

                }
            }
        }
        _ => panic!()
    };
            
    Ok(())
}

#[test]
fn test_decimal_to_fraction() {
    assert!(true, true);
}
