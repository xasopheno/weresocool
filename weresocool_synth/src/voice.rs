use crate::{
    sample::{DrumState, Waveform},
    loudness::loudness_normalization,
    Offset, SynthOp,
};

use reverb::Reverb;
use weresocool_ast::{OscType, ASR};
use weresocool_filter::*;
use weresocool_shared::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Voice {
    // pub reverb: ReverbState,
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
    pub smoothed_gain: f64,  // Exponentially smoothed gain for click-free transitions
    pub drum_state: DrumState,  // Per-voice biquad filter states for drum synth
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct SampleInfo {
    pub frequency: f64,
    pub gain: f64,
    /// Current sample index within the note (0 to total_samples-1)
    pub sample_index: usize,
    /// Total number of samples in this note
    pub total_samples: usize,
    /// Cached sample rate to avoid per-sample Settings lookup
    pub sample_rate: f64,
}

#[derive(Clone, Debug, PartialEq)]
/// Stores state which allow for a 1-step look back at the previously rendered values.
pub struct VoiceState {
    pub frequency: f64,
    pub gain: f64,
    pub osc_type: OscType,
    // pub reverb: Option<f64>,
}

impl VoiceState {
    pub const fn init() -> Self {
        Self {
            frequency: 0.0,
            gain: 0.0,
            osc_type: OscType::None,
            // reverb: None,
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
            // reverb: ReverbState::init(),
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
            smoothed_gain: 0.0,
            drum_state: DrumState::default(),
        }
    }

    pub fn copy_state_from(&mut self, other: &Voice) {
        *self = other.clone();
    }

    /// Renders a single SynthOp given an Offset
    /// This is where all of the rendering logic for a single synth_op happens
    pub fn generate_waveform<Op: SynthOp>(&mut self, op: &Op, offset: &Offset) -> Vec<f64> {
        let mut buffer: Vec<f64> = vec![0.0; op.duration_samples()];

        let p_delta = self.calculate_portamento_delta(
            op.portamento(),
            self.offset_past.frequency,
            self.offset_current.frequency,
        );

        // Drum oscillators handle their own perceived loudness — the sine
        // equal-loudness curve makes a 4 kHz hihat lose 9 dB before it even
        // hits the synth, which fights against any Fm-based pitch movement.
        let is_drum = matches!(
            op.oscillator_type(),
            OscType::Kick { .. } | OscType::Snare { .. } | OscType::HiHat { .. }
        );
        let loudness = if is_drum {
            1.0
        } else {
            loudness_normalization(self.offset_current.frequency)
        };
        let op_gain = self.calculate_op_gain(
            op.next_out(),
            self.silence_now(),
            self.silence_next(op),
            op.sample_index() + op.duration_samples(),
            op.total_samples(),
        ) * loudness;

        // self.reverb
        // .model
        // .update(self.current.reverb.unwrap_or(0.0) as f32);

        let gain_factor = op_gain * offset.gain;
        // let apply_reverb = self.reverb.state.map_or(false, |s| s > 0.0);

        let sound_to_silence = self.sound_to_silence();
        let silence_to_sound = self.silence_to_sound();

        // Exponential smoothing coefficient for click-free gain transitions
        // ~500 samples (~11ms at 44.1kHz) to reach 63% of target
        const GAIN_SMOOTHING_COEF: f64 = 0.002;

        // Cache sample_rate once per op to avoid per-sample Settings lookup
        let sample_rate = Settings::global().sample_rate;

        // Reset drum filter state at the start of each drum note so the
        // resonant filters don't carry decay/ring from a previous note.
        // The drum sample code re-coefficients the filters on this same
        // first sample, so this just zeros the history.
        if is_drum && op.sample_index() == 0 {
            self.drum_state.reset();
        }

        // Hoist loop-invariant op + voice state out of the per-sample loop.
        // All of these are constant within a single buffer render: op
        // metadata doesn't change, and the Option<...> variants on filter/
        // osc state are mutated in `update*` methods, not inside this loop.
        let portamento_length = op.portamento();
        let duration_samples = op.duration_samples();
        let last_sample_index = duration_samples.saturating_sub(1);
        let total_samples_for_info = op.total_samples();
        let op_sample_index = op.sample_index();
        let f_past = self.offset_past.frequency;
        // `f_target` is only mutated on the loop's final iteration via the
        // index == last_sample_index branch — we replicate that update
        // after the loop so the per-sample read is just a local.
        let f_target = self.offset_current.frequency;
        let distortions_slice: &[crate::DistortionDef] = op.distortions();
        let has_distortions = !distortions_slice.is_empty();
        let has_filters = self.filters.is_some();
        let has_old_filters = self.old_filters.is_some();
        let has_old_osc = self.old_osc_type.is_some();
        let filter_branch_active = has_filters || has_old_filters;
        let mut last_gain_at_end = 0.0_f64;
        let mut last_freq_at_end = f_target;

        for (index, sample) in buffer.iter_mut().enumerate() {
            // Inlined `calculate_frequency` so the per-sample call doesn't
            // re-read `self.sound_to_silence()` / `self.silence_to_sound()`
            // each iteration (those bools are loop-invariant).
            let frequency = if sound_to_silence {
                f_past
            } else if index < portamento_length && !silence_to_sound {
                (index as f64).mul_add(p_delta, f_past)
            } else {
                f_target
            };

            // Exponential smoothing prevents pops on gain changes, but its
            // ~10 ms time constant flattens drum transients — without this
            // branch the first 25 ms of a kick is barely audible. For drums
            // we let *rising* gain pass through instantly (transient survives)
            // but still smooth *falling* gain so the tail doesn't click when
            // a note ends. Sustained tones smooth in both directions.
            let gain = if is_drum {
                if gain_factor >= self.smoothed_gain {
                    self.smoothed_gain = gain_factor;
                } else {
                    self.smoothed_gain += GAIN_SMOOTHING_COEF
                        * (gain_factor - self.smoothed_gain);
                }
                self.smoothed_gain
            } else {
                self.smoothed_gain += GAIN_SMOOTHING_COEF * (gain_factor - self.smoothed_gain);
                self.smoothed_gain
            };

            let info = SampleInfo {
                frequency,
                gain,
                sample_index: op_sample_index + index,
                total_samples: total_samples_for_info,
                sample_rate,
            };

            self.phase = Voice::calculate_current_phase(&info, &self.osc_type, self.phase);

            let mut new_sample = self.osc_type.generate_sample(info, self.phase, &mut self.drum_state);

            if has_old_osc {
                if let Some(old_osc_type) = &self.old_osc_type {
                    self.old_phase = Voice::calculate_current_phase(&info, old_osc_type, self.phase);
                    let old_sample = old_osc_type.generate_sample(info, self.old_phase, &mut self.drum_state);
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
            }

            if index == last_sample_index {
                last_freq_at_end = frequency;
                last_gain_at_end = gain;
            }

            // Apply distortion effects (after oscillator, before filters)
            // Distortion is stateless - no Voice state needed
            if has_distortions {
                new_sample = crate::distortion::process_distortions(new_sample, distortions_slice);
            }

            if sound_to_silence && has_old_filters {
                new_sample = Voice::process_filter(&mut self.old_filters, new_sample);
            } else if filter_branch_active {
                let new_filtered_sample = Voice::process_filter(&mut self.filters, new_sample);

                if has_old_filters {
                    let old_filtered_sample = Voice::process_filter(&mut self.old_filters, new_sample);

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

            // if apply_reverb && gain > 0.0 {
            // new_sample = self
            // .reverb
            // .model
            // .calc_sample(new_sample as f32, gain as f32)
            // .into();
            // }

            *sample += new_sample;
        }

        // Apply the final-iteration update that originally happened inside
        // the per-sample `if index == op.duration_samples() - 1` branch.
        // Only set if the buffer had at least one sample iterated (i.e.
        // `last_sample_index < duration_samples`), matching the original
        // semantics where the branch could only fire when the loop ran.
        if duration_samples > 0 && buffer.len() > last_sample_index {
            self.offset_current.frequency = last_freq_at_end;
            self.offset_current.gain = last_gain_at_end;
        }

        buffer
    }

    pub fn update<Op: SynthOp>(&mut self, op: &Op, offset: &Offset) {
        if op.sample_index() == 0 && op.next_out() {
            self.reset();
            return;
        }

        if op.sample_index() == 0 {
            self.update_current_and_past(op);
            self.update_osc_type(op);
            // self.update_reverb(op);
            self.update_attack_decay_asr(op);

            if self.should_update_filters(op) {
                self.update_filters(op);
            }
        };

        self.update_offset_gain_and_frequency(offset);
    }

    fn should_update_filters<Op: SynthOp>(&self, op: &Op) -> bool {
        self.filters.as_ref().map_or(true, |self_filters| {
            self_filters.len() != op.filters().len()
                || self_filters
                    .iter()
                    .zip(op.filters().iter())
                    .any(|(self_filter, op_filter)| self_filter.hash != op_filter.hash)
        })
    }

    fn reset(&mut self) {
        self.filters = None;
        self.old_filters = None;
        self.old_osc_type = None;
        self.current.gain = 0.0;
        self.current.frequency = 0.0;
        self.smoothed_gain = 0.0;
    }

    fn update_current_and_past<Op: SynthOp>(&mut self, op: &Op) {
        self.past.frequency = self.current.frequency;
        self.current.frequency = op.frequency();
        self.past.osc_type = self.current.osc_type.clone();
        // self.past.reverb = self.current.reverb;

        self.past.gain = self.past_gain_from_op(op);
        self.current.gain = self.current_gain_from_op(op);
    }

    fn update_osc_type<Op: SynthOp>(&mut self, op: &Op) {
        if self.osc_type != *op.oscillator_type() && self.osc_type.is_some() {
            self.old_osc_type = Some(self.osc_type.clone());
            self.osc_crossfade_index = 0;
        }

        self.osc_type = if self.past.osc_type.is_some() && op.oscillator_type().is_none() {
            self.past.osc_type.clone()
        } else {
            op.oscillator_type().clone()
        };

        self.current.osc_type = op.oscillator_type().clone();
    }

    // fn update_reverb<Op: SynthOp>(&mut self, op: &Op) {
    // self.reverb.state = if self.past.reverb.is_some() && op.reverb().is_none() {
    // self.past.reverb
    // } else {
    // op.reverb()
    // };

    // self.current.reverb = op.reverb();
    // }

    fn update_attack_decay_asr<Op: SynthOp>(&mut self, op: &Op) {
        self.attack = op.envelope_attack().trunc() as usize;
        self.decay = op.envelope_decay().trunc() as usize;
        self.asr = op.asr_type();
    }

    fn update_filters<Op: SynthOp>(&mut self, op: &Op) {
        if self.filters.is_some() {
            std::mem::swap(&mut self.old_filters, &mut self.filters);
            self.filter_crossfade_index = 0;
            self.filters = None;
        }

        self.filters = Some(op.filters().iter().map(|f| f.to_filter()).collect());
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
        match filters.as_mut() {
            Some(filters) => filters.iter_mut().fold(sample, |acc, filter| filter.process(acc)),
            None => sample, // Return unfiltered sample if no filters configured
        }
    }
}
