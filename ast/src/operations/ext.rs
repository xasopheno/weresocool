//! The EXTENSION REGISTRY — non-audio payloads riding the point.
//!
//! weresocool is the sounds language; a `PointOp` is a *scheduled point* in
//! a composition whose core algebra (t, l, g, names) is domain-general.
//! Everything that is NOT audio synthesis — the visual extension (kintaro),
//! the MIDI side-channel, future extensions (lights, DMX, …) — lives here,
//! grouped per extension, so the core stays a pure sounds language and
//! extensions scale as new sub-structs instead of loose fields.
//!
//! THE INVARIANT: **nothing in `Ext` may parameterize synthesis.** The audio
//! engine never reads these fields. One documented exception: `visual.layer`'s
//! *presence* types the point as a visual clip, which zeroes its audio gains
//! at the RenderOp boundary — classification-by-ext, never
//! audio-parameters-from-ext.
//!
//! THE MECHANISM: `Ext::compose` opens with an exhaustive destructure, so
//! adding a new extension field is a COMPILE ERROR at exactly one site until
//! its merge law is written. That structurally kills the historical bug
//! class (hand-copied field lists in four composition sites + export paths
//! silently dropping fields nobody remembered).
//!
//! MERGE LAWS are transcribed EXACTLY from the historical per-field rules —
//! they are deliberately non-uniform; do not "clean them up":
//!   fade                → multiply            (self.fade * other.fade)
//!   layer               → self wins           (self.or(other))
//!   attach              → append              (self ++ other)
//!   colors/wgsl/midi    → append              (self ++ other)
//!   color_grading       → multiply            (ColorGrading: Mul)
//!   color_distribution  → other-if-gradient   (outer gradient wins)
//!   fit_vis             → other wins per axis (outer Fit wins)
//!   scale               → multiply (like Lm);  xa/ya/rot → add (like Fa)

use crate::color::ColorDistribution;
use crate::operations::ColorGrading;
use num_rational::{Ratio, Rational64};

/// THE EXTENSION CONTRACT — what the language requires of every extension.
/// Implementing this is how an extension plugs into the core:
///
///   * `Default` is the IDENTITY payload (a point with no extension data),
///   * `compose` is the extension's merge law, applied at every PointOp
///     composition site (Mul ×2, MulAssign, mod_by) — it must be
///     associative with `Default` as identity so normalization order
///     doesn't matter,
///   * the `Clone + Hash + Eq + Ord` bounds are what `PointOp`'s derives
///     demand of anything riding the point.
///
/// This is a STATIC registry, deliberately: extensions are fields on `Ext`,
/// not trait objects. Generics on PointOp would infect every type in the
/// system (NormalForm, Term, Defs — monomorphization everywhere); `Box<dyn>`
/// would break the Hash/Eq/Ord derives and serde. The trait names the
/// contract; the exhaustive destructure in `Ext::compose` enforces that no
/// extension is forgotten. Adding an extension = new sub-struct implementing
/// this + one field on `Ext`. Zero dynamic dispatch, zero build-time cost.
pub trait ExtensionPayload: Clone + std::hash::Hash + Eq + Ord + Default {
    /// The extension's merge law. `self` is the inner (earlier-applied)
    /// payload, `other` the outer — same convention as PointOp's Mul.
    fn compose(&self, other: &Self) -> Self;
}

