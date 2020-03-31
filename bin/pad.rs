#![allow(dead_code, unused_imports, unused_variables)]
//use weresocool::ui::were_so_cool_logo;
//use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use weresocool::{
    generation::parsed_to_render::{sum_all_waveforms, RenderReturn, RenderType},
    instrument::StereoWaveform,
    interpretable::{InputType::Filename, Interpretable},
    portaudio::real_time_managed::real_time_managed,
    render_manager::{BufferManager, RenderManager},
    renderable::{nf_to_vec_renderable, renderables_to_render_voices, RenderVoice},
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

    //let args = get_args();

    //let filename = args.value_of("filename");
    let filename1 = "songs/test_2/manager_1.socool";
    let filename2 = "songs/test_2/manager_2.socool";
    //match filename {
    //Some(_filename) => {}
    //_ => no_file_name(),
    //}

    let (nf1, basis1, table1) = match Filename(filename1).make(RenderType::NfBasisAndTable)? {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => panic!("Error. Unable to generate NormalForm"),
    };

    let (nf2, basis2, table2) = match Filename(filename2).make(RenderType::NfBasisAndTable)? {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => panic!("Error. Unable to generate NormalForm"),
    };

    let renderables1 = nf_to_vec_renderable(&nf1, &table1, &basis1);
    let renderables2 = nf_to_vec_renderable(&nf2, &table2, &basis2);

    let render_voices1 = renderables_to_render_voices(renderables1);
    let render_voices2 = renderables_to_render_voices(renderables2);

    let buffer_manager = Arc::new(Mutex::new(BufferManager::init()));
    let buffer_manager_clone = Arc::clone(&buffer_manager);
    //let voices = Arc::new(Mutex::new(render_voices));

    //let rtr = Arc::new(Mutex::new(RealTimeRender::init()));
    //let rtr_clone = Arc::clone(&rtr);

    let (send, recv) = channel();
    println!("Start...");
    thread::Builder::new()
        .name("sender".to_string())
        .spawn(move || {
            for i in 0..3 {
                thread::sleep(Duration::from_secs(1));
                send.send(render_voices2.clone()).unwrap();
            }
        })
        .unwrap();

    thread::Builder::new()
        .name("receiver".to_string())
        .spawn(move || {
            let mut render_manager = RenderManager::init(render_voices1);
            loop {
                if let Ok(v) = recv.try_recv() {
                    println!("{:?}", &v.len());

                    render_manager.push_render(v);
                };

                let batch: Vec<StereoWaveform> = render_manager.render_batch(SETTINGS.buffer_size);
                if !batch.is_empty() {
                    let stereo_waveform = sum_all_waveforms(batch);
                    buffer_manager_clone.lock().unwrap().write(stereo_waveform);
                }
            }
        })
        .unwrap();

    let mut stream = real_time_managed(Arc::clone(&buffer_manager))?;
    stream.start()?;

    while let true = stream.is_active()? {}

    stream.stop()?;

    Ok(())
}
