/// Audio Engine for WereSoCool
///
/// Handles core audio rendering loop, voice management, and render switching

use crate::generation::{sum_all_waveforms, Normalizer};
use crate::manager::resizeable_2d_vec::Resizeable2DVec;
use weresocool_ast::follow::evaluate::EvaluateAction;
use weresocool_instrument::{Offset, RenderOp, StereoWaveform};
use weresocool_instrument::renderable::render_voice::RenderVoice;
use weresocool_instrument::renderable::Renderable;
use weresocool_shared::{Settings, timing_print};

// NOTE: real-time audio rendering is intentionally **serial**. Earlier
// versions of this file ran the per-voice loop on a dedicated rayon pool
// for a benchmark-confirmed ~5.6× speedup at 100 voices — but that path
// causes audible clicks during playback because rayon's work-stealing
// scheduler competes with the portaudio callback thread for cores.
//
// Even on heavy compositions the serial path renders >100× faster than
// real time, so the buffer queue stays full and there's no upside to
// parallelizing here. Offline rendering (`parsed_to_render::render`) is
// a separate code path that still uses rayon's global pool and is
// unaffected by this comment.

#[derive(Debug)]
pub struct AudioEngine {
    pub renders: [Option<Vec<RenderVoice>>; 2],
    render_idx: usize,
    samples_processed: usize,
    normalizer: Normalizer,
    /// Samples remaining in the post-seek fade-in. While >0 the rendered
    /// waveform is multiplied by `(1 - remaining/ramp_total)` so a hard
    /// seek (which zeroes oscillator state) doesn't produce a click.
    /// 220 samples ≈ 5 ms at 44.1 kHz — long enough to mask the discontinuity,
    /// short enough that the user's "I scrubbed to here" feedback is immediate.
    seek_ramp_remaining: usize,
    seek_ramp_total: usize,
    /// Optional sub-loop `[start, end)` in absolute samples. When set, playback
    /// cycles within this window instead of the whole render — the DAW's
    /// "loop a subset" feature. Repositioning is SILENT (no `Reset` event), so a
    /// subscribed visualizer keeps its accumulated state (the painting doesn't
    /// wipe every cycle). `None` = loop the whole render as usual.
    loop_region: Option<(usize, usize)>,
}

