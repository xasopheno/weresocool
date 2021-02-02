// use std::convert::TryFrom;
use weresocool::{
    // generation::parsed_to_render::write_audio_to_file,
    generation::{generate_waveforms, RenderReturn, RenderType},
    interpretable::{InputType::Filename, Interpretable},
    ui::{get_args, no_file_name, were_so_cool_logo},
    // write::write_composition_to_wav,
};

use weresocool_error::Error;
use weresocool_instrument::renderable::nf_to_vec_renderable;

fn main() -> Result<(), Error> {
    were_so_cool_logo();
    let args = get_args();

    // let filename = args.value_of("filename");
    // match filename {
    // Some(_filename) => {}
    // _ => no_file_name(),
    // }
    let filename = "songs/template_1.socool";

    let render_return = Filename(filename).make(RenderType::Stems, None)?;
    let stems = match render_return {
        RenderReturn::Stems(stems) => stems,
        _ => panic!("huh"),
    };
    dbg!(stems.len());
    unimplemented!();
    // let renderables = nf_to_vec_renderable(&nf, &table, &basis)?;
    // let vec_sw = generate_waveforms(renderables, true);
    // for (i, sw) in vec_sw.iter().enumerate() {
    // let f = format!("{}_{}", &filename.clone().unwrap(), i);
    // let f = f.split('/').collect::<Vec<&str>>();
    // let f = f[f.len() - 1];
    // let render_return = RenderReturn::Wav(write_composition_to_wav(sw.clone())?);

    // let audio: Vec<u8> = Vec::try_from(render_return.clone())?;
    // write_audio_to_file(&audio, f, "wav")
    // }

    Ok(())
}
