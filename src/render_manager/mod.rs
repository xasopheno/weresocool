//#![allow(dead_code, unused_imports, unused_variables)]
use crate::{
    generation::parsed_to_render::{RenderReturn, RenderType},
    instrument::StereoWaveform,
    interpretable::{InputType, Interpretable},
    renderable::{nf_to_vec_renderable, renderables_to_render_voices, RenderVoice},
};
use rayon::prelude::*;

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
    renderer_write_idx: usize,
    buffer_idx: usize,
    write_idx: usize,
    read_idx: usize,
}

impl RenderManager {
    pub const fn init(render_voices: Vec<RenderVoice>) -> Self {
        Self {
            renders: [Some(render_voices), None],
            render_idx: 0,
            read_idx: 0,
        }
    }

    pub const fn init_silent() -> Self {
        Self {
            renders: [None, None],
            render_idx: 0,
            read_idx: 0,
        }
    }

    pub fn render_batch(&mut self, n_samples: usize) -> Option<Vec<StereoWaveform>> {
        match self.current_render() {
            Some(render) => Some(
                render
                    .par_iter_mut()
                    .filter_map(|voice| voice.render_batch(n_samples, None))
                    .collect(),
            ),
            None => None,
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
        self.next_render().is_some()
    }
    pub fn push_render(&mut self, render: Vec<RenderVoice>) {
        *self.next_render() = Some(render);
        *self.current_render() = None;
        self.inc_render();
    }

    pub fn prepare_render(&mut self, input: InputType<'_>) -> Result<(), Error> {
        let (nf, basis, table) = match input.make(RenderType::NfBasisAndTable)? {
            RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
            _ => panic!("Error. Unable to generate NormalForm"),
        };
        let renderables = nf_to_vec_renderable(&nf, &table, &basis);

        let render_voices = renderables_to_render_voices(renderables);

        self.push_render(render_voices);

        Ok(())
    }
}

impl BufferManager {
    pub const fn init_silent() -> Self {
        Self {
            buffers: [None, None],
            renderer_write_idx: 0,
            buffer_idx: 0,
            write_idx: 0,
            read_idx: 0,
        }
    }

    pub fn inc_buffer(&mut self) {
        self.buffer_idx = (self.buffer_idx + 1) % 2;
    }

    pub fn inc_render_write_buffer(&mut self) {
        self.renderer_write_idx = (self.renderer_write_idx + 1) % 2;
    }

    pub fn current_buffer(&mut self) -> &mut Option<Buffer> {
        &mut self.buffers[self.buffer_idx]
    }

    pub fn current_render_write_buffer(&mut self) -> &mut Option<Buffer> {
        &mut self.buffers[self.renderer_write_idx]
    }

    pub fn next_buffer(&mut self) -> &mut Option<Buffer> {
        &mut self.buffers[(self.buffer_idx + 1) % 2]
    }

    pub fn exists_new_buffer(&mut self) -> bool {
        self.next_buffer().is_some()
    }

    pub fn read(&mut self, buffer_size: usize) -> Option<StereoWaveform> {
        let next = self.exists_new_buffer();
        let current = &mut self.buffers[self.buffer_idx];

        match current {
            Some(buffer) => {
                let mut sw = buffer.read(buffer_size);

                if next {
                    if let Some(s) = sw.as_mut() {
                        s.fade_out()
                    }

                    *current = None;
                    self.inc_buffer();
                }
                sw
            }
            None => {
                if next {
                    self.inc_buffer();
                    self.read(buffer_size)
                } else {
                    None
                }
            }
        }
    }

    pub fn write(&mut self, stereo_waveform: StereoWaveform) {
        let current = self.current_render_write_buffer();
        match current {
            Some(buffer) => buffer.write(stereo_waveform),
            None => {
                let mut new_buffer = Buffer::init();
                new_buffer.write(stereo_waveform);
                *current = Some(new_buffer);
            }
        }
    }
}
