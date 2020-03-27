use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;
use weresocool::{
    generation::parsed_to_render::{sum_all_waveforms, RenderReturn, RenderType},
    instrument::StereoWaveform,
    interpretable::{InputType::Filename, Interpretable},
    portaudio::{real_time_buffer, RealTimeRender},
    renderable::{nf_to_vec_renderable, renderables_to_render_voices},
    settings::{default_settings, Settings},
    ui::{get_args, no_file_name, were_so_cool_logo},
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
    println!("       )))***=== REAL<COOL>TIME *buffered ===***(((  \n ");

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
    let renderables = nf_to_vec_renderable(&nf, &table, &basis);
    let mut voices = renderables_to_render_voices(renderables);

    let rtr = Arc::new(Mutex::new(RealTimeRender::init()));
    let rtr_clone = Arc::clone(&rtr);

    thread::spawn(move || loop {
        let batch: Vec<StereoWaveform> = voices
            .par_iter_mut()
            .filter_map(|voice| voice.render_batch(SETTINGS.buffer_size, None))
            .collect();

        if batch.len() > 0 {
            let stereo_waveform = sum_all_waveforms(batch);
            rtr_clone.lock().unwrap().write(stereo_waveform);
        } else {
            break;
        }
    });

    let mut stream = real_time_buffer(Arc::clone(&rtr))?;
    stream.start()?;

    while let true = stream.is_active()? {}

    stream.stop()?;

    Ok(())
}
