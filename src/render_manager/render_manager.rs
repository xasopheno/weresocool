use crate::{
    generation::parsed_to_render::{RenderReturn, RenderType},
    instrument::StereoWaveform,
    interpretable::{InputType, Interpretable},
    renderable::{nf_to_vec_renderable, renderables_to_render_voices, RenderVoice},
};
use rayon::prelude::*;
use weresocool_error::Error;

#[derive(Clone, Debug)]
pub struct RenderManager {
    pub renders: [Option<Vec<RenderVoice>>; 2],
    render_idx: usize,
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
