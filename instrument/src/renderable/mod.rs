pub mod render_voice;

use crate::{Basis, Oscillator, StereoWaveform};
use num_rational::Rational64;
use num_traits::CheckedMul;
pub use render_voice::{renderables_to_render_voices, RenderVoice};
use serde::{Deserialize, Serialize};
use weresocool_ast::{
    follow::evaluate::EvaluateAction, follow::types::FollowNF, NormalForm, Normalize, OscType,
    PointOp, ASR, Distortion,
    Defs,
};
use weresocool_error::Error;
use weresocool_filter::BiquadFilterDef;
pub(crate) use weresocool_shared::{lossy_rational_mul, r_to_f64, Settings, timing_now, timing_print};
use weresocool_synth::{DistortionDef, Offset};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderOp {
    pub f: f64,
    pub p: f64,
    pub l: f64,
    pub g: (f64, f64),
    /// Time
    pub t: f64,
    pub attack: f64,
    pub decay: f64,
    pub asr: ASR,
    pub samples: usize,
    pub index: usize,
    pub total_samples: usize,
    pub voice: usize,
    pub event: usize,
    pub portamento: usize,
    pub reverb: Option<f64>,
    pub osc_type: OscType,
    pub next_l_silent: bool,
    pub next_r_silent: bool,
    pub names: Vec<String>,
    pub filters: Vec<BiquadFilterDef>,
    pub distortions: Vec<DistortionDef>,
    pub next_out: bool,
    pub follows: Vec<FollowNF>,
    /// Color hash IDs (kept as `Vec<u64>` to avoid ~4.3M stringification allocations
    /// per render on a typical heavy composition). The conversion to `Vec<String>`
    /// happens at the visualization boundary in `VisualizationAdapter`, where it's
    /// memoized over the small set of distinct IDs.
    pub colors: Vec<u64>,
    pub wgsl: Vec<u64>,
    pub midi: Vec<u8>,
    /// Scalar gain (pre-pan), derived from g * basis.g
    pub gain_scalar: f64,
    /// Color gradient direction (if Some, use gradient distribution)
    pub color_gradient: Option<(f32, f32, f32)>,
    /// Color mix: 0 = pure gradient, 1 = pure random
    pub color_mix: f32,
    /// Visual Fit bands per axis (x, y, z) in world space — from
    /// `FitX/FitY/FitZ`. Sound-inert; kintaro measures per-brush extents
    /// over ops sharing a band and solves the affine.
    pub fit_vis: [Option<(f64, f64)>; 3],
    /// Initial oscillator phase (radians) to seed at voice birth, from
    /// `PointOp.phase`. `None` = legacy behavior (integrate from 0).
    #[serde(default)]
    pub initial_phase: Option<f64>,
}

impl RenderOp {
    pub fn init_fglps(f: f64, g: (f64, f64), l: f64, p: f64, s: usize) -> Self {
        Self {
            f,
            p,
            g,
            l,
            t: 0.0,
            reverb: None,
            attack: 512_f64,
            decay: 512_f64,
            asr: ASR::Long,
            samples: s,
            total_samples: s,
            index: 0,
            voice: 0,
            event: 0,
            portamento: 512,
            osc_type: OscType::None,
            next_l_silent: false,
            next_r_silent: false,
            next_out: false,
            names: Vec::new(),
            filters: Vec::new(),
            distortions: Vec::new(),
            follows: Vec::new(),
            colors: Vec::new(),
            wgsl: Vec::new(),
            midi: Vec::new(),
            gain_scalar: 1.0,
            color_gradient: None,
            fit_vis: [None; 3],
            color_mix: 1.0,
            initial_phase: None,
        }
    }

