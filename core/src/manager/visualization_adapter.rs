/// Visualization Adapter for WereSoCool
///
/// Handles conversion from RenderOp to Op4D for visualization systems

use crate::generation::{Normalizer, Op4D};
use opmap::OpMap;
use weresocool_instrument::renderable::RenderOp;

pub struct VisualizationAdapter;

impl VisualizationAdapter {
    /// Convert a batch of RenderOps to an OpMap for visualization
    ///
    /// Filters out silent operations and groups by color name
    pub fn render_ops_to_opmap(
        ops: &[RenderOp],
        normalizer: &Normalizer,
    ) -> OpMap<Op4D> {
        let mut opmap: OpMap<Op4D> = OpMap::with_capacity(ops.len());

        ops.iter().for_each(|v| {
            let name = v.colors.last().map_or("nameless", |n| n);
            let op = Self::render_op_to_normalized_op4d(v, normalizer);
            if let Some(o) = op {
                opmap.insert(name, o);
            }
        });

        opmap
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

        let mut op4d = Op4D {
            y: render_op.f.log10(),  // Convert to log scale before normalization
            z: (render_op.g.0 + render_op.g.1) / 2.0,
            x: render_op.p,
            l: render_op.l,
            t: render_op.t,
            voice: render_op.voice,
            event: render_op.event,
            names: render_op.names.to_vec(),
            colors: render_op.colors.to_vec(),
            wgsl: render_op.wgsl.clone(),
        };

        op4d.normalize(normalizer);

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

        // We'll subdivide render_op.l in multiples of frame_length
        let total_length = render_op.l;
        if total_length <= 0.0 {
            return vec![];
        }

        let mut out_ops = Vec::new();

        let mut current_time = render_op.t;
        let mut remaining = total_length;

        while remaining > 0.0 {
            // Take either a full frame_length or whatever leftover remains
            let slice_len = if remaining >= frame_length {
                frame_length
            } else {
                remaining
            };

            // Build a brand-new Op4D for just this slice
            let mut op4d = Op4D {
                y: render_op.f.log10(),  // Convert to log scale before normalization
                z: (render_op.g.0 + render_op.g.1) / 2.0,
                x: render_op.p,
                l: slice_len,
                t: current_time,
                voice: render_op.voice,
                event: render_op.event,
                names: render_op.names.clone(),
                colors: render_op.colors.clone(),
                wgsl: render_op.wgsl.clone(),
            };

            // Apply normalization
            op4d.normalize(normalizer);

            out_ops.push(op4d);

            // Advance current_time for the next slice
            current_time += slice_len;
            // Subtract this slice from the remaining length
            remaining -= slice_len;
        }

        out_ops
    }
}
