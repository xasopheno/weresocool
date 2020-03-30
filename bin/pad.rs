//use weresocool::ui::were_so_cool_logo;
//use std::sync::{Arc, Mutex};
use weresocool::{instrument::StereoWaveform, renderable::RenderVoice};

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
    Ok(())
}