/// The VISUAL extension (kintaro): everything the renderer reads off a point.
#[derive(Debug, Clone, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct VisualExt {
    /// Visual crossfade as a RATIO of the l-basis (synthesis-inert).
    /// `| Fade 1/2 | Fade 1/2` == `Fade 1/4` — multiplies like Lm.
    pub fade: Rational64,
    /// `Some(name)` = this point is a LAYER CLIP, not a note — `l` is its
    /// duration, `g` its opacity, `fade` its crossfade. Synthesis skips it
    /// BY TYPE at the RenderOp boundary; the renderable partition routes it
    /// to the visual clip table.
    pub layer: Option<String>,
    /// ATTACHED layers: this note DRIVES these layers' visibility while it
    /// sounds (the note keeps its audio — attachment is a visual side
    /// effect, unlike `layer` which types the point as a silent clip).
    /// `bd | pulse` stamps "pulse" onto bd's notes; the switching timeline
    /// is read straight off the points. Appends through composition.
    pub attach: Vec<String>,
    /// Color palette hash IDs (appended through composition).
    pub colors: Vec<u64>,
    /// WGSL brush program IDs.
    pub wgsl: Vec<u64>,
    /// Color grading adjustments (hue/saturation/brightness/vibrance/gamma).
    pub color_grading: ColorGrading,
    /// Color distribution (gradient direction + randomness mix).
    pub color_distribution: ColorDistribution,
    /// Visual Fit targets per axis (x, y, z): world-space `[a, b]` band the
    /// voice's measured extent maps onto. Outer (later-applied) Fit wins.
    pub fit_vis: [Option<(Rational64, Rational64)>; 3],
    /// LAYER PLACEMENT scale (`Sm` at term level): multiplies like Lm/Fm.
    /// 1 = fullscreen quad; 1/2 = quarter-area picture-in-picture.
    pub scale: Rational64,
    /// Placement offset (`Xa`/`Ya`), in screen fractions (1/2 = half the
    /// screen). Adds like Fa/Pa.
    pub xa: Rational64,
    pub ya: Rational64,
    /// Placement rotation (`Rz`), in TURNS (1/4 = 90°). Adds.
    pub rot: Rational64,
}

impl Default for VisualExt {
    fn default() -> Self {
        Self {
            fade: Ratio::new(1, 1),
            layer: None,
            attach: vec![],
            colors: vec![],
            wgsl: vec![],
            color_grading: ColorGrading::default(),
            color_distribution: ColorDistribution::default(),
            fit_vis: [None, None, None],
            scale: Ratio::new(1, 1),
            xa: Ratio::new(0, 1),
            ya: Ratio::new(0, 1),
            rot: Ratio::new(0, 1),
        }
    }
}

impl ExtensionPayload for VisualExt {
    fn compose(&self, other: &VisualExt) -> VisualExt {
        let VisualExt {
            fade,
            layer,
            attach,
            colors,
            wgsl,
            color_grading,
            color_distribution,
            fit_vis,
            scale,
            xa,
            ya,
            rot,
        } = self;
        VisualExt {
            fade: *fade * other.fade,
            layer: layer.clone().or(other.layer.clone()),
            attach: attach.iter().chain(&other.attach).cloned().collect(),
            colors: colors.iter().chain(&other.colors).copied().collect(),
            wgsl: wgsl.iter().chain(&other.wgsl).copied().collect(),
            color_grading: color_grading.clone() * other.color_grading.clone(),
            color_distribution: if other.color_distribution.gradient.is_some() {
                other.color_distribution.clone()
            } else {
                color_distribution.clone()
            },
            fit_vis: merge_fit(fit_vis, &other.fit_vis),
            scale: *scale * other.scale,
            xa: *xa + other.xa,
            ya: *ya + other.ya,
            rot: *rot + other.rot,
        }
    }
}

/// Per-axis merge for visual Fit: the *other* (outer) op's target wins when set.
pub fn merge_fit(
    a: &[Option<(Rational64, Rational64)>; 3],
    b: &[Option<(Rational64, Rational64)>; 3],
) -> [Option<(Rational64, Rational64)>; 3] {
    [b[0].or(a[0]), b[1].or(a[1]), b[2].or(a[2])]
}

/// The MIDI extension: external-controller routing (UDP note events).
#[derive(Debug, Clone, Default, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct MidiExt {
    /// MIDI target channels (appended through composition).
    pub channels: Vec<u8>,
}

impl ExtensionPayload for MidiExt {
    fn compose(&self, other: &MidiExt) -> MidiExt {
        let MidiExt { channels } = self;
        MidiExt {
            channels: channels.iter().chain(&other.channels).copied().collect(),
        }
    }
}

