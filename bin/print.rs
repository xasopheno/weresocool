use rayon::prelude::*;
use weresocool::{
    generation::parsed_to_render::{sum_all_waveforms, RenderReturn, RenderType},
    instrument::StereoWaveform,
    interpretable::{InputType::Filename, Interpretable},
    renderable::{nf_to_vec_renderable, renderables_to_render_voices},
    settings::{default_settings, Settings},
    ui::{get_args, no_file_name, were_so_cool_logo},
    write::write_composition_to_wav,
};

const SETTINGS: Settings = default_settings();

use failure::Fail;
use weresocool_error::Error;

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

fn run() -> Result<(), Error> {
    were_so_cool_logo();
    println!("       )))***=== Printing Cool Sounds ===***(((  \n ");
    let args = get_args();

    let filename = args.value_of("filename");
    match filename {
        Some(_filename) => {}
        _ => no_file_name(),
    }

    let (nf, basis, table) = match Filename(filename.unwrap()).make(RenderType::NfBasisAndTable)? {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => panic!("Error. Unable to generate NormalForm"),
    };
    let renderables = nf_to_vec_renderable(&nf, &table, &basis)?;
    let mut voices = renderables_to_render_voices(renderables);

    let mut result = StereoWaveform::new(0);
    loop {
        let batch: Vec<StereoWaveform> = voices
            .par_iter_mut()
            .filter_map(|voice| voice.render_batch(SETTINGS.buffer_size, None))
            .collect();

        if !batch.is_empty() {
            let stereo_waveform = sum_all_waveforms(batch);
            result.append(stereo_waveform);
        } else {
            break;
        }
    }
    write_composition_to_wav(result, "");

    Ok(())
}
