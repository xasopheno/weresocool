use crate::{
    renderable::{Offset, RenderOp},
    sample::Waveform,
    {gain::gain_at_index, loudness::loudness_normalization},
};

use reverb::Reverb;
use weresocool_ast::{OscType, ASR};
use weresocool_filter::*;
use weresocool_shared::*;

use rand::{thread_rng, Rng};
pub fn random_offset() -> f64 {
    thread_rng().gen_range(-0.5, 0.5)
}

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
    pub filters: Vec<BiquadFilter>,
    pub old_filters: Option<Vec<BiquadFilter>>,
    pub filter_crossfade_index: usize,
    pub filter_crossfade_period: usize,
    pub osc_crossfade_index: usize,
    pub osc_crossfade_period: usize,
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
            osc_type: OscType::Sine { pow: None },
            old_osc_type: None,
            attack: Settings::global().sample_rate as usize,
            decay: Settings::global().sample_rate as usize,
            asr: ASR::Long,
            filters: vec![],
            old_filters: None,
            filter_crossfade_index: 0,
            filter_crossfade_period: 0usize,
            osc_crossfade_index: 0,
            osc_crossfade_period: 0usize,
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

            self.phase = match self.osc_type {
                OscType::Noise => self.calculate_current_phase(&info, random_offset()),
                _ => self.calculate_current_phase(&info, 0.0),
            };

            let mut new_sample = self.osc_type.generate_sample(info, self.phase);

            // If we're in the middle of a crossfade
            if self.old_osc_type.is_some() && self.osc_crossfade_index < self.osc_crossfade_period {
                self.old_phase = match self.osc_type {
                    OscType::Noise => self.calculate_current_phase(&info, random_offset()),
                    _ => self.calculate_current_phase(&info, 0.0),
                };
                // Generate the old oscillator sample
                let old_sample = self
                    .old_osc_type
                    .unwrap()
                    .generate_sample(info, self.old_phase);

                // Calculate the crossfade gain
                let crossfade_gain =
                    self.osc_crossfade_index as f64 / self.osc_crossfade_period as f64;

                // Crossfade the old and new samples
                new_sample = crossfade_gain * new_sample + (1.0 - crossfade_gain) * old_sample;

                // Increment the crossfade index
                self.osc_crossfade_index += 1;
            }

            if apply_reverb && gain > 0.0 {
                new_sample = self
                    .reverb
                    .model
                    .calc_sample(new_sample as f32, gain as f32)
                    .into();
            }

            if index == op.samples - 1 {
                self.offset_current.frequency = frequency;
                self.offset_current.gain = gain;
            };

            let new_filtered_sample = self
                .filters
                .iter_mut()
                .fold(new_sample, |acc, filter| filter.process(acc));

            if self.old_filters.is_some() {
                let old_filtered_sample = self
                    .old_filters
                    .as_mut()
                    .unwrap()
                    .iter_mut()
                    .fold(new_sample, |acc, filter| filter.process(acc));

                let crossfade_gain = if self.filter_crossfade_index < self.filter_crossfade_period {
                    self.filter_crossfade_index as f64 / self.filter_crossfade_period as f64
                } else {
                    1.0
                };

                new_sample = crossfade_gain * new_filtered_sample
                    + (1.0 - crossfade_gain) * old_filtered_sample;

                self.filter_crossfade_index += 1;
            }

            *sample += new_sample;
        }

        buffer
    }

    pub fn update(&mut self, op: &RenderOp, offset: &Offset) {
        let will_update_filters = self
            .filters
            .iter()
            .map(|f| f.hash.clone())
            .collect::<Vec<String>>()
            == op
                .filters
                .iter()
                .map(|f| f.hash.clone())
                .collect::<Vec<String>>();

        if op.index == 0 {
            self.past.frequency = self.current.frequency;
            self.current.frequency = op.f;
            self.past.osc_type = self.current.osc_type;
            self.past.reverb = self.current.reverb;

            self.past.gain = self.past_gain_from_op(op);
            self.current.gain = self.current_gain_from_op(op);

            if self.osc_type != op.osc_type {
                // Save the old oscillator type
                self.old_osc_type = Some(self.osc_type);

                // Initialize the crossfade
                self.osc_crossfade_index = 0;
                self.osc_crossfade_period = 2048 * 2; // This value could be adjusted as needed
            }

            self.osc_type = if self.past.osc_type.is_some() && op.osc_type.is_none() {
                self.past.osc_type
            } else {
                op.osc_type
            };

            self.reverb.state = if self.past.reverb.is_some() && op.reverb.is_none() {
                self.past.reverb
            } else {
                op.reverb
            };

            self.attack = op.attack.trunc() as usize;
            self.decay = op.decay.trunc() as usize;
            self.asr = op.asr;
            self.current.osc_type = op.osc_type;
            self.current.reverb = op.reverb;
            if !(will_update_filters) {
                self.old_filters = Some(self.filters.clone()); // Save the old filters
                self.filter_crossfade_index = 0; // Reset the crossfade index
                self.filter_crossfade_period = 2048 * 2; // Set the length of the crossfade
                self.filters = op
                    .filters
                    .iter()
                    .map(|f| {
                        lowpass_filter(
                            f.hash.clone(),
                            r_to_f64(f.cutoff_frequency),
                            r_to_f64(f.q_factor),
                        )
                    })
                    .collect();
            }
        };
        self.offset_past.gain = self.offset_current.gain;
        self.offset_past.frequency = self.offset_current.frequency;

        self.offset_current.frequency = if self.sound_to_silence() {
            self.past.frequency * offset.freq
        } else {
            self.current.frequency * offset.freq
        }
    }
}