    pub const fn init_fglp(f: f64, g: (f64, f64), l: f64, p: f64, settings: &Settings) -> Self {
        Self {
            f,
            p,
            g,
            l,
            t: 0.0,
            reverb: None,
            attack: settings.sample_rate,
            decay: settings.sample_rate,
            asr: ASR::Long,
            samples: settings.sample_rate as usize,
            total_samples: settings.sample_rate as usize,
            index: 0,
            voice: 0,
            event: 0,
            portamento: 1024,
            osc_type: OscType::None,
            next_l_silent: false,
            next_r_silent: false,
            next_out: false,
            names: Vec::new(),
            filters: Vec::new(),
            distortions: Vec::new(),
            follows: Vec::new(),
            colors: Vec::new(),
            wgsl: Vec::new(),
            midi: Vec::new(),
            gain_scalar: 1.0,
            color_gradient: None,
            fit_vis: [None; 3],
            color_mix: 1.0,
            initial_phase: None,
        }
    }
    pub fn init_silent_with_length(l: f64) -> Self {
        Self {
            f: 0.0,
            g: (0.0, 0.0),
            p: 0.0,
            l,
            t: 0.0,
            reverb: None,
            attack: Settings::global().sample_rate,
            decay: Settings::global().sample_rate,
            asr: ASR::Long,
            samples: Settings::global().sample_rate as usize,
            total_samples: Settings::global().sample_rate as usize,
            index: 0,
            voice: 0,
            event: 0,
            portamento: 1024,
            osc_type: OscType::None,
            next_l_silent: true,
            next_r_silent: true,
            next_out: false,
            names: Vec::new(),
            filters: Vec::new(),
            distortions: Vec::new(),
            follows: Vec::new(),
            colors: Vec::new(),
            wgsl: Vec::new(),
            midi: Vec::new(),
            gain_scalar: 0.0,
            color_gradient: None,
            fit_vis: [None; 3],
            color_mix: 1.0,
            initial_phase: None,
        }
    }

    pub const fn init_silent_with_length_osc_type_reverb_and_filters(
        l: f64,
        osc_type: OscType,
        reverb: Option<f64>,
        filters: Vec<BiquadFilterDef>,
        sample_rate: f64,
    ) -> Self {
        Self {
            f: 0.0,
            g: (0.0, 0.0),
            p: 0.0,
            l,
            t: 0.0,
            reverb,
            attack: sample_rate,
            decay: sample_rate,
            asr: ASR::Long,
            samples: sample_rate as usize,
            total_samples: sample_rate as usize,
            index: 0,
            voice: 0,
            event: 0,
            portamento: 1024,
            osc_type,
            next_l_silent: true,
            next_r_silent: true,
            next_out: false,
            names: vec![],
            filters,
            distortions: vec![],
            follows: vec![],
            colors: vec![],
            wgsl: Vec::new(),
            midi: Vec::new(),
            gain_scalar: 0.0,
            color_gradient: None,
            fit_vis: [None; 3],
            color_mix: 1.0,
            initial_phase: None,
        }
    }
}

// SynthOp trait implementation is at the end of the file

// Offset moved to weresocool_synth

pub trait Renderable<T> {
    fn render(&mut self, oscillator: &mut Oscillator, _offset: Option<&Offset>) -> StereoWaveform;
}

impl Renderable<RenderOp> for RenderOp {
    fn render(&mut self, oscillator: &mut Oscillator, offset: Option<&Offset>) -> StereoWaveform {
        // Only voices that author `Follow` respond to the external (mic)
        // offset — everyone else plays as written. (An empty follow chain's
        // `eval_value` passes the offset straight through, which would make
        // the mic modulate the whole piece, so guard on non-empty.)
        let o = match offset {
            Some(o) if !self.follows.is_empty() => {
                let (f, g) = self.follows.eval_value(o.freq as f32, o.gain as f32);
                Offset {
                    freq: f as f64,
                    gain: g as f64,
                }
            }
            _ => Offset::default(),
        };

        oscillator.update(self, &o);
        oscillator.generate(self, &o)
    }
}
impl Renderable<Vec<RenderOp>> for Vec<RenderOp> {
    fn render(&mut self, oscillator: &mut Oscillator, offset: Option<&Offset>) -> StereoWaveform {
        let mut result: StereoWaveform = StereoWaveform::new(0);

        // `iter_mut` + direct call avoids a deep `RenderOp::clone()` per op per buffer.
        // `render` only reads `self.follows` and forwards `&Op` into the oscillator, so the
        // clone was never necessary — it existed to satisfy the `&mut self` receiver.
        for op in self.iter_mut() {
            if op.samples > 0 {
                let stereo_waveform = op.render(oscillator, offset);
                result.append(stereo_waveform);
            }
        }

        result
    }
}

// Implement SynthOp trait from weresocool_synth
impl weresocool_synth::SynthOp for RenderOp {
    #[inline(always)]
    fn frequency(&self) -> f64 {
        self.f
    }

