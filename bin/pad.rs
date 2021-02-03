// use std::convert::TryFrom;
use weresocool::{
    // generation::parsed_to_render::write_audio_to_file,
    generation::{RenderReturn, RenderType},
    interpretable::{InputType::Filename, Interpretable},
    ui::were_so_cool_logo,
    // write::write_composition_to_wav,
};

use weresocool_error::Error;

fn main() -> Result<(), Error> {
    were_so_cool_logo();
    let filename = "songs/template_1.socool";

    let render_return = Filename(filename).make(RenderType::Stems, None)?;
    let stems = match render_return {
        RenderReturn::Stems(stems) => stems,
        _ => panic!("huh"),
    };
    dbg!(stems.len());
    Ok(())
}
