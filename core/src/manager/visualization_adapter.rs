/// Visualization Adapter for WereSoCool
///
/// Handles conversion from RenderOp to Op4D for visualization systems

use crate::generation::{Normalizer, Op4D};
use opmap::OpMap;
use std::collections::HashMap;
use weresocool_ast::OscType;
use weresocool_instrument::renderable::RenderOp;

pub struct VisualizationAdapter;

/// Memoize `u64` color-hash IDs → their decimal string form. The set of distinct color
/// IDs in a composition is tiny (~tens at most) while the number of `RenderOp`s that
/// reference them is in the millions, so a small `HashMap` here turns 4M+ fresh
/// `String` allocations into <100.
#[derive(Default)]
struct ColorStringCache(HashMap<u64, String>);

impl ColorStringCache {
    fn get(&mut self, id: u64) -> &str {
        self.0.entry(id).or_insert_with(|| id.to_string())
    }
    fn convert(&mut self, ids: &[u64]) -> Vec<String> {
        ids.iter().map(|id| self.get(*id).to_string()).collect()
    }
}

impl VisualizationAdapter {
    /// Convert a batch of RenderOps to an OpMap for visualization
    ///
    /// Filters out silent operations and groups by color name
    pub fn render_ops_to_opmap(
        ops: &[RenderOp],
        normalizer: &Normalizer,
    ) -> OpMap<Op4D> {
        let mut opmap: OpMap<Op4D> = OpMap::with_capacity(ops.len());
        let mut color_cache = ColorStringCache::default();

        ops.iter().for_each(|v| {
            let op = Self::render_op_to_normalized_op4d_with_cache(v, normalizer, &mut color_cache);
            if let Some(o) = op {
                // Borrow the cached name *after* the conversion so the two `&mut` borrows
                // of `color_cache` don't overlap.
                let name: &str = match v.colors.last() {
                    Some(id) => color_cache.get(*id),
                    None => "nameless",
                };
                opmap.insert(name, o);
            }
        });

        opmap
    }

    fn render_op_to_normalized_op4d_with_cache(
        render_op: &RenderOp,
        normalizer: &Normalizer,
        color_cache: &mut ColorStringCache,
    ) -> Option<Op4D> {
        let mut op4d = Self::render_op_to_normalized_op4d(render_op, normalizer)?;
        op4d.colors = color_cache.convert(&render_op.colors);
        Some(op4d)
    }

    /// Convert a single RenderOp to a normalized Op4D for visualization
    ///
    /// Returns None if the operation is silent (zero frequency or gain)
    pub fn render_op_to_normalized_op4d(
        render_op: &RenderOp,
        normalizer: &Normalizer,
    ) -> Option<Op4D> {
        if render_op.f == 0.0 || render_op.g == (0.0, 0.0) {
            return None;
        }

        // Override frequency and length for drums to match their actual synthesis
        // Frequency scaling: kick (f/8) -> snare (f/2.5) -> hihat (f*1.28) gives equal log spacing
        let (visual_freq, visual_length) = match &render_op.osc_type {
            OscType::Kick { .. } => (render_op.f / 8.0, 0.01),
            OscType::Snare { .. } => (render_op.f / 2.5, 0.012),
            OscType::HiHat { open, .. } => {
                let decay = if *open { 0.02 } else { 0.008 };
                (render_op.f * 1.28, decay)
            }
            _ => (render_op.f, render_op.l),
        };

        let mut op4d = Op4D {
            y: visual_freq.log10(),  // Convert to log scale before normalization
            z: (render_op.g.0 + render_op.g.1) / 2.0,
            x: render_op.p,
            l: visual_length,
            t: render_op.t,
            voice: render_op.voice,
            event: render_op.event,
            names: render_op.names.to_vec(),
            // Colors filled in by the caller (typically via the cached path); empty here
            // is fine because the single-op variant is rarely the visualization driver.
            colors: render_op.colors.iter().map(|c| c.to_string()).collect(),
            wgsl: render_op.wgsl.clone(),
            color_gradient: render_op.color_gradient,
            color_mix: render_op.color_mix,
        };

        op4d.normalize(normalizer);

        // Note: Color selection based on gradient happens in kintaro,
        // which has access to the full ColorMap with actual RGB values.
        // We just pass the gradient direction via op4d.color_gradient.

        Some(op4d)
    }

    /// Convert a RenderOp to a list of normalized Op4D slices for frame-based visualization
    ///
    /// Splits a long RenderOp into multiple shorter Op4D operations, each with duration
    /// `frame_length`. This is useful for smooth frame-by-frame visualization at a target FPS.
    ///
    /// # Arguments
    /// * `render_op` - The source render operation
    /// * `normalizer` - Used to normalize frequency/gain values
    /// * `frame_length` - Duration of each slice in seconds (e.g., 1/30 = 0.0333 for 30 FPS)
    ///
    /// # Returns
    /// A vector of Op4D operations, each representing one frame's worth of audio.
    /// Returns empty vec if the operation is silent or has zero/negative length.
    pub fn render_op_to_normalized_op4d_list(
        render_op: &RenderOp,
        normalizer: &Normalizer,
        frame_length: f64,
    ) -> Vec<Op4D> {
        // If these conditions fail, just return an empty Vec
        if render_op.f == 0.0 || render_op.g == (0.0, 0.0) {
            return vec![];
        }

        // Override frequency and length for drums to match their actual synthesis
        // Frequency scaling: kick (f/8) -> snare (f/2.5) -> hihat (f*1.28) gives equal log spacing
        let (visual_freq, visual_length) = match &render_op.osc_type {
            OscType::Kick { .. } => (render_op.f / 8.0, 0.01),
            OscType::Snare { .. } => (render_op.f / 2.5, 0.012),
            OscType::HiHat { open, .. } => {
                let decay = if *open { 0.02 } else { 0.008 };
                (render_op.f * 1.28, decay)
            }
            _ => (render_op.f, render_op.l),
        };

        // We'll subdivide visual_length in multiples of frame_length
        let total_length = visual_length;
        if total_length <= 0.0 {
            return vec![];
        }

        let mut out_ops = Vec::new();

        let mut current_time = render_op.t;
        let mut remaining = total_length;

        // Convert color hash IDs to their decimal-string form once per RenderOp
        // rather than once per output slice (a single op can produce dozens of slices).
        let colors_strings: Vec<String> = render_op.colors.iter().map(|c| c.to_string()).collect();

        while remaining > 0.0 {
            // Take either a full frame_length or whatever leftover remains
            let slice_len = if remaining >= frame_length {
                frame_length
            } else {
                remaining
            };

            // Build a brand-new Op4D for just this slice
            let mut op4d = Op4D {
                y: visual_freq.log10(),  // Convert to log scale before normalization
                z: (render_op.g.0 + render_op.g.1) / 2.0,
                x: render_op.p,
                l: slice_len,
                t: current_time,
                voice: render_op.voice,
                event: render_op.event,
                names: render_op.names.clone(),
                colors: colors_strings.clone(),
                wgsl: render_op.wgsl.clone(),
                color_gradient: render_op.color_gradient,
                color_mix: render_op.color_mix,
            };

            // Apply normalization
            op4d.normalize(normalizer);

            // Note: Color selection based on gradient happens in kintaro
            out_ops.push(op4d);

            // Advance current_time for the next slice
            current_time += slice_len;
            // Subtract this slice from the remaining length
            remaining -= slice_len;
        }

        out_ops
    }
}
