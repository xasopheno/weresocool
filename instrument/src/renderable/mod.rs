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
pub(crate) use weresocool_shared::{lossy_rational_mul, r_to_f64, Settings};
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
    pub colors: Vec<String>,
    pub wgsl: Vec<u64>,
    pub midi: Vec<u8>,
    /// Scalar gain (pre-pan), derived from g * basis.g
    pub gain_scalar: f64,
    /// Color gradient direction (if Some, use gradient distribution)
    pub color_gradient: Option<(f32, f32, f32)>,
    /// Color mix: 0 = pure gradient, 1 = pure random
    pub color_mix: f32,
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
            color_mix: 1.0,
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
            color_mix: 1.0,
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
            color_mix: 1.0,
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
            color_mix: 1.0,
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
        let o = match offset {
            Some(o) => {
                let (f, g) = self.follows.eval_value(o.freq as f32, o.gain as f32);
                Offset {
                    freq: f as f64,
                    gain: g as f64,
                }
            }
            None => Offset::default(),
        };

        oscillator.update(self, &o);
        oscillator.generate(self, &o)
    }
}
impl Renderable<Vec<RenderOp>> for Vec<RenderOp> {
    fn render(&mut self, oscillator: &mut Oscillator, offset: Option<&Offset>) -> StereoWaveform {
        let mut result: StereoWaveform = StereoWaveform::new(0);

        for op in self.iter() {
            if op.samples > 0 {
                let stereo_waveform = op.clone().render(oscillator, offset);
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
        colors: point_op.get_transformed_colors(color_map).iter().map(|c| c.to_string()).collect(),
        wgsl: point_op.wgsl.clone(),
        midi: point_op.midi.clone(),
        gain_scalar: r_to_f64(point_op.g * basis.g).clamp(0.0, 2.0),
        color_gradient: point_op.color_distribution.gradient,
        color_mix: point_op.color_distribution.mix,
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
    // No need to apply_to_normal_form - composition is already normalized.
    // The old code did `NormalForm::init() *= composition` which just copies
    // through expensive nested loops (O(n*m) for n voices, m ops).
    // Clone is much faster.
    let clone_start = std::time::Instant::now();
    let normal_form = composition.clone();
    eprintln!("[nf_to_vec_renderable] clone: {:?} ({} voices, {} total ops)",
        clone_start.elapsed(),
        normal_form.operations.len(),
        normal_form.operations.iter().map(|v| v.len()).sum::<usize>());

    let settings = Settings::global();

    let render_start = std::time::Instant::now();
    let result: Vec<Vec<RenderOp>> = normal_form
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
    eprintln!("[nf_to_vec_renderable] create_render_ops: {:?}", render_start.elapsed());

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
