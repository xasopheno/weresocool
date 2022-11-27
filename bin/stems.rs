use std::convert::TryFrom;
use weresocool_core::{
    generation::{generate_waveforms, RenderReturn, RenderType},
    interpretable::{InputType::Filename, Interpretable},
    write::write_composition_to_wav,
};

use weresocool_error::Error;
use weresocool_instrument::renderable::nf_to_vec_renderable;

fn main() -> Result<(), Error> {
    let filename = "test.socool";

    let render_return = Filename(filename).make(RenderType::NfBasisAndTable, None)?;
    let (nf, basis, mut table) = match render_return {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => panic!("huh"),
    };
    let renderables = nf_to_vec_renderable(&nf, &mut table, &basis)?;
    let vec_sw = generate_waveforms(renderables, true);
    for (i, sw) in vec_sw.iter().enumerate() {
        let f = format!("{}_{}", &filename, i);
        let f = f.split('/').collect::<Vec<&str>>();
        let _f = f[f.len() - 1];
        let render_return = RenderReturn::Wav(write_composition_to_wav(sw.clone())?);

        let _audio: Vec<u8> = Vec::try_from(render_return.clone())?;
        // write_audio_to_file(&audio, f, "wav")
    }

    Ok(())
}
