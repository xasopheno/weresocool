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

#[cfg(test)]
mod render_manager_tests {
    use super::*;
    use crate::renderable::RenderOp;
    #[test]
    fn test_inc_render() {
        let mut r = RenderManager::init_silent();
        r.inc_render();
        assert_eq!(r.render_idx, 1);
        r.inc_render();
        assert_eq!(r.render_idx, 0);
    }

    fn render_voices_mock() -> Vec<RenderVoice> {
        vec![RenderVoice::init(&[RenderOp::init_silent_with_length(1.0)])]
    }

    #[test]
    fn test_push_render() {
        let mut r = RenderManager::init(render_voices_mock());
        r.push_render(render_voices_mock());
        assert_eq!(*r.current_render(), Some(render_voices_mock()));
        assert_eq!(*r.next_render(), None);
    }
}
