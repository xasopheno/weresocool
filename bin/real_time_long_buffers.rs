use std::sync::{Arc, Mutex};
use std::thread;
use weresocool::{
    generation::parsed_to_render::{sum_all_waveforms, RenderReturn, RenderType},
    instrument::StereoWaveform,
    interpretable::{InputType::Filename, Interpretable},
    manager::{BufferManager, RenderManager},
    portaudio::real_time_managed_long,
    renderable::{nf_to_vec_renderable, renderables_to_render_voices},
    settings::{default_settings, Settings},
    ui::{get_args, no_file_name, were_so_cool_logo},
};

use failure::Fail;
use weresocool_error::Error;

const SETTINGS: Settings = default_settings();

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
    let render_voices = renderables_to_render_voices(renderables);

    let render_manager = Arc::new(Mutex::new(RenderManager::init(render_voices)));
    //let buffer_manager = Arc::new(Mutex::new(BufferManager::init_silent()));
    //let buffer_manager_clone = Arc::clone(&buffer_manager);

    //thread::spawn(move || loop {
    //let batch: Option<Vec<StereoWaveform>> = render_manager.render_batch(SETTINGS.buffer_size);

    //if let Some(b) = batch {
    //if !b.is_empty() {
    //let stereo_waveform = sum_all_waveforms(b);
    //buffer_manager_clone.lock().unwrap().write(stereo_waveform);
    //} else {
    //break;
    //}
    //}
    //});

    let mut stream = real_time_managed_long(Arc::clone(&render_manager))?;
    stream.start()?;

    while let true = stream.is_active()? {}

    stream.stop()?;

    Ok(())
}