/// Length of the post-seek gain ramp in samples. ~5 ms at 44.1 kHz.
const SEEK_RAMP_SAMPLES: usize = 220;

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
            seek_ramp_remaining: 0,
            seek_ramp_total: 0,
            loop_region: None,
        }
    }

    /// Set (or clear) the sub-loop window `[start, end)` in absolute samples.
    /// `end <= start` clears it.
    pub fn set_loop_region(&mut self, region: Option<(usize, usize)>) {
        self.loop_region = match region {
            Some((s, e)) if e > s => Some((s, e)),
            _ => None,
        };
    }

    /// Seek the audio playhead to `target_sample` (absolute, from the
    /// start of the current render). Voices reset oscillator state and
    /// reposition their per-op cursors; a ~5 ms gain ramp masks the
    /// resulting click. Buffer queue must be drained by the caller
    /// (typically `RenderManager::seek_to_sample`) so stale audio from
    /// the old position doesn't play after the seek.
    pub fn seek_to_sample(&mut self, target_sample: usize) {
        if let Some(voices) = self.current_render().as_mut() {
            for voice in voices.iter_mut() {
                voice.seek(target_sample);
            }
        }
        self.samples_processed = target_sample;
        // Arm the ramp. If a previous ramp was still in flight (seek
        // during seek), we restart it — the latest discontinuity is the
        // one that matters.
        self.seek_ramp_remaining = SEEK_RAMP_SAMPLES;
        self.seek_ramp_total = SEEK_RAMP_SAMPLES;
    }

    /// Apply the post-seek gain ramp (in-place) to a freshly rendered
    /// stereo buffer. Sample count of the buffer = `samples` (the
    /// L and R buffers are equal-length). Called from `render`.
    fn apply_seek_ramp(&mut self, sw: &mut StereoWaveform) {
        if self.seek_ramp_remaining == 0 { return; }
        let total = self.seek_ramp_total.max(1) as f32;
        let n = sw.l_buffer.len().min(sw.r_buffer.len());
        for i in 0..n {
            if self.seek_ramp_remaining == 0 { break; }
            // Linear ramp from 0 (just after seek) up to 1 (ramp done).
            let consumed = self.seek_ramp_total - self.seek_ramp_remaining;
            let g = (consumed as f32 / total).min(1.0);
            sw.l_buffer[i] *= g as f64;
            sw.r_buffer[i] *= g as f64;
            self.seek_ramp_remaining -= 1;
        }
    }

    pub fn normalizer(&self) -> &Normalizer {
        &self.normalizer
    }

    pub fn samples_processed(&self) -> usize {
        self.samples_processed
    }

    /// Total samples in the currently-loaded render — i.e. the timeline
    /// length the slider should map its range to. Computed by summing
    /// op `samples` for voice 0; the NF guarantees all voices in a
    /// render share the same total length, so picking one is fine.
    /// Returns `None` if no render is loaded yet.
    pub fn total_samples(&self) -> Option<usize> {
        self.current_render_ref().as_ref().and_then(|voices| {
            voices.first().map(|v| v.ops.iter().map(|op| op.samples).sum())
        })
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

                        // Render each voice serially. See the file-level note —
                        // rayon is intentionally absent from the real-time path.
                        let per_voice: Vec<PerVoiceOutput> = render_voices
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
                            .collect();

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

            // Sub-loop wrap: if we've reached the region end, reposition the
            // voices back to the region start (silently — no Reset, so the
            // painting keeps its accumulated state). Batch-granular; the ~5ms
            // seek ramp masks the oscillator phase discontinuity at the wrap.
            if let Some((start, end)) = self.loop_region {
                if self.samples_processed >= end {
                    let overflow = self.samples_processed - end;
                    let span = end - start;
                    let target = start + overflow % span;
                    if let Some(voices) = self.current_render().as_mut() {
                        for v in voices.iter_mut() {
                            v.seek(target);
                        }
                    }
                    self.samples_processed = target;
                    self.seek_ramp_remaining = SEEK_RAMP_SAMPLES;
                    self.seek_ramp_total = SEEK_RAMP_SAMPLES;
                    continue;
                }
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

            // Post-seek fade-in: masks the click from `seek_to_sample`
            // having reset voice oscillator state. Applied after pad so
            // the silence we just padded with also gets gated to zero
            // during the ramp (matters only if the seek landed near the
            // very end of the timeline, but cheap to do regardless).
            self.apply_seek_ramp(&mut combined_sw);

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

    /// Set the runtime `gain_mul` on every voice in the *current* render whose
    /// ops carry `tag` in their `names` (a `#tag` marker in the source). Used
    /// for live per-voice faders (DAW mixer volume) — no re-render. Returns the
    /// number of voices matched (0 if the tag isn't present / no render loaded).
    pub fn set_gain_for_tagged_voices(&mut self, tag: &str, gain: f64) -> usize {
        let mut matched = 0;
        if let Some(voices) = self.current_render().as_mut() {
            for voice in voices.iter_mut() {
                if voice.ops.iter().any(|op| op.names.iter().any(|n| n == tag)) {
                    voice.gain_mul = gain;
                    matched += 1;
                }
            }
        }
        matched
    }

    /// Replace the *current* render's voices in place, preserving the playhead
    /// (`samples_processed`). Each new voice is seeked to the current render
    /// position so the timeline continues uninterrupted — used for live mix
    /// changes (mute/solo/volume/arm in the DAW) where the composition length
    /// is unchanged and playback must NOT jump back to the start.
    ///
    /// Unlike `inc_render`, this switches nothing and emits nothing: the render
    /// index is untouched and no `Reset`/`AudioReady` events fire, so a
    /// subscribed visualizer keeps all of its accumulated state (the painting
    /// keeps building). The already-buffered old-mix audio plays out and the
    /// new voices pick up exactly where it left off — so the caller must NOT
    /// drain the buffer queue. The seek fade-in ramp masks the oscillator
    /// phase discontinuity at the swap point.
    pub fn swap_current_render(&mut self, mut render: Vec<RenderVoice>) {
        let pos = self.samples_processed;
        // In loop playback `samples_processed` grows unbounded across loops,
        // but a voice's timeline is only `0..total`. Fold the playhead back
        // into the loop with a modulo before seeking — otherwise the swapped
        // voices would park past the end and restart from sample 0. All voices
        // in a render share the same total length (NF guarantee), so voice 0's
        // op-sample sum is the loop length.
        let total: usize = render
            .first()
            .map(|v| v.ops.iter().map(|op| op.samples).sum())
            .unwrap_or(0);
        let seek_pos = if total > 0 { pos % total } else { pos };
        for voice in render.iter_mut() {
            voice.seek(seek_pos);
        }
        *self.current_render() = Some(render);
        // Preserve the absolute playhead counter (it stays in phase with the
        // looped voice position via the same modulo the render loop uses).
        self.samples_processed = pos;
        // Arm the ~5 ms ramp so the fresh oscillator state doesn't click.
        self.seek_ramp_remaining = SEEK_RAMP_SAMPLES;
        self.seek_ramp_total = SEEK_RAMP_SAMPLES;
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

    // Split MIDI-directed ops from audio-directed. The MIDI case is rare
    // in most pieces, so a quick scan first lets us skip the partition
    // (and its two Vec allocations) for the common all-audio path.
    let has_midi = batch.iter().any(|op| !op.midi.is_empty());
    let (midi_batch, mut audio_batch): (Vec<RenderOp>, Vec<RenderOp>) = if has_midi {
        batch.into_iter().partition(|op| !op.midi.is_empty())
    } else {
        (Vec::new(), batch)
    };

    let mut voice_rendered = audio_batch.render(&mut voice.oscillator, Some(offset));

    // Runtime per-voice fader (DAW mixer volume): scale the AUDIO only, leaving
    // the viz ops below at their rendered gain so the fader doesn't resize the
    // painting. Skip the multiply at unity — the overwhelmingly common case.
    if voice.gain_mul != 1.0 {
        let g = voice.gain_mul;
        for s in voice_rendered.l_buffer.iter_mut() { *s *= g; }
        for s in voice_rendered.r_buffer.iter_mut() { *s *= g; }
    }

    let viz_ops = if collect_viz_ops {
        audio_batch
            .iter()
            .filter(|op| {
                // A note's FIRST batch (index 0) always passes — every note
                // announces its onset to the vis/envelope layer exactly once.
                // Without this, short notes (drum hits) randomly vanish from
                // the visuals entirely when the hash sampling skips them.
                if op.index == 0 {
                    return true;
                }
                let hash = op.index.wrapping_mul(2654435761) % 1_000_000;
                hash < vis_threshold
            })
            .cloned()
            .map(|mut op| {
                // Match the audio path: only Follow voices track the mic offset
                // (empty follows would otherwise pass it through, shifting the
                // whole piece's visuals).
                if !op.follows.is_empty() {
                    let follow_offset =
                        op.follows.eval_value(offset.freq as f32, offset.gain as f32);
                    op.f *= follow_offset.0 as f64;
                    op.g = (
                        op.g.0 * follow_offset.1 as f64,
                        op.g.1 * follow_offset.1 as f64,
                    );
                }
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
