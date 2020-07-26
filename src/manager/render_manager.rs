use crate::{
    generation::parsed_to_render::{RenderReturn, RenderType},
    generation::sum_all_waveforms,
    interpretable::{InputType, Interpretable},
};
use rayon::prelude::*;
use weresocool_error::Error;
use weresocool_instrument::renderable::{
    nf_to_vec_renderable, renderables_to_render_voices, RenderVoice,
};
use weresocool_instrument::StereoWaveform;

#[derive(Clone, Debug)]
pub struct RenderManager {
    pub renders: [Option<Vec<RenderVoice>>; 2],
    pub current_volume: f32,
    pub past_volume: f32,
    render_idx: usize,
    read_idx: usize,
}

impl RenderManager {
    pub const fn init(render_voices: Vec<RenderVoice>) -> Self {
        Self {
            renders: [Some(render_voices), None],
            past_volume: 1.0,
            current_volume: 1.0,
            render_idx: 0,
            read_idx: 0,
        }
    }

    pub const fn init_silent() -> Self {
        Self {
            renders: [None, None],
            past_volume: 0.0,
            current_volume: 0.0,
            render_idx: 0,
            read_idx: 0,
        }
    }

    pub fn read(&mut self, buffer_size: usize) -> Option<StereoWaveform> {
        let next = self.exists_next_render();
        let current = self.current_render();

        match current {
            Some(render_voices) => {
                let rendered: Vec<StereoWaveform> = render_voices
                    .par_iter_mut()
                    .filter_map(|voice| voice.render_batch(buffer_size, None))
                    .collect();
                if !rendered.is_empty() {
                    let mut sw: StereoWaveform = sum_all_waveforms(rendered);

                    if next {
                        sw.fade_out();

                        *current = None;
                        self.inc_render();
                    }

                    sw.pad(buffer_size);

                    Some(sw)
                } else {
                    *self.current_render() = None;
                    None
                }
            }
            None => {
                if next {
                    self.inc_render();
                    self.read(buffer_size)
                } else {
                    None
                }
            }
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

    pub fn exists_current_render(&mut self) -> bool {
        self.current_render().is_some()
    }

    pub fn exists_next_render(&mut self) -> bool {
        self.next_render().is_some()
    }

    pub fn push_render(&mut self, render: Vec<RenderVoice>) {
        *self.next_render() = Some(render);
    }

    pub fn prepare_render(&mut self, input: InputType<'_>) -> Result<(), Error> {
        let (nf, basis, table) = match input.make(RenderType::NfBasisAndTable)? {
            RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
            _ => return Err(Error::with_msg("Failed Parse/Render")),
        };
        let renderables = nf_to_vec_renderable(&nf, &table, &basis)?;

        let render_voices = renderables_to_render_voices(renderables);

        self.push_render(render_voices);
        Ok(())
    }
}

pub fn prepare_render_outside(input: InputType<'_>) -> Result<Vec<RenderVoice>, Error> {
    let (nf, basis, table) = match input.make(RenderType::NfBasisAndTable)? {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => return Err(Error::with_msg("Failed Parse/Render")),
    };
    let renderables = nf_to_vec_renderable(&nf, &table, &basis)?;

    let render_voices = renderables_to_render_voices(renderables);

    Ok(render_voices)
}

#[cfg(test)]
mod render_manager_tests {
    use super::*;
    use weresocool_instrument::renderable::RenderOp;
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
        assert_eq!(*r.current_render(), Some(render_voices_mock()));
        assert_eq!(*r.next_render(), None);
        r.push_render(render_voices_mock());
        assert_eq!(*r.current_render(), Some(render_voices_mock()));
        assert_eq!(*r.next_render(), Some(render_voices_mock()));
    }
}
