use weresocool::generation::{filename_to_render, RenderReturn, RenderType};
use error::Error;

fn main() -> Result<(), Error> {
    println!("Hello Scratch Pad");
    let filename = "songs/template.socool";
    let stereo_waveform =

    match filename_to_render(filename, RenderType::NfBasisAndTable)? {
        RenderReturn::NfBasisAndTable(nf, basis, table) => dbg!(nf),
        _ => panic!()
    };
            
    Ok(())
}

#[test]
fn test_decimal_to_fraction() {
    assert!(true, true);
}