    #[inline(always)]
    fn gain_left(&self) -> f64 {
        self.g.0
    }

    #[inline(always)]
    fn gain_right(&self) -> f64 {
        self.g.1
    }

    #[inline(always)]
    fn pan(&self) -> f64 {
        self.p
    }

    #[inline(always)]
    fn duration_samples(&self) -> usize {
        self.samples
    }

    #[inline(always)]
    fn oscillator_type(&self) -> &OscType {
        &self.osc_type
    }

    #[inline(always)]
    fn filters(&self) -> &[BiquadFilterDef] {
        &self.filters
    }

    #[inline(always)]
    fn envelope_attack(&self) -> f64 {
        self.attack
    }

    #[inline(always)]
    fn envelope_decay(&self) -> f64 {
        self.decay
    }

    #[inline(always)]
    fn asr_type(&self) -> ASR {
        self.asr
    }

    #[inline(always)]
    fn portamento(&self) -> usize {
        self.portamento
    }

    #[inline(always)]
    fn reverb(&self) -> Option<f64> {
        self.reverb
    }

    #[inline(always)]
    fn initial_phase(&self) -> Option<f64> {
        self.initial_phase
    }

    #[inline(always)]
    fn sample_index(&self) -> usize {
        self.index
    }

    #[inline(always)]
    fn total_samples(&self) -> usize {
        self.total_samples
    }

    #[inline(always)]
    fn next_left_silent(&self) -> bool {
        self.next_l_silent
    }

    #[inline(always)]
    fn next_right_silent(&self) -> bool {
        self.next_r_silent
    }

    #[inline(always)]
    fn next_out(&self) -> bool {
        self.next_out
    }

    #[inline(always)]
    fn distortions(&self) -> &[DistortionDef] {
        &self.distortions
    }
}

fn pointop_to_renderop(
    point_op: &PointOp,
    time: &mut Rational64,
    voice: usize,
    event: usize,
    basis: &Basis,
    next: Option<&PointOp>,
    color_map: &mut weresocool_ast::color::ColorMap,
    sample_rate: f64,
) -> RenderOp {
    let mut next_l_gain = 0.0;
    let mut next_r_gain = 0.0;
    let mut next_out = false;
    let next_silent;

    match next {
        Some(op) => {
            let (l, r) = point_op_to_gains(op, basis, 1.0, 1.0);
            next_l_gain = l;
            next_r_gain = r;
            next_silent = op.is_silent();
            next_out = op.is_out;
        }

        None => next_silent = true,
    }

    let next_l_silent = next_silent || next_l_gain == 0.0;
    let next_r_silent = next_silent || next_r_gain == 0.0;

    let (f, g, p, l) = calculate_fgpl(basis, point_op);

    let render_op = RenderOp {
        f,
        g,
        p,
        l,
        t: r_to_f64(*time),
        reverb: point_op.reverb.map(r_to_f64),
        index: 0,
        samples: (l * sample_rate).round() as usize,
        total_samples: (l * sample_rate).round() as usize,
        attack: r_to_f64(point_op.attack * basis.a) * sample_rate,
        decay: r_to_f64(point_op.decay * basis.d) * sample_rate,
        osc_type: point_op.osc_type.clone(),
        asr: point_op.asr,
        portamento: (r_to_f64(point_op.portamento) * 1024_f64) as usize,
        voice,
        event,
        next_l_silent,
        next_r_silent,
        names: point_op.names.to_vec(),
        filters: point_op
            .filters
            .iter()
            .map(|f| BiquadFilterDef {
                hash: f.hash.clone(),
                filter_type: f.filter_type,
                cutoff_frequency: f.cutoff_frequency * basis.f,
                q_factor: f.q_factor,
            })
            .collect(),
        distortions: point_op
            .distortions
            .iter()
            .map(|d| match d {
                Distortion::Wavefolder { threshold, stages, input_gain, output_gain } => DistortionDef::Wavefolder {
                    threshold: r_to_f64(*threshold),
                    stages: *stages,
                    input_gain: r_to_f64(*input_gain),
                    output_gain: r_to_f64(*output_gain),
                },
                Distortion::SoftClip { threshold, input_gain, output_gain } => DistortionDef::SoftClip {
                    threshold: r_to_f64(*threshold),
                    input_gain: r_to_f64(*input_gain),
                    output_gain: r_to_f64(*output_gain),
                },
                Distortion::Overdrive { input_gain, output_gain } => DistortionDef::Overdrive {
                    input_gain: r_to_f64(*input_gain),
                    output_gain: r_to_f64(*output_gain),
                },
                Distortion::Bitcrusher { bits, input_gain, output_gain } => DistortionDef::Bitcrusher {
                    bits: *bits,
                    input_gain: r_to_f64(*input_gain),
                    output_gain: r_to_f64(*output_gain),
                },
                Distortion::Tanh { input_gain, output_gain } => DistortionDef::Tanh {
                    input_gain: r_to_f64(*input_gain),
                    output_gain: r_to_f64(*output_gain),
                },
            })
            .collect(),
        next_out,
        follows: point_op.follows.clone(),
        colors: point_op.get_transformed_colors(color_map).into_owned(),
        wgsl: point_op.wgsl.clone(),
        midi: point_op.midi.clone(),
        gain_scalar: r_to_f64(point_op.g * basis.g).clamp(0.0, 2.0),
        color_gradient: point_op.color_distribution.gradient,
        fit_vis: point_op.fit_vis.map(|band| band.map(|(a, b)| (
            *a.numer() as f64 / *a.denom() as f64,
            *b.numer() as f64 / *b.denom() as f64,
        ))),
        color_mix: point_op.color_distribution.mix,
        initial_phase: point_op.phase.map(r_to_f64),
    };

    *time += point_op.l * basis.l;

    render_op
}

