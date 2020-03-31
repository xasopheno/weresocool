#![allow(dead_code, unused_imports, unused_variables)]
//use weresocool::ui::were_so_cool_logo;
//use std::sync::{Arc, Mutex};

use crate::{
    generation::parsed_to_render::{sum_all_waveforms, RenderReturn, RenderType},
    instrument::StereoWaveform,
    interpretable::{InputType::Filename, Interpretable},
    portaudio::{real_time_buffer, RealTimeRender},
    renderable::{nf_to_vec_renderable, renderables_to_render_voices, RenderVoice},
    settings::{default_settings, Settings},
    ui::{get_args, no_file_name, were_so_cool_logo},
};
use rayon::prelude::*;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use failure::Fail;
use weresocool_error::Error;

#[derive(Clone, Debug)]
pub struct Buffer {
    pub stereo_waveform: StereoWaveform,
    pub write_idx: usize,
    pub read_idx: usize,
}

/// This assumes that all buffers are the same size
impl Buffer {
    pub fn init() -> Self {
        Self {
            stereo_waveform: StereoWaveform::new(0),
            write_idx: 0,
            read_idx: 0,
        }
    }
    pub fn write(&mut self, stereo_waveform: StereoWaveform) {
        self.stereo_waveform.append(stereo_waveform);
        self.write_idx += 1;
    }

    pub fn read(&mut self, buffer_size: usize) -> Option<StereoWaveform> {
        let sw = self.stereo_waveform.get_buffer(self.read_idx, buffer_size);
        if sw.is_some() {
            self.read_idx += 1;
        };
        sw
    }
}

#[derive(Clone, Debug)]
pub struct RenderManager {
    pub renders: [Option<Vec<RenderVoice>>; 2],
    render_idx: usize,
    read_idx: usize,
}

#[derive(Clone, Debug)]
pub struct BufferManager {
    pub buffers: [Option<Buffer>; 2],
    buffer_idx: usize,
    write_idx: usize,
    read_idx: usize,
}

impl RenderManager {
    pub fn init(render_voices: Vec<RenderVoice>) -> Self {
        Self {
            renders: [Some(render_voices), None],
            render_idx: 0,
            read_idx: 0,
        }
    }

    pub fn render_batch(&mut self, n_samples: usize) -> Vec<StereoWaveform> {
        match self.current_render() {
            Some(render) => render
                .par_iter_mut()
                .filter_map(|voice| voice.render_batch(n_samples, None))
                .collect(),
            None => [].to_vec(),
        }
    }

    pub fn inc_render(&mut self) {
        self.render_idx = (self.render_idx + 1) % 2;
    }

    pub fn current_render(&mut self) -> &mut Option<Vec<RenderVoice>> {
        &mut self.renders[self.render_idx]
    }

    pub fn next_render(&mut self) -> &mut Option<Vec<RenderVoice>> {
        &mut self.renders[(self.render_idx + 1) % 2]
    }

    pub fn exists_new_render(&mut self) -> bool {
        match self.next_render() {
            Some(_) => true,
            None => false,
        }
    }
    pub fn push_render(&mut self, render: Vec<RenderVoice>) {
        *self.next_render() = Some(render);
        self.inc_render();
    }

    pub async fn prepare_render(&mut self, language: &str) -> Result<(), Error> {
        thread::spawn(move || {});

        unimplemented!();
    }
}

impl BufferManager {
    pub fn init() -> Self {
        Self {
            buffers: [Some(Buffer::init()), None],
            buffer_idx: 0,
            write_idx: 0,
            read_idx: 0,
        }
    }

    pub fn inc_buffer(&mut self) {
        self.buffer_idx = (self.buffer_idx + 1) % 2;
    }

    pub fn current_buffer(&mut self) -> &mut Option<Buffer> {
        &mut self.buffers[self.buffer_idx]
    }

    pub fn next_buffer(&mut self) -> &mut Option<Buffer> {
        &mut self.buffers[(self.buffer_idx + 1) % 2]
    }

    pub fn exists_new_buffer(&mut self) -> bool {
        match self.next_buffer() {
            Some(_) => true,
            None => false,
        }
    }

    pub fn read(&mut self, buffer_size: usize) -> Option<StereoWaveform> {
        let current = self.current_buffer();
        match current {
            Some(buffer) => {
                //println!("read_idx {}", buffer.read_idx);
                //println!("______write_idx {}", buffer.write_idx);
                //
                let mut sw = buffer.read(buffer_size);
                if self.exists_new_buffer() {
                    if let Some(s) = sw.as_mut() {
                        s.fade_out()
                    }

                    self.inc_buffer();
                }
                sw
            }
            None => None,
        }
    }

    pub fn write(&mut self, stereo_waveform: StereoWaveform) {
        let current = self.current_buffer();
        match current {
            Some(buffer) => buffer.write(stereo_waveform),
            None => {}
        }
    }

    pub fn fade_stereowaveform(sw: &mut StereoWaveform) {
        unimplemented!()
    }
}