/// The extension registry carried by every point. One field per extension;
/// each owns its types and merge laws. Add a new extension = add a field
/// here + its sub-struct — `compose`'s destructure makes forgetting the
/// merge law a compile error.
#[derive(Debug, Clone, Default, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Ext {
    pub visual: VisualExt,
    pub midi: MidiExt,
}

impl ExtensionPayload for Ext {
    fn compose(&self, other: &Ext) -> Ext {
        // Exhaustive destructure: a new extension field fails HERE until
        // wired — and its type must implement ExtensionPayload to be
        // composable at all. That's the whole plug-in contract.
        let Ext { visual, midi } = self;
        Ext {
            visual: visual.compose(&other.visual),
            midi: midi.compose(&other.midi),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_laws() {
        let mut a = Ext::default();
        a.visual.fade = Ratio::new(1, 2);
        a.visual.layer = Some("self_wins".into());
        a.visual.colors = vec![1];
        a.midi.channels = vec![3];

        let mut b = Ext::default();
        b.visual.fade = Ratio::new(1, 2);
        b.visual.layer = Some("loses".into());
        b.visual.colors = vec![2];
        b.midi.channels = vec![4];

        let c = a.compose(&b);
        assert_eq!(c.visual.fade, Ratio::new(1, 4), "fade multiplies");
        assert_eq!(c.visual.layer.as_deref(), Some("self_wins"), "layer: self wins");
        assert_eq!(c.visual.colors, vec![1, 2], "colors append");
        assert_eq!(c.midi.channels, vec![3, 4], "midi appends");
    }
}

/// NORMALIZATION for the extension ops — the extension module owns its own
/// arm bodies; normalize.rs has exactly one delegating arm (`Op::Ext`).
/// `defs` is needed because `ColorGradient` writes gradients into the
/// ColorMap (the one ext op that touches shared definitions; it never
/// touches `defs.rand_ctx`).
pub fn normalize_ext_op(
    ext_op: &crate::ast::ExtOp,
    input: &mut crate::operations::NormalForm,
    defs: &mut crate::operations::Defs,
) {
    use crate::ast::ExtOp;
    use crate::wgsl::rational_to_f32;
    use crate::OscType;
    use num_traits::CheckedMul;
    use weresocool_shared::lossy_rational_mul;
    match ext_op {
        ExtOp::Color(color) => {
            input.fmap_mut(|op| {
            op.ext.visual.colors.push(*color);
            });
        }
        ExtOp::WGSL(wgsl_id) => {
            // Prepend the WGSL id so outer transforms run first
            // This allows outer Vm/Xm/etc to affect inner transforms
            input.fmap_mut(|op| {
            op.ext.visual.wgsl.insert(0, *wgsl_id);
            });
        },
        ExtOp::Fade { m } => input.fmap_mut(|op| {
            op.ext.visual.fade *= m;
        }),
        ExtOp::LayerSm { m } => input.fmap_mut(|op| {
            op.ext.visual.scale *= m;
        }),
        ExtOp::LayerXa { a } => input.fmap_mut(|op| {
            op.ext.visual.xa += a;
        }),
        ExtOp::LayerYa { a } => input.fmap_mut(|op| {
            op.ext.visual.ya += a;
        }),
        ExtOp::LayerRz { a } => input.fmap_mut(|op| {
            op.ext.visual.rot += a;
        }),
        ExtOp::Attach { name } => {
            // Visual side effect only — audio untouched; the note now
            // drives this layer's visibility for its span.
            input.fmap_mut(|op| {
                op.ext.visual.attach.push(name.clone());
            });
        }
        ExtOp::Layer { name } => {
            // The input point(s) BECOME this layer's clip(s): visual
            // payload set, audio payload cleared. l stays (clip length,
            // composable by Lm/Seq/FitLength); g stays (OPACITY, composable
            // by Gm); fade rides. Pitch/pan zeroed — no sound by shape,
            // and guaranteed silent by type at the RenderOp boundary.
            input.fmap_mut(|op| {
            op.ext.visual.layer = Some(name.clone());
            op.fm = Ratio::new(0, 1);
            op.fa = Ratio::new(0, 1);
            op.osc_type = OscType::None;
            });
        }
        ExtOp::Midi { channels } => {
            // Attach midi channels to each PointOp
            let chans = channels.clone();
            input.fmap_mut(|op| {
            op.ext.midi.channels.extend(chans.iter().cloned());
            });
        }
            // Color grading operations
        ExtOp::Hue { value } => {
            input.fmap_mut(|op| {
            op.ext.visual.color_grading.hue += value;
            });
        }
        ExtOp::Saturation { value } => {
            input.fmap_mut(|op| {
            op.ext.visual.color_grading.saturation = op
                .ext
                .visual
                .color_grading
                .saturation
                .checked_mul(value)
                .unwrap_or_else(|| lossy_rational_mul(op.ext.visual.color_grading.saturation, *value));
            });
        }
        ExtOp::Brightness { value } => {
            input.fmap_mut(|op| {
            op.ext.visual.color_grading.brightness += value;
            });
        }
        ExtOp::Vibrance { value } => {
            input.fmap_mut(|op| {
            op.ext.visual.color_grading.vibrance += value;
            });
        }
        ExtOp::Gamma { value } => {
            input.fmap_mut(|op| {
            op.ext.visual.color_grading.gamma = op
                .ext
                .visual
                .color_grading
                .gamma
                .checked_mul(value)
                .unwrap_or_else(|| lossy_rational_mul(op.ext.visual.color_grading.gamma, *value));
            });
        }
        ExtOp::ColorBlend { color_id, amount: _ } => {
            // Add the blend color and amount to colors
            // We'll store the color_id and handle blending at render time
            input.fmap_mut(|op| {
            op.ext.visual.colors.push(*color_id);
            // Store blend amount in a special way - we'll need to track this
            // For now, just add the color; blending logic will be in render
            });
        }
        ExtOp::ColorAdd { color_id } => {
            // Simply add the color to the palette
            input.fmap_mut(|op| {
            op.ext.visual.colors.push(*color_id);
            });
        }
        ExtOp::FitVis { axis, a, b } => {
            // Stamp the target band onto every note. Later (outer)
            // applications overwrite — outer Fit wins per axis.
            let band = Some((*a, *b));
            let axis = *axis.min(&2);
            input.fmap_mut(|op| {
            op.ext.visual.fit_vis[axis] = band;
            });
        }
        ExtOp::ColorGradient { x, y, z } => {
            // Set gradient direction for color distribution
            let gx = rational_to_f32(*x);
            let gy = rational_to_f32(*y);
            let gz = rational_to_f32(*z);
            let gradient = (gx, gy, gz);

            // Collect color_ids that need gradient update
            let color_ids_to_update: Vec<u64> = input.operations
            .iter()
            .flat_map(|seq| seq.iter())
            .filter_map(|op| op.ext.visual.colors.last().copied())
            .collect();

            // Update the gradient on colors in the ColorMap
            // This associates the gradient with the brush definition, not per-operation
            for color_id in color_ids_to_update {
            defs.colors.set_gradient(color_id, gradient);
            }

            // Also keep setting op.ext.visual.color_distribution for backwards compatibility
            input.fmap_mut(|op| {
            op.ext.visual.color_distribution.gradient = Some(gradient);
            // Set mix to 0 (pure gradient) when gradient is applied
            if op.ext.visual.color_distribution.mix == 1.0 {
                op.ext.visual.color_distribution.mix = 0.0;
            }
            });
        }
        ExtOp::ColorMix { amount } => {
            // Set the mix amount for color distribution (0 = pure gradient, 1 = pure random)
            let mix = rational_to_f32(*amount);
            input.fmap_mut(|op| {
            op.ext.visual.color_distribution.mix = mix;
            });
        }
    }
}