pub fn point_op_to_gains(
    point_op: &PointOp,
    basis: &Basis,
    angle: f64,
    frequency: f64,
) -> (f64, f64) {
    if *point_op.g.numer() == 0 {
        return (0.0, 0.0);
    }

    let pm = r_to_f64(point_op.pm);
    let pa = r_to_f64(point_op.pa);
    let g = r_to_f64(point_op.g);
    let base_p = r_to_f64(basis.p);
    let base_g = r_to_f64(basis.g);

    let ild = calculate_ild(angle, frequency);

    let l_gain = g * (((pa.mul_add(pm, 1.0 + ild)) + base_p) / 2.0) * base_g;
    let r_gain = g * (((pa.mul_add(pm, -1.0 - ild)) + base_p) / -2.0) * base_g;

    (l_gain, r_gain)
}

fn calculate_ild(angle: f64, frequency: f64) -> f64 {
    const MAX_ILD: f64 = 1.0;
    const FREQUENCY_FACTOR: f64 = 0.001;

    let angle_factor = (angle / 90.0).cos();
    let frequency_factor = frequency * FREQUENCY_FACTOR;

    MAX_ILD * angle_factor * frequency_factor
}

pub fn m_a_and_basis_to_f64(basis: Rational64, m: Rational64, a: Rational64) -> f64 {
    r_to_f64(
        basis
            .checked_mul(&m)
            .unwrap_or_else(|| lossy_rational_mul(basis, m)),
    ) + r_to_f64(a)
}

pub fn float_to_angle(input: f64) -> f64 {
    // Map the input in the range -1 to 1 into the range -90 to 90.
    input * 90.0
}

pub fn calculate_fgpl(basis: &Basis, point_op: &PointOp) -> (f64, (f64, f64), f64, f64) {
    let settings = Settings::global();
    let p = m_a_and_basis_to_f64(basis.p, point_op.pm, point_op.pa);

    let (mut f, mut g) = if point_op.is_silent() {
        (0.0, (0.0, 0.0))
    } else {
        let f = m_a_and_basis_to_f64(basis.f, point_op.fm, point_op.fa);
        let g = point_op_to_gains(point_op, basis, f, float_to_angle(p));
        (f, g)
    };
    let l = r_to_f64(point_op.l * basis.l);
    if f < settings.min_freq {
        f = 0.0;
        g = (0.0, 0.0);
    };

    (f, g, p, l)
}

