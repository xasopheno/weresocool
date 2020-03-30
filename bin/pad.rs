//use weresocool::ui::were_so_cool_logo;
//use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;
use weresocool::{
    generation::parsed_to_render::{sum_all_waveforms, RenderReturn, RenderType},
    instrument::StereoWaveform,
    interpretable::{InputType::Filename, Interpretable},
    portaudio::{real_time_buffer, RealTimeRender},
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

struct RenderManager {
    pub renders: [Option<Vec<RenderVoice>>; 2],
    render_idx: usize,
    read_idx: usize,
}

impl RenderManager {
    fn inc_render(&mut self) {
        self.render_idx = (self.render_idx + 1) % 2;
    }

    fn current_render(&mut self) -> &mut Option<Vec<RenderVoice>> {
        &mut self.renders[self.render_idx]
    }

    fn next_render(&mut self) -> &mut Option<Vec<RenderVoice>> {
        &mut self.renders[(self.render_idx + 1) % 2]
    }

    fn exists_new_render(&mut self) -> bool {
        match self.next_render() {
            Some(_) => true,
            None => false,
        }
    }
    fn push_render(&mut self, render: Vec<RenderVoice>) {
        *self.next_render() = Some(render);
    }

    fn prepare_render(&mut self, language: &str) -> Result<(), Error> {
        unimplemented!();
    }
}

struct BufferManager {
    pub buffers: [Option<StereoWaveform>; 2],
    buffer_idx: usize,
    write_idx: usize,
    read_idx: usize,
}

impl BufferManager {
    fn inc_buffer(&mut self) {
        self.buffer_idx = (self.buffer_idx + 1) % 2;
    }

    fn current_buffer(&mut self) -> &mut Option<StereoWaveform> {
        &mut self.buffers[self.buffer_idx]
    }

    fn next_buffer(&mut self) -> &mut Option<StereoWaveform> {
        &mut self.buffers[(self.buffer_idx + 1) % 2]
    }

    fn exists_new_buffer(&mut self) -> bool {
        match self.next_buffer() {
            Some(_) => true,
            None => false,
        }
    }

    fn fade_vec() -> Vec<f32> {
        (0..2048)
            .into_iter()
            .rev()
            .collect::<Vec<usize>>()
            .iter()
            .map(|s| *s as f32 / 2048.0)
            .collect()
    }

    fn fade_stereowaveform(sw: &mut StereoWaveform) {
        unimplemented!()
    }
}

fn run() -> Result<(), Error> {
    were_so_cool_logo();
    println!("       )))***=== REAL<COOL>TIME *buffered ===***(((  \n ");

    let args = get_args();

    //let filename = args.value_of("filename");
    let filename = args.value_of("filename");
    //match filename {
    //Some(_filename) => {}
    //_ => no_file_name(),
    //}

    let (nf, basis, table) = match Filename(filename.unwrap()).make(RenderType::NfBasisAndTable)? {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => panic!("Error. Unable to generate NormalForm"),
    };
    let renderables = nf_to_vec_renderable(&nf, &table, &basis);
    //let render_voices = renderables_to_render_voices(renderables);

    //let voices = Arc::new(Mutex::new(render_voices));

    //let rtr = Arc::new(Mutex::new(RealTimeRender::init()));
    //let rtr_clone = Arc::clone(&rtr);

    //thread::spawn(move || loop {
    //let batch: Vec<StereoWaveform> = voices
    //.lock()
    //.unwrap()
    //.par_iter_mut()
    //.filter_map(|voice| voice.render_batch(SETTINGS.buffer_size, None))
    //.collect();

    //if !batch.is_empty() {
    //let stereo_waveform = sum_all_waveforms(batch);
    //rtr_clone.lock().unwrap().write(stereo_waveform);
    //} else {
    //break;
    //}
    //});

    //let mut stream = real_time_buffer(Arc::clone(&rtr))?;
    //stream.start()?;

    //while let true = stream.is_active()? {}

    //stream.stop()?;

    Ok(())
}
