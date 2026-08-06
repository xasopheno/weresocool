use crate::{
    gain::gain_at_index,
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
            drum_state: {
                let mut s = DrumState::default();
                s.voice_index = index as u32;
                // Haas delay: left channel (voice 0) at zero, right (voice 1)
                // lags by 3 samples (~62 µs at 48 kHz) on all noise reads.
                // Brain reads this as "wide source," not as echo.
                s.noise_delay_samples = (index as u32) * 3;
                s
            },
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

        // `op_is_drum`: this op explicitly carries a drum oscillator (a real
        // hit, or at least a drum-typed op). `is_drum`: the oscillator that
        // will actually render — `self.osc_type` after `update()` resolves
        // carry-over, so a kick ringing into a silence op still counts.
        let op_is_drum = op.oscillator_type().is_drum();
        let is_drum = self.osc_type.is_drum();
        let silence_now = self.silence_now();

        // Drum oscillators handle their own perceived loudness — the sine
        // equal-loudness curve makes a 4 kHz hihat lose 9 dB before it even
        // hits the synth, which fights against any Fm-based pitch movement.
        // Keyed on the *rendered* osc so a drum tail carried into a silence
        // op doesn't suddenly pick up the equal-loudness gain tilt.
        let loudness = if is_drum {
            1.0
        } else {
            loudness_normalization(self.offset_current.frequency)
        };
        let op_gain = self.calculate_op_gain(
            op.next_out(),
            silence_now,
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

        // Reset drum state only on a true NOTE-ON: the op explicitly carries
        // a drum oscillator AND has audible gain. Silence ops (Fm 0,
        // Silence) keep a drum osc alive via carry-over, but resetting there
        // would retrigger the attack inside the silence — the tail must keep
        // ringing on the persistent note clock instead. The drum sample code
        // re-coefficients the filters on the note's first sample, so this
        // just zeros the history and advances the note counter.
        if op_is_drum && op.sample_index() == 0 && !silence_now {
            self.drum_state.reset();
        }

        // Hoist loop-invariant op + voice state out of the per-sample loop.
        // All of these are constant within a single buffer render: op
        // metadata doesn't change, and the Option<...> variants on filter/
        // osc state are mutated in `update*` methods, not inside this loop.
        let portamento_length = op.portamento();
        let duration_samples = op.duration_samples();
        // THE ARTICULATION WINDOW — the reason `Gate` exists.
        //
        // `calculate_op_gain` above is evaluated ONCE per buffer, with the
        // index at the buffer's END, so a window test up there silences a
        // whole note rather than the tail of one. The gate has to be a
        // per-sample decision, and it belongs here for a second reason: both
        // envelope branches in `asr.rs` can only end a note when
        // `silence_next` is true, i.e. a note's release is a property of its
        // NEIGHBOUR. That is exactly why composers wrote
        // `Overlay [Seq [Fm 1, Fm 0, Fm 0] | Lm 1/3, Fm 0]` — the `Fm 0`
        // events were there to MANUFACTURE the silence that lets a note
        // stop, at three events per note. A gated note ends on its own edge
        // and asks its neighbour nothing.
        //
        // Measured against the SLOT, never against `duration_samples`: the
        // slot is what the composer wrote, and nothing downstream moves.
        let gate_end = op.gate_end();
        let gated = gate_end < op.total_samples();
        // THE RELEASE IS THE NOTE'S OWN DECAY, measured against the WINDOW.
        //
        // The idiom this replaces —
        // `Overlay [Seq [Fm 1, Fm 0, Fm 0] | Lm 1/3, Fm 0]` — made a short
        // sub-event whose neighbour was silent, so it took the real decay
        // path in `asr.rs`, and `is_short` gave it `total/2` to fade in.
        // That is a release of tens of milliseconds, not a few, and it is
        // why the old spelling sounded clean.
        //
        // A fixed short ramp here (12 ms was the first attempt) cuts a
        // sustained tone an order of magnitude faster than the thing it
        // replaces, and that difference IS the click. So the window is
        // treated as the note's length for envelope purposes, which is the
        // same rule `is_short` applies to a genuinely short note.
        let gate_release = self.decay.min(gate_end / 2).max(1);

        // Fade length for sustained tones: the gain ramps from the previous
        // op's ending gain to this op's target across the WHOLE op duration
        // (min 250 samples to keep very short ops click-free). This is what
        // makes a long fade actually take its full written length.
        let sample_limit = if duration_samples > 250 { duration_samples } else { 250 };
        // A GATED NOTE RISES INSIDE ITS WINDOW, not across its slot.
        //
        // The ramp above spans the whole op, so a note gated to a third of
        // its slot was still climbing when the window closed — it reached a
        // sixth of the amplitude the old spelling gave it AND got cut on the
        // way up, which is what made it click. The idiom this replaces
        // (`Seq [Fm 1, Fm 0, Fm 0] | Lm 1/3`) made a genuinely SHORT note,
        // so its envelope compressed into that length: full rise, full fall.
        // Matching that is the whole job.
        let sample_limit = if gated {
            if gate_end > 250 { gate_end } else { 250 }
        } else {
            sample_limit
        };
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

        // DRUM NOTE CLOCK — drum envelopes are functions of time since
        // note-on, not time since op start. A real drum op renders on the
        // op's own clock (identical to the note clock by construction); a
        // tail carried into a silence op — or an old drum osc crossfading
        // out under a new tone — continues from wherever the note left off
        // instead of restarting at 0 and replaying the attack.
        let old_is_drum = self.old_osc_type.as_ref().is_some_and(|o| o.is_drum());
        let any_drum = is_drum || old_is_drum;
        // Soft-choke decay: ~3 ms to 1/e at any sample rate.
        let choke_coef = (-1.0 / (0.003 * sample_rate)).exp();
        let drum_base = if op_is_drum {
            op_sample_index
        } else {
            self.drum_state.note_sample
        };
        let sample_index_base = if any_drum { drum_base } else { op_sample_index };
        let mut last_gain_at_end = 0.0_f64;
        let mut last_freq_at_end = f_target;

        // Analyzer-measured initial phase to anchor at a voice birth, or None.
        //
        // We only seed at a TRUE birth — when the voice has actually decayed to
        // silence. `silence_to_sound` is derived from the op's gain *parameters*,
        // but the voice's rendered gain (`offset_past.gain`, where the gain ramp
        // starts) can still carry a residual when a preceding gap was too short
        // to ramp to zero. Anchoring the phase while that residual is audible
        // steps the live waveform — a click. Gating on the prior gain being
        // negligible vs this op's target keeps the seed at real births (the
        // phase benefit) and skips voice-reuse boundaries (no click).
        //
        // The seed is applied AT index 0 below, not before the loop: the gain
        // ramps from 0, so `calculate_current_phase` resets phase to 0 on the
        // first (silent) sample and would wipe a pre-loop seed. Setting it as
        // the index-0 phase anchors the trajectory so index 1+ carry the
        // measured phase as the gain ramps up.
        let seed_phase = if silence_to_sound
            && self.offset_past.gain.abs() <= 0.05 * gain_factor.abs()
        {
            op.initial_phase()
        } else {
            None
        };

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
                // Sustained tones: linear ramp from the previous op's ending
                // gain to this op's target across the full op duration. This
                // IS the fade envelope — using the fixed-time-constant
                // exponential smoother here collapsed every fade (however
                // long) into an ~11 ms glide, which broke fade rendering.
                gain_at_index(self.offset_past.gain, gain_factor, index, sample_limit)
            };

            // Close the articulation window. `op_sample_index + index` is
            // the position within the NOTE, so this survives the chunking a
            // held note gets in live playback — every chunk carries the same
            // slot and the same window.
            let gain = if gated {
                let pos = op_sample_index + index;
                if pos >= gate_end {
                    0.0
                } else if pos + gate_release > gate_end {
                    gain * ((gate_end - pos) as f64 / gate_release as f64)
                } else {
                    gain
                }
            } else {
                gain
            };

            let info = SampleInfo {
                frequency,
                gain,
                sample_index: sample_index_base + index,
                total_samples: total_samples_for_info,
                sample_rate,
            };

            // Drum oscillators derive everything from the note clock
            // (`info.sample_index`) and never read `phase` — computing it for
            // them costs a `fmod` and an `OscType` comparison per sample per
            // voice, which is real money once a piece carries a kit per line.
            if !is_drum {
                self.phase = match seed_phase {
                    // Anchor the measured phase at the birth's first sample,
                    // bypassing the gain==0 reset; index 1+ advance normally.
                    Some(phi) if index == 0 => phi,
                    _ => Voice::calculate_current_phase(&info, &self.osc_type, self.phase),
                };
            }

            let mut new_sample = self.osc_type.generate_sample(info, self.phase, &mut self.drum_state);

            if has_old_osc {
                if let Some(old_osc_type) = &self.old_osc_type {
                    if !old_osc_type.is_drum() {
                        // Advance the OUTGOING voice from its OWN phase. This
                        // read `self.phase` — the INCOMING voice's — so the
                        // fading-out tone was re-seeded from the new
                        // oscillator every sample. Harmless while both are
                        // tonal and their phases track each other; ruinous
                        // into a drum, because a drum does not advance
                        // `self.phase` at all. The outgoing tone was then
                        // evaluated at one frozen phase for the whole
                        // crossfade — a constant, i.e. DC, ~0.2 full scale
                        // for ~85 ms at 48 kHz on every tone-to-drum
                        // transition. `pop_check`'s 0.20 threshold sits just
                        // above it, so nothing caught it.
                        self.old_phase =
                            Voice::calculate_current_phase(&info, old_osc_type, self.old_phase);
                    }
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

            // SOFT CHOKE — fold in the ~3 ms decaying tail of whatever this
            // note-on cut (seeded by DrumState::reset()), and track the
            // final drum output so the NEXT note-on can do the same. Turns
            // the one-sample choke step (an audible pop on kick-after-kick
            // and snare rolls) into a fast fade under the new transient.
            if is_drum {
                new_sample += self.drum_state.choke_z;
                self.drum_state.choke_z *= choke_coef;
                self.drum_state.last_out = new_sample;
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

        // Advance the persistent drum note clock past this buffer so a tail
        // carried into the next op (or the next batch of this op) continues
        // from the right point in the note's envelope.
        if any_drum {
            self.drum_state.note_sample = drum_base + duration_samples;
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

    /// Re-latch this voice's per-op state (freq/gain/osc/envelope/filters)
    /// from `op` after a mid-op seek. `update()` only latches at an op's
    /// FIRST sample; a seek that lands inside an op never presents sample 0,
    /// so a freshly-reset voice would render silence until the next op
    /// boundary (inaudible on short notes, fatal on long held ones). The
    /// voice was just re-init'd, so past state is zero and the gain ramps in
    /// from silence — the engine's post-seek master ramp masks the join.
    pub fn latch_for_seek<Op: SynthOp>(&mut self, op: &Op) {
        self.update_current_and_past(op);
        self.update_osc_type(op);
        self.update_attack_decay_asr(op);
        if self.should_update_filters(op) {
            self.update_filters(op);
        }
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
        // Resolve what this op will actually render: the op's own oscillator
        // if it has one, otherwise the previous oscillator carried forward
        // (e.g. a tone or drum tail ringing into a silence op).
        let resolved = if self.past.osc_type.is_some() && op.oscillator_type().is_none() {
            self.past.osc_type.clone()
        } else {
            op.oscillator_type().clone()
        };

        // Crossfade only on a genuine change of the *rendered* oscillator.
        // Comparing against `resolved` (not the op's raw osc) means an osc
        // carried into silence no longer "crossfades" with itself — for
        // drums that self-crossfade ran every stateful filter and the KS
        // delay line twice per sample on shared state. Drum→drum changes
        // also skip the crossfade: both sides would share one DrumState
        // (state corruption), and the new hit's reset + transient masks
        // the swap anyway — real drum machines choke the previous hit.
        let drum_to_drum = self.osc_type.is_drum() && resolved.is_drum();
        if resolved != self.osc_type && self.osc_type.is_some() && !drum_to_drum {
            self.old_osc_type = Some(self.osc_type.clone());
            self.osc_crossfade_index = 0;
        }

        self.osc_type = resolved;
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
