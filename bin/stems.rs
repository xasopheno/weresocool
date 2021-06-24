use std::convert::TryFrom;
use weresocool::{
    // generation::parsed_to_render::write_audio_to_file,
    generation::{generate_waveforms, RenderReturn, RenderType},
    interpretable::{InputType::Filename, Interpretable},
    ui::{get_args, no_file_name, were_so_cool_logo},
    write::write_composition_to_wav,
};

use weresocool_error::Error;
use weresocool_instrument::renderable::nf_to_vec_renderable;

fn main() -> Result<(), Error> {
    were_so_cool_logo();
    let args = get_args();

    let filename = args.value_of("filename");
    match filename {
        Some(_filename) => {}
        _ => no_file_name(),
    }

    let render_return = Filename(filename.unwrap()).make(RenderType::NfBasisAndTable, None)?;
    let (nf, basis, mut table) = match render_return {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => panic!("huh"),
    };
    let renderables = nf_to_vec_renderable(&nf, &mut table, &basis)?;
    let vec_sw = generate_waveforms(renderables, true);
    for (i, sw) in vec_sw.iter().enumerate() {
        let f = format!("{}_{}", &filename.unwrap(), i);
        let f = f.split('/').collect::<Vec<&str>>();
        let _f = f[f.len() - 1];
        let render_return = RenderReturn::Wav(write_composition_to_wav(sw.clone())?);

        let _audio: Vec<u8> = Vec::try_from(render_return.clone())?;
        // write_audio_to_file(&audio, f, "wav")
    }

    Ok(())
}
