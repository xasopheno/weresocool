use crate::{
    renderable::{Offset, RenderOp},
    sample::Waveform,
    {gain::gain_at_index, loudness::loudness_normalization},
};

use reverb::Reverb;
use weresocool_ast::{OscType, ASR};
use weresocool_filter::*;
use weresocool_shared::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Voice {
    pub reverb: ReverbState,
    pub index: usize,
    pub past: VoiceState,
    pub current: VoiceState,
    pub offset_past: VoiceState,
    pub offset_current: VoiceState,
    pub phase: f64,
    pub old_phase: f64,
    pub osc_type: OscType,
    pub old_osc_type: Option<OscType>,
    pub attack: usize,
    pub decay: usize,
    pub asr: ASR,
    pub filters: Option<Vec<BiquadFilter>>,
    pub old_filters: Option<Vec<BiquadFilter>>,
    pub filter_crossfade_index: usize,
    pub osc_crossfade_index: usize,
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct SampleInfo {
    pub frequency: f64,
    pub gain: f64,
}

#[derive(Clone, Debug, PartialEq)]
/// Stores state which allow for a 1-step look back at the previously rendered values.
pub struct VoiceState {
    pub frequency: f64,
    pub gain: f64,
    pub osc_type: OscType,
    pub reverb: Option<f64>,
}

impl VoiceState {
    pub const fn init() -> Self {
        Self {
            frequency: 0.0,
            gain: 0.0,
            osc_type: OscType::None,
            reverb: None,
        }
    }

    /// Check if the previous voice state was silent
    pub fn silent(&self) -> bool {
        self.frequency < Settings::global().min_freq || self.gain == 0.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReverbState {
    model: Reverb,
    state: Option<f64>,
}

impl ReverbState {
    pub fn init() -> Self {
        Self {
            model: Reverb::new(),
            state: None,
        }
    }
}

impl Voice {
    pub fn init(index: usize) -> Self {
        Self {
            index,
            reverb: ReverbState::init(),
            past: VoiceState::init(),
            current: VoiceState::init(),
            offset_past: VoiceState::init(),
            offset_current: VoiceState::init(),
            phase: 0.0,
            old_phase: 0.0,
            osc_type: OscType::None,
            old_osc_type: None,
            attack: Settings::global().sample_rate as usize,
            decay: Settings::global().sample_rate as usize,
            asr: ASR::Long,
            filters: None,
            old_filters: None,
            filter_crossfade_index: 0,
            osc_crossfade_index: 0,
        }
    }

    /// Renders a single RenderOp given an Offset
    /// This is where all of the rendering logic for a single render_op happens
    pub fn generate_waveform(&mut self, op: &RenderOp, offset: &Offset) -> Vec<f64> {
        let mut buffer: Vec<f64> = vec![0.0; op.samples];

        let p_delta = self.calculate_portamento_delta(
            op.portamento,
            self.offset_past.frequency,
            self.offset_current.frequency,
        );

        let op_gain = self.calculate_op_gain(
            op.next_out,
            self.silence_now(),
            self.silence_next(op),
            op.index + op.samples,
            op.total_samples,
        ) * loudness_normalization(self.offset_current.frequency);

        self.reverb
            .model
            .update(self.current.reverb.unwrap_or(0.0) as f32);

        let gain_factor = op_gain * offset.gain;
        let sample_limit = if op.samples > 250 { op.samples } else { 250 };
        let apply_reverb = self.reverb.state.map_or(false, |s| s > 0.0);

        let sound_to_silence = self.sound_to_silence();

        for (index, sample) in buffer.iter_mut().enumerate() {
            let frequency = self.calculate_frequency(
                index,
                op.portamento,
                p_delta,
                self.offset_past.frequency,
                self.offset_current.frequency,
            );
            let gain = gain_at_index(self.offset_past.gain, gain_factor, index, sample_limit);
            let info = SampleInfo { frequency, gain };

            self.phase = Voice::calculate_current_phase(&info, &self.osc_type, self.phase);

            let mut new_sample = self.osc_type.generate_sample(info, self.phase);

            if let Some(old_osc_type) = &self.old_osc_type {
                self.old_phase = Voice::calculate_current_phase(&info, old_osc_type, self.phase);
                let old_sample = old_osc_type.clone().generate_sample(info, self.old_phase);
                new_sample = if sound_to_silence {
                    old_sample
                } else {
                    Voice::process_crossfade(
                        &mut self.osc_crossfade_index,
                        &mut self.old_osc_type,
                        new_sample,
                        old_sample,
                    )
                };
            }

            if index == op.samples - 1 {
                self.offset_current.frequency = frequency;
                self.offset_current.gain = gain;
            };

            if sound_to_silence && self.old_filters.is_some() {
                new_sample = Voice::process_filter(&mut self.old_filters, new_sample);
            } else if self.filters.is_some() || self.old_filters.is_some() {
                let new_filtered_sample = Voice::process_filter(&mut self.filters, new_sample);

                if self.old_filters.is_some() {
                    let old_filtered_sample =
                        Voice::process_filter(&mut self.old_filters, new_sample);

                    new_sample = Voice::process_crossfade(
                        &mut self.filter_crossfade_index,
                        &mut self.old_filters,
                        new_filtered_sample,
                        old_filtered_sample,
                    );
                } else {
                    new_sample = new_filtered_sample
                }
            }

            if apply_reverb && gain > 0.0 {
                new_sample = self
                    .reverb
                    .model
                    .calc_sample(new_sample as f32, gain as f32)
                    .into();
            }

            *sample += new_sample;
        }

        buffer
    }

    pub fn update(&mut self, op: &RenderOp, offset: &Offset) {
        if op.index == 0 && op.next_out {
            self.reset();
            return;
        }

        if op.index == 0 {
            self.update_current_and_past(op);
            self.update_osc_type(op);
            self.update_reverb(op);
            self.update_attack_decay_asr(op);

            if self.should_update_filters(op) {
                self.update_filters(op);
            }
        };

        self.update_offset_gain_and_frequency(offset);
    }

    fn should_update_filters(&self, op: &RenderOp) -> bool {
        self.filters.as_ref().map_or(true, |self_filters| {
            self_filters.len() != op.filters.len()
                || self_filters
                    .iter()
                    .zip(op.filters.iter())
                    .any(|(self_filter, op_filter)| self_filter.hash != op_filter.hash)
        })
    }

    fn reset(&mut self) {
        self.filters = None;
        self.old_filters = None;
        self.old_osc_type = None;
        self.current.gain = 0.0;
        self.current.frequency = 0.0;
    }

    fn update_current_and_past(&mut self, op: &RenderOp) {
        self.past.frequency = self.current.frequency;
        self.current.frequency = op.f;
        self.past.osc_type = self.current.osc_type.clone();
        self.past.reverb = self.current.reverb;

        self.past.gain = self.past_gain_from_op(op);
        self.current.gain = self.current_gain_from_op(op);
    }

    fn update_osc_type(&mut self, op: &RenderOp) {
        if self.osc_type != op.osc_type && self.osc_type.is_some() {
            self.old_osc_type = Some(self.osc_type.clone());
            self.osc_crossfade_index = 0;
        }

        self.osc_type = if self.past.osc_type.is_some() && op.osc_type.is_none() {
            self.past.osc_type.clone()
        } else {
            op.osc_type.clone()
        };

        self.current.osc_type = op.osc_type.clone();
    }

    fn update_reverb(&mut self, op: &RenderOp) {
        self.reverb.state = if self.past.reverb.is_some() && op.reverb.is_none() {
            self.past.reverb
        } else {
            op.reverb
        };

        self.current.reverb = op.reverb;
    }

    fn update_attack_decay_asr(&mut self, op: &RenderOp) {
        self.attack = op.attack.trunc() as usize;
        self.decay = op.decay.trunc() as usize;
        self.asr = op.asr;
    }

    fn update_filters(&mut self, op: &RenderOp) {
        if self.filters.is_some() {
            std::mem::swap(&mut self.old_filters, &mut self.filters);
            self.filter_crossfade_index = 0;
            self.filters = None;
        }

        self.filters = Some(op.filters.iter().map(|f| f.to_filter()).collect());
    }

    fn update_offset_gain_and_frequency(&mut self, offset: &Offset) {
        self.offset_past.gain = self.offset_current.gain;
        self.offset_past.frequency = self.offset_current.frequency;

        self.offset_current.frequency = if self.sound_to_silence() {
            self.past.frequency * offset.freq
        } else {
            self.current.frequency * offset.freq
        };
    }

    fn process_crossfade(
        crossfade_index: &mut usize,
        old_obj: &mut Option<impl Clone>,
        new_sample: f64,
        old_sample: f64,
    ) -> f64 {
        let crossfade_period = Settings::global().crossfade_period as f64;
        let crossfade_ratio = if *crossfade_index as f64 <= crossfade_period {
            *crossfade_index as f64 / crossfade_period
        } else {
            1.0
        };

        // Fast Equal Power Crossfade
        let fade_out = (1.0 - crossfade_ratio).sqrt();
        let fade_in = crossfade_ratio.sqrt();

        let processed_sample = fade_in * new_sample + fade_out * old_sample;

        if *crossfade_index as f64 >= crossfade_period {
            *old_obj = None;
        }

        *crossfade_index += 1;
        processed_sample
    }

    fn process_filter(filters: &mut Option<Vec<BiquadFilter>>, sample: f64) -> f64 {
        filters
            .as_mut()
            .unwrap()
            .iter_mut()
            .fold(sample, |acc, filter| filter.process(acc))
    }
}
