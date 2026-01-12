/// Audio Engine for WereSoCool
///
/// Handles core audio rendering loop, voice management, and render switching

use crate::generation::{sum_all_waveforms, Normalizer};
use crate::manager::resizeable_2d_vec::Resizeable2DVec;
use weresocool_ast::follow::evaluate::EvaluateAction;
use weresocool_instrument::{Offset, RenderOp, StereoWaveform};
use weresocool_instrument::renderable::render_voice::RenderVoice;
use weresocool_instrument::renderable::Renderable;
use weresocool_shared::Settings;

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

                        for (i, voice) in render_voices.iter_mut().enumerate() {
                            match voice.get_batch(
                                remaining_buffer_size,
                                None,
                                loop_play,
                            ) {
                                Some(batch) => {
                                    any_data_rendered = true;
                                    let batch_samples: usize = batch.iter().map(|op| op.samples).sum();
                                    samples_rendered = samples_rendered.max(batch_samples);

                                    // Split MIDI-directed ops from audio-directed
                                    let (midi_batch, mut audio_batch): (Vec<_>, Vec<_>) = batch
                                        .into_iter()
                                        .partition(|op| !op.midi.is_empty());

                                    midi_ops.extend(midi_batch.into_iter());

                                    let voice_rendered =
                                        audio_batch.render(&mut voice.oscillator, Some(&offset));
                                    rendered_per_voice.push(voice_rendered);

                                    if collect_viz_ops {
                                        // Use 1,000,000 scale to support filter rates as low as 0.000001
                                        let vis_threshold = (Settings::global().vis_filter_rate * 1_000_000.0) as usize;
                                        let b: Vec<_> = audio_batch
                                            .iter()
                                            .filter(|op| {
                                                let hash = op.index.wrapping_mul(2654435761) % 1_000_000;
                                                hash < vis_threshold
                                            })
                                            .cloned()
                                            .map(|mut op| {
                                                let follow_offset = op.follows.eval_value(
                                                    offset.freq as f32,
                                                    offset.gain as f32,
                                                );
                                                op.f *= follow_offset.0 as f64;
                                                op.g = (
                                                    op.g.0 * follow_offset.1 as f64,
                                                    op.g.1 * follow_offset.1 as f64,
                                                );
                                                op
                                            })
                                            .collect();

                                        total_ops.extend_at(i, b);
                                    }
                                }
                                None => {
                                    // Voice has finished
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