pub fn nf_to_vec_renderable(
    composition: &NormalForm,
    defs: &mut Defs,
    basis: &Basis,
) -> Result<Vec<Vec<RenderOp>>, Error> {
    // We have `composition: &NormalForm` already and only read `.operations` below,
    // so cloning the whole NF was pure waste — a previous comment claimed clone was
    // "much faster than the old approach", but the actual cheapest option is no clone
    // at all. Saves ~110ms on drum_sounds.socool (634 voices, 714k ops).
    let settings = Settings::global();

    let render_start = timing_now!();
    let result: Vec<Vec<RenderOp>> = composition
        .operations
        .iter()
        .enumerate()
        .map(|(voice, vec_point_op)| {
            create_render_ops(
                voice,
                vec_point_op,
                basis,
                settings.sample_rate,
                settings.pad_end,
                &mut defs.colors,
            )
        })
        .collect();
    timing_print!("[nf_to_vec_renderable] create_render_ops: {:?} ({} voices, {} total ops)",
        render_start.elapsed(),
        composition.operations.len(),
        composition.operations.iter().map(|v| v.len()).sum::<usize>());

    Ok(result)
}

fn create_render_ops(
    voice: usize,
    vec_point_op: &[PointOp],
    basis: &Basis,
    sample_rate: f64,
    pad_end: bool,
    color_map: &mut weresocool_ast::color::ColorMap,
) -> Vec<RenderOp> {
    let mut time = Rational64::new(0, 1);
    // Pre-allocate to avoid reallocations
    let capacity = vec_point_op.len() + if pad_end { 1 } else { 0 };
    let mut result: Vec<RenderOp> = Vec::with_capacity(capacity);

    let len = vec_point_op.len();
    for (event, p_op) in vec_point_op.iter().enumerate() {
        let is_last = event == len - 1;
        // If pad_end is true and this is the last op, next should be None
        // so decay envelope applies. Otherwise wrap to first op for looping.
        // Pass reference instead of cloning to avoid 1.3M clones.
        let next_op: Option<&PointOp> = if is_last && pad_end {
            None
        } else {
            let next_e = if is_last { 0 } else { event + 1 };
            Some(&vec_point_op[next_e])
        };
        let op = pointop_to_renderop(
            p_op,
            &mut time,
            voice,
            event,
            basis,
            next_op,
            color_map,
            sample_rate,
        );
        result.push(op);
    }

    if pad_end {
        let filters = if let Some(last_op) = vec_point_op.last() {
            last_op.filters.to_vec()
        } else {
            vec![]
        };
        result.push(
            RenderOp::init_silent_with_length_osc_type_reverb_and_filters(
                1.0,
                OscType::None,
                None,
                filters,
                sample_rate,
            ),
        );
    }

    result
}

#[cfg(test)]
mod phase_tests {
    use super::*;
    use weresocool_synth::Oscillator;

    /// A fresh `Oscillator` starts silent (`offset_past.gain == 0`), so the
    /// first op is a true silence→sound birth and the phase seed fires. Gain
    /// ramps linearly from 0, so sample[0] is 0 regardless of phase — the seed
    /// shows from the next sample on: with `Some(π/2)` the waveform sits near
    /// the sine peak (cos≈1), with `None` it starts near the zero crossing
    /// (sin≈0). Gain ramps are identical, so seeded[1] dominates unseeded[1].
    #[test]
    fn initial_phase_seeds_birth_sample() {
        Settings::init_test();
        let sr = Settings::global().sample_rate;
        let samples = (0.1 * sr) as usize;

        let make_op = |phase: Option<f64>| {
            let mut op = RenderOp::init_fglps(440.0, (1.0, 1.0), 0.1, 0.0, samples);
            op.osc_type = OscType::Sine { pow: None };
            op.initial_phase = phase;
            op
        };

        let render = |phase: Option<f64>| {
            let mut osc = Oscillator::init();
            let op = make_op(phase);
            osc.update(&op, &Offset::default());
            osc.generate(&op, &Offset::default())
        };

        let seeded = render(Some(std::f64::consts::FRAC_PI_2));
        let unseeded = render(None);

        // sample[0] is 0 for both (gain ramp starts at 0); the seed's effect is
        // visible once the ramp lifts off — compare the first non-zero sample.
        assert_eq!(seeded.l_buffer[0], 0.0);
        assert!(
            seeded.l_buffer[1].abs() > unseeded.l_buffer[1].abs() * 5.0,
            "seeded sample[1] {} should dominate unseeded {}",
            seeded.l_buffer[1],
            unseeded.l_buffer[1],
        );
    }
}
