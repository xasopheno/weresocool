/// Audio Engine for WereSoCool
///
/// Handles core audio rendering loop, voice management, and render switching

use crate::generation::{sum_all_waveforms, Normalizer};
use crate::manager::resizeable_2d_vec::Resizeable2DVec;
use rayon::prelude::*;
use weresocool_ast::follow::evaluate::EvaluateAction;
use weresocool_instrument::{Offset, RenderOp, StereoWaveform};
use weresocool_instrument::renderable::render_voice::RenderVoice;
use weresocool_instrument::renderable::Renderable;
use weresocool_shared::{Settings, timing_print};

/// Voice counts at or above this threshold use the rayon parallel
/// voice-render path. Below it, the per-iteration overhead of work-
/// stealing exceeds the gain (heuristic confirmed by bench at 8 vs 100
/// voices). Tunable; the only correctness constraint is `>= 1`.
const PARALLEL_VOICE_THRESHOLD: usize = 16;

#[derive(Debug)]
pub struct AudioEngine {
    pub renders: [Option<Vec<RenderVoice>>; 2],
    render_idx: usize,
    samples_processed: usize,
    normalizer: Normalizer,
}

/// Result of a rendering operation
pub struct RenderResult {
    pub waveform: StereoWaveform,
    pub ops_per_voice: Vec<Vec<RenderOp>>,
    pub midi_ops: Vec<RenderOp>,
    pub read_start_samples: usize,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            renders: [None, None],
            render_idx: 0,
            samples_processed: 0,
            normalizer: Normalizer::default(),
        }
    }

    pub fn normalizer(&self) -> &Normalizer {
        &self.normalizer
    }

    pub fn samples_processed(&self) -> usize {
        self.samples_processed
    }

    /// Core audio rendering method
    /// Returns audio waveform, operations for visualization, and MIDI operations
    pub fn render(
        &mut self,
        buffer_size: usize,
        offset: Offset,
        collect_viz_ops: bool,
    ) -> Option<RenderResult> {
        let mut remaining_buffer_size = buffer_size;
        // Get voice count for pre-allocation
        let num_voices = self.current_render_ref().as_ref().map(|v| v.len()).unwrap_or(0);
        // Collect ops for visualization only - pre-allocate with voice count if needed
        let mut total_ops: Resizeable2DVec<RenderOp> = if collect_viz_ops {
            Resizeable2DVec::new(num_voices)
        } else {
            Resizeable2DVec::new(0)
        };
        // Final combined waveform we build progressively
        let mut combined_sw = StereoWaveform::new_empty();
        // MIDI ops accumulated for this read window
        let mut midi_ops: Vec<RenderOp> = Vec::new();

        // Track absolute sample position at the start of this read
        let read_start_samples = self.samples_processed;

        while remaining_buffer_size > 0 {
            // Compute next_exists before mutable borrow
            let next_exists = self.exists_next_render();

            // Start mutable borrow scope
            let (samples_processed, render_finished) = {
                let current_render_option = self.current_render();

                match current_render_option {
                    Some(render_voices) => {
                        let mut any_data_rendered = false;
                        let mut rendered_per_voice: Vec<StereoWaveform> = Vec::with_capacity(render_voices.len());
                        let loop_play = !next_exists && Settings::global().loop_play;
                        let mut samples_rendered = 0usize;
                        // Read once outside the (possibly parallel) inner closure.
                        let vis_threshold = if collect_viz_ops {
                            (Settings::global().vis_filter_rate * 1_000_000.0) as usize
                        } else {
                            0
                        };

                        // Render each voice. Voices own independent state
                        // (oscillator, sample_index, op_index) and the
                        // per-voice work — get_batch + audio_batch.render —
                        // doesn't touch any shared state, so we can run it
                        // across the rayon pool when the voice count is
                        // large enough to amortize work-stealing overhead.
                        //
                        // All collected outputs are merged serially after
                        // the parallel section. Merge operations
                        // (midi_ops.extend, rendered_per_voice.push,
                        // total_ops.extend_at, samples_rendered.max) are
                        // commutative, so iteration order doesn't matter
                        // for correctness.
                        let per_voice: Vec<PerVoiceOutput> = if render_voices.len() >= PARALLEL_VOICE_THRESHOLD {
                            render_voices
                                .par_iter_mut()
                                .enumerate()
                                .map(|(i, voice)| {
                                    render_one_voice(
                                        i,
                                        voice,
                                        remaining_buffer_size,
                                        loop_play,
                                        &offset,
                                        collect_viz_ops,
                                        vis_threshold,
                                    )
                                })
                                .collect()
                        } else {
                            render_voices
                                .iter_mut()
                                .enumerate()
                                .map(|(i, voice)| {
                                    render_one_voice(
                                        i,
                                        voice,
                                        remaining_buffer_size,
                                        loop_play,
                                        &offset,
                                        collect_viz_ops,
                                        vis_threshold,
                                    )
                                })
                                .collect()
                        };

                        for out in per_voice {
                            let PerVoiceOutput {
                                voice_index,
                                rendered,
                            } = out;
                            if let Some(VoiceRenderData {
                                batch_samples,
                                midi_batch,
                                voice_rendered,
                                viz_ops,
                            }) = rendered
                            {
                                any_data_rendered = true;
                                samples_rendered = samples_rendered.max(batch_samples);
                                midi_ops.extend(midi_batch);
                                rendered_per_voice.push(voice_rendered);
                                if collect_viz_ops {
                                    total_ops.extend_at(voice_index, viz_ops);
                                }
                            }
                        }

                        if any_data_rendered && samples_rendered > 0 {
                            // Mix all voices (NormalForm ensures they have the same length)
                            let batch_sw = sum_all_waveforms(rendered_per_voice);
                            combined_sw.append(batch_sw);
                            // Advance absolute playhead samples
                            self.samples_processed = self.samples_processed.saturating_add(samples_rendered);
                            (samples_rendered, false)
                        } else if any_data_rendered {
                            (0, false)
                        } else {
                            // All voices have finished
                            (0, true)
                        }
                    }
                    None => {
                        // No current render
                        (0, true)
                    }
                }
            }; // End of mutable borrow

            if samples_processed > 0 {
                remaining_buffer_size = remaining_buffer_size.saturating_sub(samples_processed);
            }

            if render_finished || (next_exists && !Settings::global().loop_play) {
                if self.exists_next_render() {
                    weresocool_shared::timing_print!("[audio_engine] switching to next render (render_finished={}, next_exists={}, loop_play={})",
                        render_finished, next_exists, Settings::global().loop_play);
                    self.inc_render(true); // Copy oscillators for seamless transitions
                    continue; // Continue processing with next render
                } else {
                    break; // No more renders, exit loop
                }
            }

            if samples_processed == 0 {
                // No samples processed, break to avoid infinite loop
                break;
            }
        }

        // If we rendered anything, pad to the buffer size and return
        if combined_sw.l_buffer.len() > 0 {
            combined_sw.pad(buffer_size);

            Some(RenderResult {
                waveform: combined_sw,
                // Use into_vec() to avoid cloning - we no longer need total_ops after this
                ops_per_voice: total_ops.into_vec(),
                midi_ops,
                read_start_samples,
            })
        } else {
            None
        }
    }

    /// Increment to the next render, optionally copying oscillator state for smooth transitions
    pub fn inc_render(&mut self, copy_oscillators: bool) {
        // Since self.renders has length 2, we can split it at index 1
        let (first, second) = self.renders.split_at_mut(1);

        let (current_render_option, next_render_option) = if self.render_idx == 0 {
            (&first[0], &mut second[0])
        } else {
            (&second[0], &mut first[0])
        };

        // Only copy oscillators if requested (for seamless transitions)
        if copy_oscillators {
            if let (Some(current_voices), Some(next_voices)) =
                (current_render_option.as_ref(), next_render_option.as_mut())
            {
                // Ensure that both renders have the same number of voices
                let min_length = std::cmp::min(current_voices.len(), next_voices.len());
                for i in 0..min_length {
                    let current_oscillator = &current_voices[i].oscillator;
                    let next_oscillator = &mut next_voices[i].oscillator;

                    next_oscillator.copy_state_from(current_oscillator);
                }
            }
        }

        // Reset samples processed for the new render
        self.samples_processed = 0;

        *self.current_render() = None;
        self.render_idx = (self.render_idx + 1) % 2;
    }

    pub fn current_render(&mut self) -> &mut Option<Vec<RenderVoice>> {
        &mut self.renders[self.render_idx]
    }

    pub fn next_render(&mut self) -> &mut Option<Vec<RenderVoice>> {
        &mut self.renders[(self.render_idx + 1) % 2]
    }

    pub fn current_render_ref(&self) -> &Option<Vec<RenderVoice>> {
        &self.renders[self.render_idx]
    }

    pub fn exists_current_render(&self) -> bool {
        self.renders[(self.render_idx) % 2].is_some()
    }

    pub fn exists_next_render(&self) -> bool {
        self.renders[(self.render_idx + 1) % 2].is_some()
    }

    pub fn push_render(&mut self, render: Vec<RenderVoice>) {
        *self.next_render() = Some(render);
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Output of a single voice's per-batch work. Returned from the
/// (possibly parallel) per-voice section so the orchestrator can merge
/// results serially.
struct PerVoiceOutput {
    voice_index: usize,
    /// `None` if the voice ran out of ops this batch.
    rendered: Option<VoiceRenderData>,
}

struct VoiceRenderData {
    batch_samples: usize,
    midi_batch: Vec<RenderOp>,
    voice_rendered: StereoWaveform,
    viz_ops: Vec<RenderOp>,
}

/// Pure (modulo each voice's own oscillator + indices) per-voice work.
/// Safe to call in parallel because it only mutates the voice handed in.
fn render_one_voice(
    voice_index: usize,
    voice: &mut RenderVoice,
    remaining_buffer_size: usize,
    loop_play: bool,
    offset: &Offset,
    collect_viz_ops: bool,
    vis_threshold: usize,
) -> PerVoiceOutput {
    let Some(batch) = voice.get_batch(remaining_buffer_size, None, loop_play) else {
        return PerVoiceOutput { voice_index, rendered: None };
    };

    let batch_samples: usize = batch.iter().map(|op| op.samples).sum();

    // Split MIDI-directed ops from audio-directed.
    let (midi_batch, mut audio_batch): (Vec<_>, Vec<_>) = batch
        .into_iter()
        .partition(|op| !op.midi.is_empty());

    let voice_rendered = audio_batch.render(&mut voice.oscillator, Some(offset));

    let viz_ops = if collect_viz_ops {
        audio_batch
            .iter()
            .filter(|op| {
                let hash = op.index.wrapping_mul(2654435761) % 1_000_000;
                hash < vis_threshold
            })
            .cloned()
            .map(|mut op| {
                let follow_offset = op.follows.eval_value(offset.freq as f32, offset.gain as f32);
                op.f *= follow_offset.0 as f64;
                op.g = (
                    op.g.0 * follow_offset.1 as f64,
                    op.g.1 * follow_offset.1 as f64,
                );
                op
            })
            .collect()
    } else {
        Vec::new()
    };

    PerVoiceOutput {
        voice_index,
        rendered: Some(VoiceRenderData {
            batch_samples,
            midi_batch,
            voice_rendered,
            viz_ops,
        }),
    }
}
