//! Built-in drum preset tables + per-note parameter resolution.
//!
//! Design: presets are not flat knob settings — they replace the *formula
//! coefficients* behind the macro knobs. Today's default kick computes
//! `click_amount = 0.10 + attack × 0.35`; a preset supplies its own
//! `(base, slope)` pair, so the user's `attack` macro keeps working on every
//! preset, scaled to that preset's character. Resolution precedence per
//! field: explicit user param → preset formula evaluated at (user macro,
//! else preset macro center).
//!
//! Presets also carry INTERNAL knobs with no DSL exposure (pitch-envelope
//! shape, cymbal mode scale, wire tunings, compressor settings). This is
//! what makes `Kick 909` genuinely different from `Kick 808` instead of
//! just "different numbers on the same machine" — while the language
//! surface stays small.
//!
//! The preset NAMES are declared in `weresocool_ast::drum_presets` (the
//! parser validates against them at parse time); `tables_match_ast_names`
//! below keeps the two in sync. `wsc` is the signature voicing and is what
//! a bare `Kick` / `Snare` / `HiHat` resolves to.
//!
//! Everything here is resolved ONCE per note (cached in `DrumState`,
//! invalidated by `DrumState::reset()` on each note-on) — it also removes
//! the per-sample `Rational64 → f64` conversion chains the drums used to do.

use weresocool_ast::{ClapParams, HiHatParams, KickParams, RimshotParams, SnareParams};
use weresocool_shared::r_to_f64;

/// Per-drum parallel-compression settings (threshold/ratio are the
/// soft-knee curve; dry/wet the parallel blend; times in seconds).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Comp {
    pub threshold: f64,
    pub ratio: f64,
    pub dry: f64,
    pub wet: f64,
    pub attack_s: f64,
    pub release_s: f64,
}

/// Kick pitch-envelope shape.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum KickPitchEnv {
    /// Two exponentials: a fast "knee" carrying `fast_w` of the drop, and a
    /// slower settle. Time constants are fractions of `pitch_decay`.
    /// The classic 808 shape. (wsc/808: fast_w 0.65, fracs 0.10 / 0.333…)
    TwoStage { fast_w: f64, fast_frac: f64, slow_frac: f64 },
    /// One exponential at `pitch_decay` — the punchier 909 shape.
    SingleExp,
}

impl KickPitchEnv {
    /// Returns `(exp_pd, integral)` — the instantaneous envelope value and
    /// its time-integral, both needed for phase-continuous pitch sweeps.
    #[inline]
    pub fn eval(&self, t: f64, pitch_decay: f64) -> (f64, f64) {
        match *self {
            KickPitchEnv::TwoStage { fast_w, fast_frac, slow_frac } => {
                let tau_fast = pitch_decay * fast_frac;
                let tau_slow = pitch_decay * slow_frac;
                let exp_fast = (-t / tau_fast).exp();
                let exp_slow = (-t / tau_slow).exp();
                let slow_w = 1.0 - fast_w;
                let exp_pd = fast_w * exp_fast + slow_w * exp_slow;
                let integral = fast_w * tau_fast * (1.0 - exp_fast)
                    + slow_w * tau_slow * (1.0 - exp_slow);
                (exp_pd, integral)
            }
            KickPitchEnv::SingleExp => {
                let exp_pd = (-t / pitch_decay).exp();
                (exp_pd, pitch_decay * (1.0 - exp_pd))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// KICK
// ═══════════════════════════════════════════════════════════════════════

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct KickPreset {
    // Macro centers — what the knobs mean when the user omits them.
    pub attack: f64,
    pub body: f64,
    pub tone: f64,
    pub length: f64,
    // Formula coefficients: value = k.0 + k.1 × macro.
    pub amp_decay_k: (f64, f64),    // on length
    pub click_amount_k: (f64, f64), // on attack
    pub click_freq_k: (f64, f64),   // on tone
    pub saturation_k: (f64, f64),   // on body
    pub shell_k: (f64, f64),        // on body
    // Direct param defaults.
    pub pitch_decay: f64,
    pub pitch_range: f64,
    pub hump: f64,
    pub ks_mix: f64,
    pub velocity_tilt: f64,
    // ── Internal-only knobs (no DSL exposure) ──
    pub pitch_env: KickPitchEnv,
    /// Additive-sine level under the KS body (today: 0.55).
    pub sine_level: f64,
    /// Self-FM warmth depth during the pitch sweep (today: 0.6).
    pub fm_depth: f64,
    /// Body lowpass sweep, multiples of f_base: (transient hi, settled lo).
    pub cutoff_mult: (f64, f64),
    pub body_q: f64,
    /// Click bandpass ring make-up gain (today: 4.5).
    pub click_gain: f64,
    pub comp: Comp,
    /// Extra drive into the output tape saturator — "knock" pushes this.
    pub drive_out: f64,
    /// Loudness calibration trim. Set from `measure_preset_loudness`.
    pub gain_trim: f64,
}

/// Today's engine constants, verbatim — the baseline every preset diverges
/// from, and the bit-identical behavior for params-only compositions.
const KICK_WSC: KickPreset = KickPreset {
    attack: 0.5,
    body: 0.5,
    tone: 0.5,
    length: 0.5,
    amp_decay_k: (0.35, 0.55),
    click_amount_k: (0.10, 0.35),
    click_freq_k: (1700.0, 1300.0),
    saturation_k: (0.40, 0.30),
    shell_k: (0.45, 0.20),
    pitch_decay: 0.045,
    pitch_range: 4.5,
    hump: 0.5,
    ks_mix: 0.85,
    velocity_tilt: 0.5,
    pitch_env: KickPitchEnv::TwoStage { fast_w: 0.65, fast_frac: 0.10, slow_frac: 1.0 / 3.0 },
    sine_level: 0.55,
    fm_depth: 0.6,
    cutoff_mult: (6.0, 1.5),
    body_q: 1.2,
    click_gain: 4.5,
    comp: Comp { threshold: 0.25, ratio: 5.0, dry: 0.55, wet: 0.90, attack_s: 0.0008, release_s: 0.080 },
    drive_out: 1.0,
    gain_trim: 1.0,
};

/// 808: long sub boom. Sine-dominant body, slow settle, almost no click,
/// gentle saturation — the weight comes from the fundamental, not drive.
const KICK_808: KickPreset = KickPreset {
    length: 0.7,
    amp_decay_k: (0.45, 0.65),
    click_amount_k: (0.02, 0.15),
    click_freq_k: (1400.0, 800.0),
    saturation_k: (0.25, 0.25),
    shell_k: (0.30, 0.15),
    pitch_decay: 0.060,
    pitch_range: 3.2,
    hump: 0.9,
    ks_mix: 0.45,
    sine_level: 0.85,
    fm_depth: 0.35,
    cutoff_mult: (4.0, 1.3),
    body_q: 1.1,
    comp: Comp { threshold: 0.30, ratio: 3.5, dry: 0.65, wet: 0.75, attack_s: 0.0010, release_s: 0.120 },
    gain_trim: 0.718,
    ..KICK_WSC
};

/// 909: the dance-floor kick. Single-exponential pitch drop, prominent
/// beater click, tighter decay, harder drive, less physical-model softness.
const KICK_909: KickPreset = KickPreset {
    attack: 0.7,
    length: 0.35,
    amp_decay_k: (0.22, 0.40),
    click_amount_k: (0.30, 0.45),
    click_freq_k: (2200.0, 1200.0),
    saturation_k: (0.55, 0.35),
    shell_k: (0.30, 0.15),
    pitch_decay: 0.045,
    pitch_range: 5.5,
    hump: 0.7,
    ks_mix: 0.35,
    pitch_env: KickPitchEnv::SingleExp,
    sine_level: 0.80,
    fm_depth: 0.8,
    cutoff_mult: (8.0, 1.8),
    body_q: 1.3,
    click_gain: 5.5,
    comp: Comp { threshold: 0.22, ratio: 6.0, dry: 0.50, wet: 1.00, attack_s: 0.0006, release_s: 0.060 },
    drive_out: 1.15,
    gain_trim: 0.902,
    ..KICK_WSC
};

/// knock: trap kick — 808 bones, but shorter, driven hard into the tape
/// stage and compressed heavily so it knocks on small speakers.
const KICK_KNOCK: KickPreset = KickPreset {
    body: 0.7,
    length: 0.45,
    amp_decay_k: (0.30, 0.45),
    click_amount_k: (0.06, 0.20),
    click_freq_k: (1500.0, 900.0),
    saturation_k: (0.60, 0.35),
    shell_k: (0.35, 0.15),
    pitch_decay: 0.050,
    pitch_range: 3.8,
    hump: 1.4,
    ks_mix: 0.55,
    sine_level: 0.80,
    fm_depth: 0.5,
    cutoff_mult: (4.5, 1.4),
    body_q: 1.25,
    comp: Comp { threshold: 0.18, ratio: 7.0, dry: 0.45, wet: 1.05, attack_s: 0.0008, release_s: 0.070 },
    drive_out: 1.5,
    gain_trim: 0.706,
    ..KICK_WSC
};

/// acoustic: KS waveguide dominant, audible wooden shell ring, soft wide
/// click, minimal saturation — a played drum, not a programmed one.
const KICK_ACOUSTIC: KickPreset = KickPreset {
    attack: 0.6,
    // A PLAYED drum: the felt beater slapping the head is most of what
    // you actually hear from a real kick — prominent wide click in the
    // 2.6-4 kHz register, wooden shell knock, shorter sub, minimal pitch
    // sweep (real heads barely glide), low saturation.
    amp_decay_k: (0.22, 0.28),
    click_amount_k: (0.28, 0.30),
    click_freq_k: (2600.0, 1400.0),
    saturation_k: (0.18, 0.15),
    shell_k: (0.85, 0.25),
    pitch_decay: 0.030,
    pitch_range: 2.2,
    hump: 0.5,
    ks_mix: 1.05,
    velocity_tilt: 0.7,
    sine_level: 0.35,
    fm_depth: 0.3,
    cutoff_mult: (7.0, 1.6),
    body_q: 1.0,
    click_gain: 5.0,
    comp: Comp { threshold: 0.30, ratio: 3.0, dry: 0.70, wet: 0.60, attack_s: 0.0012, release_s: 0.100 },
    gain_trim: 1.853,
    ..KICK_WSC
};

/// dust: lofi — dark, soft, short. The kick on a worn record.
const KICK_DUST: KickPreset = KickPreset {
    tone: 0.3,
    length: 0.4,
    amp_decay_k: (0.25, 0.35),
    click_amount_k: (0.04, 0.10),
    click_freq_k: (900.0, 500.0),
    saturation_k: (0.45, 0.25),
    shell_k: (0.35, 0.15),
    pitch_decay: 0.040,
    pitch_range: 2.4,
    hump: 0.6,
    ks_mix: 0.70,
    velocity_tilt: 0.3,
    sine_level: 0.70,
    fm_depth: 0.25,
    cutoff_mult: (2.8, 1.1),
    body_q: 0.9,
    click_gain: 3.0,
    comp: Comp { threshold: 0.28, ratio: 4.0, dry: 0.60, wet: 0.80, attack_s: 0.0010, release_s: 0.090 },
    drive_out: 1.2,
    gain_trim: 0.977,
    ..KICK_WSC
};

/// glass: a tuned bell-kick — almost pure sine (ks_mix near zero), a sparkly
/// high click, and barely any pitch glide so it reads as a pitched musical
/// tone rather than a thump. Clean: saturation ~0, wide-open body filter.
const KICK_GLASS: KickPreset = KickPreset {
    tone: 0.8,
    amp_decay_k: (0.30, 0.45),
    click_amount_k: (0.18, 0.25),
    click_freq_k: (3200.0, 1600.0),
    saturation_k: (0.05, 0.08),
    shell_k: (0.25, 0.12),
    pitch_decay: 0.030,
    pitch_range: 1.9,
    hump: 0.4,
    ks_mix: 0.15,
    sine_level: 0.95,
    fm_depth: 0.15,
    cutoff_mult: (10.0, 3.0),
    body_q: 1.6,
    click_gain: 6.0,
    comp: Comp { threshold: 0.32, ratio: 3.0, dry: 0.68, wet: 0.65, attack_s: 0.0012, release_s: 0.090 },
    gain_trim: 1.057,
    ..KICK_WSC
};

/// doom: a deep cavernous sub-kick — long slow pitch glide, pure low sine,
/// very dark body filter, huge hump. Massive and CLEAN (the distortion is
/// `crush`'s job) — this one is felt more than heard.
const KICK_DOOM: KickPreset = KickPreset {
    body: 0.7,
    tone: 0.2,
    length: 0.85,
    amp_decay_k: (0.55, 0.75),
    click_amount_k: (0.02, 0.08),
    click_freq_k: (700.0, 400.0),
    saturation_k: (0.35, 0.25),
    shell_k: (0.45, 0.20),
    pitch_decay: 0.085,
    pitch_range: 3.5,
    hump: 1.3,
    ks_mix: 0.30,
    sine_level: 0.92,
    fm_depth: 0.25,
    cutoff_mult: (2.4, 0.9),
    body_q: 1.0,
    click_gain: 2.5,
    comp: Comp { threshold: 0.26, ratio: 4.5, dry: 0.55, wet: 0.95, attack_s: 0.0012, release_s: 0.150 },
    gain_trim: 0.638,
    ..KICK_WSC
};

/// crush: a distorted industrial kick — saturation slammed, self-FM grit,
/// and `drive_out` pushed hard into the tape clipper so the body squares
/// off. Bright enough that the distortion harmonics scream.
const KICK_CRUSH: KickPreset = KickPreset {
    attack: 0.7,
    body: 0.9,
    tone: 0.6,
    length: 0.45,
    amp_decay_k: (0.30, 0.40),
    click_amount_k: (0.20, 0.35),
    click_freq_k: (1800.0, 1000.0),
    saturation_k: (0.95, 0.40),
    shell_k: (0.35, 0.15),
    pitch_decay: 0.050,
    pitch_range: 4.2,
    hump: 1.2,
    ks_mix: 0.55,
    sine_level: 0.70,
    fm_depth: 1.4,
    cutoff_mult: (6.0, 1.8),
    body_q: 1.4,
    click_gain: 4.5,
    comp: Comp { threshold: 0.14, ratio: 9.0, dry: 0.35, wet: 1.15, attack_s: 0.0007, release_s: 0.070 },
    drive_out: 2.8,
    gain_trim: 0.518,
    ..KICK_WSC
};

/// air: a soft felt-mallet thud — NO click (click_amount ~0), low hump,
/// gentle KS body, clean. All breath and round low end, no transient slap.
const KICK_AIR: KickPreset = KickPreset {
    attack: 0.2,
    body: 0.4,
    tone: 0.4,
    amp_decay_k: (0.40, 0.50),
    click_amount_k: (0.00, 0.04),
    click_freq_k: (900.0, 400.0),
    saturation_k: (0.08, 0.10),
    shell_k: (0.20, 0.10),
    pitch_decay: 0.060,
    pitch_range: 2.4,
    hump: 0.25,
    ks_mix: 0.60,
    sine_level: 0.80,
    fm_depth: 0.20,
    cutoff_mult: (3.0, 1.1),
    body_q: 0.9,
    click_gain: 2.0,
    comp: Comp { threshold: 0.32, ratio: 2.5, dry: 0.70, wet: 0.60, attack_s: 0.0015, release_s: 0.120 },
    drive_out: 0.9,
    gain_trim: 1.041,
    ..KICK_WSC
};

/// snap: an ultra-tight gated kick — click-forward with a tiny fast-decaying
/// thump and a snappy pitch sweep. Almost no body tail (amp_decay slammed
/// short). A "tick + boop" with zero sustain.
const KICK_SNAP: KickPreset = KickPreset {
    attack: 0.85,
    tone: 0.6,
    length: 0.15,
    amp_decay_k: (0.10, 0.18),
    click_amount_k: (0.40, 0.45),
    click_freq_k: (2200.0, 1200.0),
    saturation_k: (0.30, 0.25),
    shell_k: (0.20, 0.10),
    pitch_decay: 0.025,
    pitch_range: 5.5,
    hump: 1.0,
    ks_mix: 0.25,
    sine_level: 0.85,
    fm_depth: 0.50,
    cutoff_mult: (7.0, 1.6),
    body_q: 1.3,
    click_gain: 5.5,
    comp: Comp { threshold: 0.20, ratio: 6.0, dry: 0.55, wet: 0.90, attack_s: 0.0006, release_s: 0.040 },
    drive_out: 1.1,
    gain_trim: 1.069,
    ..KICK_WSC
};

pub fn kick_preset(name: Option<&str>) -> &'static KickPreset {
    match name {
        Some("808") => &KICK_808,
        Some("909") => &KICK_909,
        Some("knock") => &KICK_KNOCK,
        Some("acoustic") => &KICK_ACOUSTIC,
        Some("dust") => &KICK_DUST,
        Some("glass") => &KICK_GLASS,
        Some("doom") => &KICK_DOOM,
        Some("crush") => &KICK_CRUSH,
        Some("air") => &KICK_AIR,
        Some("snap") => &KICK_SNAP,
        // `wsc`, bare drums, and anything unexpected (parser validates, so
        // unexpected means version skew — fail soft, not loud).
        _ => &KICK_WSC,
    }
}

/// Flat per-note kick parameters. `Copy` so the per-sample read is free.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ResolvedKick {
    pub tune: f64,
    pub pitch_decay: f64,
    pub pitch_range: f64,
    pub amp_decay: f64,
    pub click_amount: f64,
    pub click_freq: f64,
    pub saturation: f64,
    pub hump: f64,
    pub shell: f64,
    pub ks_mix: f64,
    pub velocity_tilt: f64,
    pub internal: &'static KickPreset,
}

#[inline]
fn p(o: Option<num_rational::Rational64>, default: f64) -> f64 {
    o.map(r_to_f64).unwrap_or(default)
}

pub fn resolve_kick(params: Option<&KickParams>) -> ResolvedKick {
    let pre = kick_preset(params.and_then(|p| p.preset.as_deref()));
    let attack = p(params.and_then(|x| x.attack), pre.attack);
    let body = p(params.and_then(|x| x.body), pre.body);
    let tone = p(params.and_then(|x| x.tone), pre.tone);
    let length = p(params.and_then(|x| x.length), pre.length);
    ResolvedKick {
        tune: p(params.and_then(|x| x.tune), 1.0),
        pitch_decay: p(params.and_then(|x| x.pitch_decay), pre.pitch_decay),
        pitch_range: p(params.and_then(|x| x.pitch_range), pre.pitch_range),
        amp_decay: p(params.and_then(|x| x.amp_decay), pre.amp_decay_k.0 + length * pre.amp_decay_k.1),
        click_amount: p(params.and_then(|x| x.click_amount), pre.click_amount_k.0 + attack * pre.click_amount_k.1),
        click_freq: p(params.and_then(|x| x.click_freq), pre.click_freq_k.0 + tone * pre.click_freq_k.1),
        saturation: p(params.and_then(|x| x.saturation), pre.saturation_k.0 + body * pre.saturation_k.1),
        hump: p(params.and_then(|x| x.hump), pre.hump),
        shell: p(params.and_then(|x| x.shell), pre.shell_k.0 + body * pre.shell_k.1),
        ks_mix: p(params.and_then(|x| x.ks_mix), pre.ks_mix),
        velocity_tilt: p(params.and_then(|x| x.velocity_tilt), pre.velocity_tilt),
        internal: pre,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SNARE
// ═══════════════════════════════════════════════════════════════════════

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SnarePreset {
    // Macro centers.
    pub attack: f64,
    pub wires: f64,
    pub tone: f64,
    pub length: f64,
    // Formula coefficients: value = k.0 + k.1 × macro.
    pub shell_decay_k: (f64, f64),       // on length
    pub wire_decay_k: (f64, f64),        // on wires
    pub wire_mix_k: (f64, f64),          // on wires
    pub attack_amount_k: (f64, f64),     // on attack
    pub shell_pitch_range_k: (f64, f64), // on attack
    pub head_damping_k: (f64, f64),      // on tone (slope is negative today)
    pub crack_k: (f64, f64),             // on attack
    pub bright_shift_k: (f64, f64),      // on tone — scales the wire band freqs
    // Direct param defaults.
    pub shell_tune: f64,
    pub shell_pitch_decay: f64,
    pub saturation: f64,
    pub crack_freq: f64,
    pub ks_mix: f64,
    pub velocity_tilt: f64,
    // ── Internal-only knobs ──
    /// The snare's natural fundamental in Hz, and how strongly it tracks the
    /// played note. `f_base = base_pitch · tune · (note/185)^pitch_track`:
    /// `pitch_track` 1.0 fully tracks the note (the classic behavior — a
    /// 185 Hz `base_pitch` then reproduces `f_base = note·tune` exactly);
    /// lower values pin the drum to its own register so a low bass root
    /// doesn't drag the snare into the mud. Each character kit sits in its
    /// own register, which is most of what makes them sound like different
    /// drums rather than one drum re-EQ'd.
    pub base_pitch: f64,
    pub pitch_track: f64,
    /// The four wire resonance frequencies (Hz) before `bright_shift`.
    pub wire_freqs: [f64; 4],
    pub wire_qs: [f64; 4],
    pub wire_gains: [f64; 4],
    /// Broadband highpassed-noise bed under the resonant wire bands —
    /// the dense "rattle"; bands alone whistle.
    pub wire_bed_gain: f64,
    /// Head mode ratios: (top second mode, bottom second mode).
    pub mode_ratios: (f64, f64),
    /// Head mode amplitudes: top1, top2, bot1, bot2.
    pub mode_amps: [f64; 4],
    /// Beater bandpass (center Hz, Q) and ring make-up gain.
    pub beater_bp: (f64, f64),
    pub beater_gain: f64,
    /// Crack comb taps (spike, body) — sets the noise-burst register.
    pub crack_taps: (usize, usize),
    pub crack_gain: f64,
    /// Mid-band crack waveshaper: drive = base + vel × slope; mix likewise.
    pub mid_crack_drive_k: (f64, f64),
    pub mid_crack_mix_k: (f64, f64),
    pub comp: Comp,
    pub drive_out: f64,
    pub gain_trim: f64,
}

const SNARE_WSC: SnarePreset = SnarePreset {
    attack: 0.5,
    wires: 0.5,
    tone: 0.5,
    length: 0.5,
    shell_decay_k: (0.09, 0.10),
    wire_decay_k: (0.14, 0.16),
    wire_mix_k: (0.35, 0.35),
    attack_amount_k: (0.18, 0.32),
    // Subtle and FAST — wide slow pitch glides read as "boing", the
    // classic fake-snare tell. Real heads settle within ~20 ms.
    shell_pitch_range_k: (0.08, 0.16),
    head_damping_k: (1.8, -0.7),
    crack_k: (0.2, 0.4),
    bright_shift_k: (1.0, 0.4),
    shell_tune: 1.74,
    shell_pitch_decay: 60.0,
    saturation: 0.20,
    crack_freq: 3000.0,
    ks_mix: 0.55,
    velocity_tilt: 0.5,
    // 185 Hz @ full tracking → f_base = note·tune (the historical behavior;
    // every preset that doesn't override these is bit-identical).
    base_pitch: 185.0,
    pitch_track: 1.0,
    wire_freqs: [5800.0, 9700.0, 13300.0, 16200.0],
    wire_qs: [4.0, 3.4, 2.8, 2.3],
    wire_gains: [1.8, 1.3, 0.85, 0.55],
    wire_bed_gain: 1.1,
    mode_ratios: (1.59, 1.51),
    mode_amps: [0.36, 0.20, 0.26, 0.14],
    beater_bp: (3500.0, 5.0),
    beater_gain: 4.6,
    crack_taps: (6, 10),
    crack_gain: 1.9,
    // Drive ceiling kept moderate: at full velocity the old (1.5, 4.0)
    // slope pushed the 2-5 kHz tap to 7.5× tanh — buzzy, not cracky.
    mid_crack_drive_k: (1.5, 2.5),
    mid_crack_mix_k: (0.25, 0.40),
    comp: Comp { threshold: 0.30, ratio: 4.5, dry: 0.55, wet: 0.90, attack_s: 0.005, release_s: 0.060 },
    drive_out: 0.60,
    gain_trim: 1.0,
};

/// 808: the tight noise-forward snap. Wires scaled dark, short shell,
/// little tonal body — closer to a shaped noise burst than a drum.
const SNARE_808: SnarePreset = SnarePreset {
    wires: 0.65,
    shell_decay_k: (0.07, 0.08),
    wire_decay_k: (0.10, 0.12),
    wire_mix_k: (0.50, 0.35),
    attack_amount_k: (0.10, 0.20),
    shell_pitch_range_k: (0.08, 0.15),
    crack_k: (0.15, 0.30),
    bright_shift_k: (0.85, 0.30),
    shell_tune: 1.80,
    saturation: 0.30,
    crack_freq: 2500.0,
    ks_mix: 0.30,
    wire_gains: [1.9, 1.2, 0.6, 0.35],
    wire_bed_gain: 1.45,
    mode_amps: [0.30, 0.12, 0.18, 0.08],
    beater_bp: (2800.0, 4.0),
    beater_gain: 3.0,
    crack_gain: 1.6,
    comp: Comp { threshold: 0.28, ratio: 4.0, dry: 0.60, wet: 0.85, attack_s: 0.005, release_s: 0.070 },
    gain_trim: 0.921,
    ..SNARE_WSC
};

/// 909: tonal body + a bright wide noise burst, punchy compression.
const SNARE_909: SnarePreset = SnarePreset {
    attack: 0.6,
    tone: 0.7,
    shell_decay_k: (0.10, 0.10),
    wire_decay_k: (0.12, 0.14),
    wire_mix_k: (0.40, 0.35),
    attack_amount_k: (0.25, 0.35),
    shell_pitch_range_k: (0.10, 0.18),
    crack_k: (0.30, 0.45),
    bright_shift_k: (1.10, 0.40),
    shell_tune: 1.65,
    saturation: 0.35,
    crack_freq: 3500.0,
    ks_mix: 0.40,
    wire_bed_gain: 1.55,
    mode_ratios: (1.50, 1.45),
    mode_amps: [0.42, 0.22, 0.22, 0.10],
    beater_bp: (4000.0, 5.0),
    beater_gain: 4.5,
    crack_gain: 2.1,
    comp: Comp { threshold: 0.24, ratio: 5.5, dry: 0.50, wet: 1.00, attack_s: 0.004, release_s: 0.050 },
    drive_out: 0.70,
    gain_trim: 0.804,
    ..SNARE_WSC
};

/// trap: bright, short, aggressive mid-band crack — the modern produced snap.
const SNARE_TRAP: SnarePreset = SnarePreset {
    attack: 0.7,
    tone: 0.7,
    shell_decay_k: (0.08, 0.08),
    wire_decay_k: (0.08, 0.10),
    wire_mix_k: (0.40, 0.30),
    attack_amount_k: (0.30, 0.40),
    shell_pitch_range_k: (0.15, 0.22),
    crack_k: (0.40, 0.45),
    bright_shift_k: (1.15, 0.40),
    saturation: 0.45,
    crack_freq: 4200.0,
    ks_mix: 0.40,
    wire_gains: [1.6, 1.4, 1.0, 0.7],
    wire_bed_gain: 1.45,
    beater_bp: (4200.0, 5.5),
    beater_gain: 5.0,
    crack_taps: (5, 8),
    crack_gain: 2.4,
    mid_crack_drive_k: (2.0, 3.0),
    mid_crack_mix_k: (0.35, 0.45),
    comp: Comp { threshold: 0.20, ratio: 6.5, dry: 0.45, wet: 1.05, attack_s: 0.004, release_s: 0.050 },
    drive_out: 0.80,
    gain_trim: 0.825,
    ..SNARE_WSC
};

/// brush: head-dominant, long shell, soft beater — acoustic-adjacent.
const SNARE_BRUSH: SnarePreset = SnarePreset {
    attack: 0.35,
    wires: 0.4,
    shell_decay_k: (0.18, 0.16),
    wire_decay_k: (0.18, 0.18),
    wire_mix_k: (0.20, 0.25),
    attack_amount_k: (0.08, 0.15),
    shell_pitch_range_k: (0.10, 0.20),
    crack_k: (0.08, 0.20),
    saturation: 0.10,
    crack_freq: 2600.0,
    ks_mix: 0.80,
    velocity_tilt: 0.7,
    wire_gains: [1.5, 1.1, 0.7, 0.45],
    wire_bed_gain: 0.7,
    mode_amps: [0.42, 0.24, 0.24, 0.12],
    beater_bp: (2600.0, 3.5),
    beater_gain: 2.5,
    crack_gain: 1.0,
    mid_crack_drive_k: (1.2, 2.5),
    mid_crack_mix_k: (0.15, 0.30),
    comp: Comp { threshold: 0.34, ratio: 3.0, dry: 0.70, wet: 0.60, attack_s: 0.006, release_s: 0.090 },
    gain_trim: 1.747,
    ..SNARE_WSC
};

/// dust: lofi — dark wires, low crack, soft comp. Sampled-off-vinyl snare.
const SNARE_DUST: SnarePreset = SnarePreset {
    tone: 0.3,
    shell_decay_k: (0.10, 0.10),
    wire_decay_k: (0.10, 0.12),
    wire_mix_k: (0.30, 0.30),
    attack_amount_k: (0.10, 0.20),
    crack_k: (0.10, 0.25),
    bright_shift_k: (0.80, 0.25),
    saturation: 0.40,
    crack_freq: 2200.0,
    ks_mix: 0.50,
    velocity_tilt: 0.3,
    wire_freqs: [4600.0, 7800.0, 10600.0, 13000.0],
    wire_gains: [1.6, 1.0, 0.5, 0.25],
    wire_bed_gain: 0.8,
    beater_bp: (2400.0, 4.0),
    beater_gain: 2.8,
    crack_gain: 1.1,
    mid_crack_drive_k: (1.8, 3.0),
    mid_crack_mix_k: (0.25, 0.40),
    comp: Comp { threshold: 0.30, ratio: 4.0, dry: 0.60, wet: 0.80, attack_s: 0.005, release_s: 0.080 },
    drive_out: 0.85,
    gain_trim: 0.995,
    ..SNARE_WSC
};

/// glass: a high singing bell-snare — sits up at ~250 Hz (its own register,
/// barely tracking the note), loud long-ringing tonal modes for the bell,
/// and bright shimmering wires kept PRESENT (not recessed) so it sparkles.
/// Crack near zero. A pitched, ringing instrument.
const SNARE_GLASS: SnarePreset = SnarePreset {
    tone: 0.8,
    wires: 0.5,
    length: 0.65,
    base_pitch: 250.0,
    pitch_track: 0.25,
    shell_decay_k: (0.20, 0.18),
    wire_decay_k: (0.14, 0.14),
    wire_mix_k: (0.36, 0.28),
    attack_amount_k: (0.10, 0.20),
    crack_k: (0.03, 0.08),
    bright_shift_k: (1.35, 0.40),
    shell_tune: 1.50,
    saturation: 0.05,
    crack_freq: 4200.0,
    ks_mix: 0.35,
    wire_freqs: [7200.0, 11500.0, 15000.0, 18000.0],
    wire_gains: [1.5, 1.4, 1.2, 0.9],
    wire_bed_gain: 0.85,
    mode_ratios: (1.59, 2.14),
    mode_amps: [0.50, 0.34, 0.30, 0.18],
    beater_bp: (5000.0, 6.0),
    beater_gain: 2.8,
    crack_gain: 0.8,
    mid_crack_drive_k: (1.0, 1.5),
    mid_crack_mix_k: (0.08, 0.18),
    comp: Comp { threshold: 0.34, ratio: 2.5, dry: 0.68, wet: 0.65, attack_s: 0.005, release_s: 0.090 },
    drive_out: 0.45,
    gain_trim: 1.835,
    ..SNARE_WSC
};

/// doom: a deep cavernous tom-snare — bottom head tuned WAY down
/// (shell_tune 0.55), bottom modes dominant, dark low wires, very long
/// ring. Massive and clean — opposite of glass on the dark/bright axis.
const SNARE_DOOM: SnarePreset = SnarePreset {
    attack: 0.4,
    wires: 0.45,
    tone: 0.2,
    length: 0.85,
    base_pitch: 100.0,
    pitch_track: 0.35,
    shell_decay_k: (0.30, 0.25),
    wire_decay_k: (0.16, 0.16),
    wire_mix_k: (0.30, 0.28),
    attack_amount_k: (0.10, 0.20),
    crack_k: (0.08, 0.18),
    bright_shift_k: (0.65, 0.20),
    shell_tune: 0.55,
    saturation: 0.25,
    crack_freq: 2000.0,
    ks_mix: 0.55,
    wire_freqs: [3800.0, 6200.0, 8800.0, 11000.0],
    wire_gains: [1.8, 1.2, 0.7, 0.4],
    wire_bed_gain: 1.0,
    mode_ratios: (1.40, 1.35),
    mode_amps: [0.30, 0.10, 0.48, 0.30],
    beater_bp: (2200.0, 3.5),
    beater_gain: 2.8,
    crack_taps: (8, 12),
    crack_gain: 1.4,
    mid_crack_drive_k: (1.2, 2.0),
    mid_crack_mix_k: (0.12, 0.25),
    comp: Comp { threshold: 0.28, ratio: 4.0, dry: 0.58, wet: 0.90, attack_s: 0.006, release_s: 0.110 },
    drive_out: 0.7,
    gain_trim: 1.398,
    ..SNARE_WSC
};

/// crush: a distorted industrial snare — saturation slammed, mid-crack
/// driven hard into the tanh square (3.5-5×), output clipped. Buzzy, brutal.
const SNARE_CRUSH: SnarePreset = SnarePreset {
    attack: 0.7,
    wires: 0.6,
    tone: 0.55,
    base_pitch: 165.0,
    pitch_track: 0.4,
    shell_decay_k: (0.11, 0.11),
    wire_decay_k: (0.13, 0.15),
    wire_mix_k: (0.45, 0.35),
    attack_amount_k: (0.25, 0.38),
    crack_k: (0.42, 0.45),
    bright_shift_k: (0.95, 0.35),
    shell_tune: 1.65,
    saturation: 0.95,
    crack_freq: 3000.0,
    ks_mix: 0.45,
    wire_freqs: [5400.0, 9000.0, 12500.0, 15500.0],
    wire_gains: [1.8, 1.5, 1.0, 0.7],
    wire_bed_gain: 1.5,
    mode_amps: [0.34, 0.16, 0.20, 0.10],
    beater_bp: (3600.0, 5.0),
    beater_gain: 4.5,
    crack_taps: (5, 8),
    crack_gain: 2.4,
    // Drive the 2-5 kHz tap WAY past the linear zone so the whole snare
    // squares off into buzz — this is the dominant timbre, not an accent.
    mid_crack_drive_k: (5.0, 7.0),
    mid_crack_mix_k: (0.60, 0.65),
    comp: Comp { threshold: 0.15, ratio: 9.0, dry: 0.35, wet: 1.20, attack_s: 0.004, release_s: 0.060 },
    drive_out: 1.6,
    gain_trim: 0.601,
    ..SNARE_WSC
};

/// air: a brushed breath-snare — NO crack, NO beater (both ~0). A broad
/// low-gain noise bed carries everything (wire_bed 1.8, resonant bands
/// recessed), long sizzle wash, tiny tonal body. All texture, no snap.
const SNARE_AIR: SnarePreset = SnarePreset {
    attack: 0.15,
    wires: 0.5,
    tone: 0.45,
    length: 0.65,
    base_pitch: 200.0,
    pitch_track: 0.2,
    shell_decay_k: (0.14, 0.14),
    wire_decay_k: (0.22, 0.22),
    wire_mix_k: (0.55, 0.30),
    attack_amount_k: (0.02, 0.06),
    crack_k: (0.01, 0.05),
    bright_shift_k: (0.85, 0.25),
    shell_tune: 1.74,
    saturation: 0.08,
    crack_freq: 2400.0,
    ks_mix: 0.35,
    wire_freqs: [5000.0, 8000.0, 11000.0, 14000.0],
    wire_gains: [0.9, 0.7, 0.5, 0.35],
    wire_bed_gain: 1.8,
    mode_amps: [0.18, 0.08, 0.12, 0.06],
    beater_bp: (2400.0, 3.0),
    beater_gain: 1.5,
    crack_taps: (8, 14),
    crack_gain: 0.6,
    mid_crack_drive_k: (1.0, 1.5),
    mid_crack_mix_k: (0.05, 0.12),
    comp: Comp { threshold: 0.36, ratio: 2.5, dry: 0.72, wet: 0.55, attack_s: 0.008, release_s: 0.120 },
    drive_out: 0.5,
    gain_trim: 1.09,
    ..SNARE_WSC
};

/// snap: a gated click-snare — crack and beater forward, shell and wire
/// decays slammed to near-zero (0.03-0.05s), tiny tonal body. Pure dry
/// "pak" with no ring and no sizzle tail. Opposite of air.
const SNARE_SNAP: SnarePreset = SnarePreset {
    attack: 0.85,
    wires: 0.4,
    tone: 0.6,
    length: 0.1,
    base_pitch: 220.0,
    pitch_track: 0.3,
    shell_decay_k: (0.03, 0.03),
    wire_decay_k: (0.04, 0.05),
    wire_mix_k: (0.40, 0.30),
    attack_amount_k: (0.35, 0.40),
    crack_k: (0.45, 0.45),
    bright_shift_k: (1.15, 0.35),
    shell_tune: 1.68,
    saturation: 0.20,
    crack_freq: 3600.0,
    ks_mix: 0.25,
    wire_freqs: [6000.0, 10000.0, 13500.0, 16500.0],
    wire_gains: [1.6, 1.3, 0.9, 0.6],
    wire_bed_gain: 1.2,
    mode_amps: [0.28, 0.12, 0.14, 0.06],
    beater_bp: (3800.0, 5.5),
    beater_gain: 5.5,
    crack_taps: (4, 7),
    crack_gain: 2.6,
    mid_crack_drive_k: (2.0, 3.0),
    mid_crack_mix_k: (0.35, 0.45),
    comp: Comp { threshold: 0.22, ratio: 6.0, dry: 0.50, wet: 0.95, attack_s: 0.003, release_s: 0.035 },
    drive_out: 0.7,
    gain_trim: 1.015,
    ..SNARE_WSC
};

pub fn snare_preset(name: Option<&str>) -> &'static SnarePreset {
    match name {
        Some("808") => &SNARE_808,
        Some("909") => &SNARE_909,
        Some("trap") => &SNARE_TRAP,
        Some("brush") => &SNARE_BRUSH,
        Some("dust") => &SNARE_DUST,
        Some("glass") => &SNARE_GLASS,
        Some("doom") => &SNARE_DOOM,
        Some("crush") => &SNARE_CRUSH,
        Some("air") => &SNARE_AIR,
        Some("snap") => &SNARE_SNAP,
        _ => &SNARE_WSC,
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ResolvedSnare {
    pub tune: f64,
    pub shell_decay: f64,
    pub wire_decay: f64,
    pub wire_mix: f64,
    pub shell_tune: f64,
    pub attack_amount: f64,
    pub shell_pitch_decay: f64,
    pub shell_pitch_range: f64,
    pub head_damping_ratio: f64,
    pub saturation: f64,
    pub crack: f64,
    pub crack_freq: f64,
    pub ks_mix: f64,
    pub velocity_tilt: f64,
    /// Wire-band frequency multiplier from the `tone` macro.
    pub bright_shift: f64,
    /// The resolved fundamental in Hz. `resolve_snare` seeds this assuming
    /// the note sits at the neutral 185 Hz; the synth overwrites it once the
    /// actual played frequency is known (see `base_pitch` / `pitch_track`).
    pub base_freq: f64,
    pub internal: &'static SnarePreset,
}

pub fn resolve_snare(params: Option<&SnareParams>) -> ResolvedSnare {
    let pre = snare_preset(params.and_then(|p| p.preset.as_deref()));
    let attack = p(params.and_then(|x| x.attack), pre.attack);
    let wires = p(params.and_then(|x| x.wires), pre.wires);
    let tone = p(params.and_then(|x| x.tone), pre.tone);
    let length = p(params.and_then(|x| x.length), pre.length);
    ResolvedSnare {
        tune: p(params.and_then(|x| x.tune), 1.0),
        shell_decay: p(params.and_then(|x| x.shell_decay), pre.shell_decay_k.0 + length * pre.shell_decay_k.1),
        wire_decay: p(params.and_then(|x| x.wire_decay), pre.wire_decay_k.0 + wires * pre.wire_decay_k.1),
        wire_mix: p(params.and_then(|x| x.wire_mix), pre.wire_mix_k.0 + wires * pre.wire_mix_k.1),
        shell_tune: p(params.and_then(|x| x.shell_tune), pre.shell_tune),
        attack_amount: p(params.and_then(|x| x.attack_amount), pre.attack_amount_k.0 + attack * pre.attack_amount_k.1),
        shell_pitch_decay: p(params.and_then(|x| x.shell_pitch_decay), pre.shell_pitch_decay),
        shell_pitch_range: p(params.and_then(|x| x.shell_pitch_range), pre.shell_pitch_range_k.0 + attack * pre.shell_pitch_range_k.1),
        head_damping_ratio: p(params.and_then(|x| x.head_damping_ratio), pre.head_damping_k.0 + tone * pre.head_damping_k.1),
        saturation: p(params.and_then(|x| x.saturation), pre.saturation),
        crack: p(params.and_then(|x| x.crack), pre.crack_k.0 + attack * pre.crack_k.1),
        crack_freq: p(params.and_then(|x| x.crack_freq), pre.crack_freq),
        ks_mix: p(params.and_then(|x| x.ks_mix), pre.ks_mix),
        velocity_tilt: p(params.and_then(|x| x.velocity_tilt), pre.velocity_tilt),
        bright_shift: pre.bright_shift_k.0 + tone * pre.bright_shift_k.1,
        // Seeded at the neutral note; the synth recomputes with the real one.
        base_freq: pre.base_pitch * p(params.and_then(|x| x.tune), 1.0),
        internal: pre,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// HIHAT (closed + open share a preset; open only changes decay coefficients)
// ═══════════════════════════════════════════════════════════════════════

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HiHatPreset {
    // Macro centers.
    pub attack: f64,
    pub metal: f64,
    pub length: f64,
    // Formula coefficients.
    /// decay_rate = base + (1 − length) × range; (closed, open) pairs.
    pub decay_closed_k: (f64, f64),
    pub decay_open_k: (f64, f64),
    pub shimmer_k: (f64, f64),       // on metal
    pub brightness_k: (f64, f64),    // on metal
    pub attack_amount_k: (f64, f64), // on attack
    pub pitch_drop_k: (f64, f64),    // on attack
    // Direct param defaults.
    pub ping_amount: f64,
    pub velocity_tilt: f64,
    // ── Internal-only knobs ──
    /// Cymbal "size": multiplies all 22 mode ratios (>1 = smaller/brighter).
    pub mode_freq_scale: f64,
    /// Spectral tilt across the mode bank: amp ×= tilt^(i/21). 1.0 neutral,
    /// >1 favors high modes (909 sizzle), <1 darkens (808 square-bank).
    pub mode_amp_tilt: f64,
    /// Nonlinear modal coupling drive (today: 0.06).
    pub coupling: f64,
    /// Air noise bands: (center Hz, Q, gain, decay multiplier) × 2.
    pub air_lo: (f64, f64, f64, f64),
    pub air_hi: (f64, f64, f64, f64),
    /// Shimmer/air blend (today: 0.45 / 0.55).
    pub mix: (f64, f64),
    /// Highpass keeping modes out of the kick band (Hz).
    pub hp_cutoff: f64,
    /// Ping: freq multiplier on f_base, clamp range, decay tau (s).
    pub ping_freq_mult: f64,
    pub ping_clamp: (f64, f64),
    pub ping_tau: f64,
    pub gain_trim: f64,
}

const HIHAT_WSC: HiHatPreset = HiHatPreset {
    attack: 0.5,
    metal: 0.5,
    length: 0.5,
    decay_closed_k: (30.0, 18.0),
    decay_open_k: (6.0, 4.0),
    shimmer_k: (0.9, 0.4),
    brightness_k: (0.5, 0.6),
    attack_amount_k: (0.15, 0.35),
    pitch_drop_k: (0.008, 0.016),
    ping_amount: 0.18,
    velocity_tilt: 0.4,
    mode_freq_scale: 1.0,
    mode_amp_tilt: 1.0,
    coupling: 0.06,
    air_lo: (6000.0, 4.0, 1.6, 0.85),
    air_hi: (11000.0, 3.0, 1.2, 1.55),
    mix: (0.45, 0.55),
    hp_cutoff: 1500.0,
    ping_freq_mult: 4.5,
    ping_clamp: (1800.0, 3500.0),
    ping_tau: 0.0030,
    gain_trim: 1.0,
};

/// 808: dark metallic square-bank character — low modes favored, less air.
const HIHAT_808: HiHatPreset = HiHatPreset {
    decay_closed_k: (34.0, 18.0),
    decay_open_k: (7.0, 4.0),
    shimmer_k: (0.85, 0.30),
    brightness_k: (0.40, 0.45),
    attack_amount_k: (0.10, 0.25),
    ping_amount: 0.10,
    mode_freq_scale: 1.06,
    mode_amp_tilt: 0.55,
    coupling: 0.04,
    air_lo: (5200.0, 4.5, 1.3, 0.90),
    air_hi: (9500.0, 3.5, 0.8, 1.60),
    mix: (0.55, 0.45),
    ping_freq_mult: 4.0,
    gain_trim: 1.801,
    ..HIHAT_WSC
};

/// 909: bright noisy sizzle — air-forward, high modes up, crisp attack.
const HIHAT_909: HiHatPreset = HiHatPreset {
    attack: 0.65,
    decay_closed_k: (28.0, 16.0),
    decay_open_k: (5.5, 3.5),
    shimmer_k: (0.95, 0.40),
    brightness_k: (0.65, 0.60),
    attack_amount_k: (0.25, 0.40),
    ping_amount: 0.12,
    mode_amp_tilt: 1.6,
    coupling: 0.05,
    air_lo: (6800.0, 3.5, 1.8, 0.85),
    air_hi: (12000.0, 2.8, 1.7, 1.45),
    mix: (0.35, 0.65),
    gain_trim: 0.63,
    ..HIHAT_WSC
};

/// trap: the tight tick for hat rolls — very fast decay, strong attack.
const HIHAT_TRAP: HiHatPreset = HiHatPreset {
    attack: 0.7,
    decay_closed_k: (44.0, 22.0),
    decay_open_k: (8.0, 5.0),
    shimmer_k: (0.95, 0.40),
    brightness_k: (0.60, 0.55),
    attack_amount_k: (0.30, 0.40),
    pitch_drop_k: (0.012, 0.020),
    ping_amount: 0.25,
    velocity_tilt: 0.6,
    mode_freq_scale: 1.12,
    mode_amp_tilt: 1.3,
    air_lo: (6500.0, 4.0, 1.5, 0.90),
    air_hi: (11500.0, 3.0, 1.4, 1.50),
    mix: (0.40, 0.60),
    ping_freq_mult: 5.0,
    ping_clamp: (2200.0, 4000.0),
    ping_tau: 0.0024,
    gain_trim: 0.989,
    ..HIHAT_WSC
};

/// acoustic: a looser, live pair — longer ring, more modal beating.
const HIHAT_ACOUSTIC: HiHatPreset = HiHatPreset {
    attack: 0.4,
    decay_closed_k: (22.0, 14.0),
    decay_open_k: (4.5, 3.0),
    shimmer_k: (0.90, 0.40),
    brightness_k: (0.55, 0.55),
    attack_amount_k: (0.10, 0.25),
    ping_amount: 0.08,
    velocity_tilt: 0.6,
    coupling: 0.09,
    air_lo: (5800.0, 3.5, 1.5, 0.80),
    air_hi: (10500.0, 2.8, 1.1, 1.40),
    mix: (0.50, 0.50),
    gain_trim: 0.946,
    ..HIHAT_WSC
};

/// dust: lofi — air pulled down, dark modes, thin and papery.
const HIHAT_DUST: HiHatPreset = HiHatPreset {
    metal: 0.35,
    decay_closed_k: (34.0, 18.0),
    decay_open_k: (7.0, 4.0),
    shimmer_k: (0.85, 0.30),
    brightness_k: (0.35, 0.40),
    attack_amount_k: (0.10, 0.20),
    ping_amount: 0.06,
    velocity_tilt: 0.3,
    mode_amp_tilt: 0.5,
    coupling: 0.04,
    air_lo: (5000.0, 4.5, 1.0, 0.95),
    air_hi: (8800.0, 3.5, 0.5, 1.70),
    mix: (0.50, 0.50),
    hp_cutoff: 1800.0,
    gain_trim: 2.229,
    ..HIHAT_WSC
};

/// glass: a tiny bright cymbal — high mode scale, extreme upper tilt, lots
/// of sparkling air, long ring. The brightest, most crystalline hat.
const HIHAT_GLASS: HiHatPreset = HiHatPreset {
    metal: 0.75,
    length: 0.6,
    decay_closed_k: (22.0, 16.0),
    decay_open_k: (4.0, 3.0),
    shimmer_k: (0.95, 0.45),
    brightness_k: (0.75, 0.62),
    ping_amount: 0.12,
    mode_freq_scale: 1.20,
    mode_amp_tilt: 2.0,
    coupling: 0.04,
    air_lo: (7500.0, 3.5, 1.7, 0.85),
    air_hi: (14000.0, 2.5, 1.7, 1.55),
    mix: (0.40, 0.60),
    hp_cutoff: 1800.0,
    ping_freq_mult: 5.5,
    gain_trim: 0.621,
    ..HIHAT_WSC
};

/// doom: a huge dark gong-plate — large size (mode scale 0.85), dark tilt,
/// heavy clangy beating (coupling 0.12), very long. Cavernous, not crisp.
const HIHAT_DOOM: HiHatPreset = HiHatPreset {
    metal: 0.5,
    length: 0.75,
    decay_closed_k: (20.0, 14.0),
    decay_open_k: (3.5, 2.5),
    shimmer_k: (0.85, 0.35),
    brightness_k: (0.35, 0.40),
    ping_amount: 0.06,
    mode_freq_scale: 0.85,
    mode_amp_tilt: 0.55,
    coupling: 0.12,
    air_lo: (4500.0, 3.5, 1.6, 0.92),
    air_hi: (8000.0, 2.8, 1.1, 1.55),
    mix: (0.58, 0.42),
    hp_cutoff: 1000.0,
    gain_trim: 1.017,
    ..HIHAT_WSC
};

/// crush: a harsh clangy cymbal — maxed modal coupling (0.16) so the modes
/// intermodulate into a metallic clang, bright and noisy. The trash-can hat.
const HIHAT_CRUSH: HiHatPreset = HiHatPreset {
    attack: 0.7,
    metal: 0.7,
    decay_closed_k: (30.0, 18.0),
    decay_open_k: (5.5, 3.5),
    shimmer_k: (0.98, 0.45),
    brightness_k: (0.70, 0.60),
    attack_amount_k: (0.30, 0.40),
    ping_amount: 0.20,
    mode_freq_scale: 1.05,
    mode_amp_tilt: 1.3,
    coupling: 0.16,
    air_lo: (6000.0, 4.5, 1.9, 0.88),
    air_hi: (11000.0, 3.5, 1.7, 1.50),
    mix: (0.45, 0.55),
    ping_freq_mult: 5.0,
    gain_trim: 0.84,
    ..HIHAT_WSC
};

/// air: pure breath — modes pulled way down (tilt 0.6) so the air bands
/// dominate (mix .70 toward air, air gains boosted), soft attack, long wash.
/// A whispered cymbal, almost no metallic ping.
const HIHAT_AIR: HiHatPreset = HiHatPreset {
    attack: 0.15,
    metal: 0.4,
    length: 0.6,
    decay_closed_k: (24.0, 14.0),
    decay_open_k: (3.5, 2.5),
    shimmer_k: (0.90, 0.35),
    brightness_k: (0.45, 0.45),
    attack_amount_k: (0.05, 0.15),
    ping_amount: 0.03,
    velocity_tilt: 0.4,
    mode_freq_scale: 0.95,
    mode_amp_tilt: 0.6,
    coupling: 0.08,
    air_lo: (5500.0, 3.0, 2.0, 0.80),
    air_hi: (10000.0, 2.4, 1.6, 1.40),
    mix: (0.30, 0.70),
    hp_cutoff: 1400.0,
    gain_trim: 0.493,
    ..HIHAT_WSC
};

/// snap: a tiny dry tick — fastest decay of any hat, strong attack, bright.
/// All click, no wash — the inverse of `air`.
const HIHAT_SNAP: HiHatPreset = HiHatPreset {
    attack: 0.8,
    metal: 0.55,
    decay_closed_k: (52.0, 24.0),
    decay_open_k: (9.0, 5.0),
    shimmer_k: (0.95, 0.40),
    brightness_k: (0.62, 0.55),
    attack_amount_k: (0.30, 0.40),
    pitch_drop_k: (0.012, 0.020),
    ping_amount: 0.22,
    velocity_tilt: 0.6,
    mode_freq_scale: 1.15,
    mode_amp_tilt: 1.4,
    coupling: 0.04,
    air_lo: (6800.0, 4.2, 1.4, 0.92),
    air_hi: (12000.0, 3.2, 1.3, 1.48),
    mix: (0.42, 0.58),
    ping_freq_mult: 5.2,
    ping_clamp: (2200.0, 4200.0),
    ping_tau: 0.0022,
    gain_trim: 1.188,
    ..HIHAT_WSC
};

pub fn hihat_preset(name: Option<&str>) -> &'static HiHatPreset {
    match name {
        Some("808") => &HIHAT_808,
        Some("909") => &HIHAT_909,
        Some("trap") => &HIHAT_TRAP,
        Some("acoustic") => &HIHAT_ACOUSTIC,
        Some("dust") => &HIHAT_DUST,
        Some("glass") => &HIHAT_GLASS,
        Some("doom") => &HIHAT_DOOM,
        Some("crush") => &HIHAT_CRUSH,
        Some("air") => &HIHAT_AIR,
        Some("snap") => &HIHAT_SNAP,
        _ => &HIHAT_WSC,
    }
}

/// 22-mode cymbal table — first 22 Bessel-function zeros for a circular
/// plate, with intentional close pairs for *beating*. `(ratio, base_amp,
/// decay_mult)`. Per-preset character comes from `mode_freq_scale` (size)
/// and `mode_amp_tilt` (spectral tilt) applied at resolution; brightness
/// continues to scale modes 8+ per-sample (velocity-driven).
pub const HIHAT_MODES: [(f64, f64, f64); 22] = [
    (1.000, 0.14, 1.0),
    (1.594, 0.18, 1.3),
    (1.612, 0.13, 1.35), // beats with 1.594
    (2.135, 0.16, 1.7),
    (2.295, 0.18, 1.9),
    (2.310, 0.12, 1.95), // beats with 2.295
    (2.653, 0.15, 2.3),
    (2.917, 0.14, 2.7),
    (3.156, 0.13, 3.1),
    (3.500, 0.11, 3.6),
    (3.598, 0.09, 3.7), // beats with 3.500
    (3.652, 0.08, 3.8), // beats with 3.598
    (4.060, 0.09, 4.4),
    (4.131, 0.07, 4.5), // beats with 4.060
    (4.601, 0.08, 5.2),
    (4.832, 0.07, 5.5),
    (5.158, 0.06, 6.0),
    (5.412, 0.05, 6.5),
    (5.872, 0.05, 7.0),
    (6.205, 0.04, 7.7),
    (6.560, 0.04, 8.4),
    (6.957, 0.03, 9.2),
];

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ResolvedHiHat {
    pub tune: f64,
    pub decay_rate: f64,
    pub shimmer: f64,
    pub brightness: f64,
    pub attack_amount: f64,
    pub ping_amount: f64,
    pub pitch_drop: f64,
    pub velocity_tilt: f64,
    /// Per-mode amplitudes with the preset's spectral tilt baked in
    /// (`base_amp × mode_amp_tilt^(i/21)`) — precomputed because the mode
    /// loop runs per sample and `powf` is not free.
    pub mode_amp: [f64; 22],
    pub internal: &'static HiHatPreset,
}

pub fn resolve_hihat(params: Option<&HiHatParams>, open: bool) -> ResolvedHiHat {
    let pre = hihat_preset(params.and_then(|p| p.preset.as_deref()));
    let attack = p(params.and_then(|x| x.attack), pre.attack);
    let metal = p(params.and_then(|x| x.metal), pre.metal);
    let length = p(params.and_then(|x| x.length), pre.length);
    let decay_k = if open { pre.decay_open_k } else { pre.decay_closed_k };
    let mut mode_amp = [0.0; 22];
    for (i, (_, base_amp, _)) in HIHAT_MODES.iter().enumerate() {
        mode_amp[i] = base_amp * pre.mode_amp_tilt.powf(i as f64 / 21.0);
    }
    ResolvedHiHat {
        tune: p(params.and_then(|x| x.tune), 1.0),
        decay_rate: p(params.and_then(|x| x.decay_rate), decay_k.0 + (1.0 - length) * decay_k.1),
        shimmer: p(params.and_then(|x| x.shimmer), pre.shimmer_k.0 + metal * pre.shimmer_k.1),
        brightness: p(params.and_then(|x| x.brightness), pre.brightness_k.0 + metal * pre.brightness_k.1),
        attack_amount: p(params.and_then(|x| x.attack_amount), pre.attack_amount_k.0 + attack * pre.attack_amount_k.1),
        ping_amount: p(params.and_then(|x| x.ping_amount), pre.ping_amount),
        pitch_drop: p(params.and_then(|x| x.pitch_drop), pre.pitch_drop_k.0 + attack * pre.pitch_drop_k.1),
        velocity_tilt: p(params.and_then(|x| x.velocity_tilt), pre.velocity_tilt),
        mode_amp,
        internal: pre,
    }
}


// ═══════════════════════════════════════════════════════════════════════
// CLAP — noise burst train + resonant tail (the analog clap architecture)
// ═══════════════════════════════════════════════════════════════════════

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ClapPreset {
    // Macro centers.
    pub attack: f64,
    pub spread: f64,
    pub tone: f64,
    pub length: f64,
    // Formula coefficients.
    /// Burst envelope time constant (s) = k.0 − attack × k.1 (sharper when hot).
    pub burst_tau_k: (f64, f64),
    /// Spacing between bursts (s) = k.0 + spread × k.1.
    pub spacing_k: (f64, f64),
    /// Main bandpass center (Hz) = k.0 + tone × k.1.
    pub bp1_freq_k: (f64, f64),
    /// Upper band gain = k.0 + tone × k.1.
    pub bp2_gain_k: (f64, f64),
    /// Tail time constant (s) = k.0 + length × k.1.
    pub tail_k: (f64, f64),
    // Direct param defaults.
    pub saturation: f64,
    pub velocity_tilt: f64,
    // ── Internal-only knobs ──
    /// Number of bursts before the tail (3-4 typical).
    pub n_bursts: usize,
    /// Per-note random spacing jitter, fraction of spacing.
    pub spacing_jitter: f64,
    /// Each successive burst slightly louder — the last hit carries.
    pub burst_growth: f64,
    pub bp1_q: f64,
    /// Upper band: frequency ratio on bp1 and Q.
    pub bp2_ratio: f64,
    pub bp2_q: f64,
    /// Tail level relative to burst peak.
    pub tail_level: f64,
    pub drive_out: f64,
    pub gain_trim: f64,
}

const CLAP_WSC: ClapPreset = ClapPreset {
    attack: 0.5,
    spread: 0.5,
    tone: 0.5,
    length: 0.5,
    burst_tau_k: (0.0050, 0.0025),
    spacing_k: (0.007, 0.007),
    bp1_freq_k: (950.0, 700.0),
    bp2_gain_k: (0.25, 0.55),
    tail_k: (0.055, 0.150),
    saturation: 0.35,
    velocity_tilt: 0.5,
    n_bursts: 4,
    spacing_jitter: 0.18,
    burst_growth: 1.12,
    bp1_q: 1.3,
    bp2_ratio: 2.3,
    bp2_q: 2.0,
    tail_level: 0.55,
    drive_out: 0.8,
    gain_trim: 1.0,
};

/// 808: the iconic spread clap — slower bursts, resonant ~1 kHz band,
/// generous reverb-ish tail.
const CLAP_808: ClapPreset = ClapPreset {
    spread: 0.6,
    length: 0.6,
    burst_tau_k: (0.0060, 0.0020),
    spacing_k: (0.009, 0.006),
    bp1_freq_k: (900.0, 500.0),
    bp2_gain_k: (0.15, 0.40),
    tail_k: (0.080, 0.160),
    spacing_jitter: 0.10,
    bp1_q: 1.8,
    tail_level: 0.65,
    gain_trim: 0.919,
    ..CLAP_WSC
};

/// 909: tighter and noisier — three fast bursts, brighter, shorter tail.
const CLAP_909: ClapPreset = ClapPreset {
    attack: 0.65,
    burst_tau_k: (0.0040, 0.0020),
    spacing_k: (0.006, 0.005),
    bp1_freq_k: (1100.0, 700.0),
    bp2_gain_k: (0.35, 0.55),
    tail_k: (0.045, 0.110),
    n_bursts: 3,
    spacing_jitter: 0.22,
    bp1_q: 1.1,
    tail_level: 0.50,
    drive_out: 0.9,
    gain_trim: 1.009,
    ..CLAP_WSC
};

/// trap: bright layered snap — wide top band, sharp bursts, medium tail.
const CLAP_TRAP: ClapPreset = ClapPreset {
    attack: 0.7,
    tone: 0.7,
    burst_tau_k: (0.0040, 0.0020),
    spacing_k: (0.006, 0.006),
    bp1_freq_k: (1200.0, 800.0),
    bp2_gain_k: (0.45, 0.60),
    tail_k: (0.060, 0.140),
    bp1_q: 1.2,
    bp2_ratio: 2.6,
    tail_level: 0.60,
    drive_out: 0.95,
    gain_trim: 0.606,
    ..CLAP_WSC
};

/// dust: lofi — dark, papery, short.
const CLAP_DUST: ClapPreset = ClapPreset {
    tone: 0.3,
    burst_tau_k: (0.0060, 0.0020),
    spacing_k: (0.008, 0.006),
    bp1_freq_k: (700.0, 400.0),
    bp2_gain_k: (0.10, 0.25),
    tail_k: (0.045, 0.100),
    spacing_jitter: 0.25,
    bp1_q: 1.6,
    tail_level: 0.45,
    saturation: 0.5,
    velocity_tilt: 0.3,
    gain_trim: 1.742,
    ..CLAP_WSC
};

/// glass: a bright sparkly clap — high resonant band, crisp bursts,
/// shimmering top, short-medium tail.
const CLAP_GLASS: ClapPreset = ClapPreset {
    tone: 0.75,
    burst_tau_k: (0.0045, 0.0020),
    spacing_k: (0.007, 0.006),
    bp1_freq_k: (1300.0, 800.0),
    bp2_gain_k: (0.45, 0.55),
    tail_k: (0.050, 0.110),
    bp1_q: 1.4,
    bp2_ratio: 2.6,
    tail_level: 0.45,
    saturation: 0.15,
    gain_trim: 1.045,
    ..CLAP_WSC
};

/// doom: a big dark slow clap — low resonant band, wide bursts, long
/// reverby tail. The cavernous hand-clap in a stairwell.
const CLAP_DOOM: ClapPreset = ClapPreset {
    attack: 0.4,
    spread: 0.6,
    tone: 0.2,
    length: 0.8,
    burst_tau_k: (0.0060, 0.0020),
    spacing_k: (0.010, 0.006),
    bp1_freq_k: (650.0, 350.0),
    bp2_gain_k: (0.10, 0.30),
    tail_k: (0.100, 0.180),
    bp1_q: 1.8,
    tail_level: 0.70,
    saturation: 0.40,
    drive_out: 0.85,
    gain_trim: 0.954,
    ..CLAP_WSC
};

/// crush: a distorted aggressive clap — saturation slammed, output driven
/// into the clipper. Buzzy and harsh.
const CLAP_CRUSH: ClapPreset = ClapPreset {
    attack: 0.6,
    tone: 0.6,
    burst_tau_k: (0.0045, 0.0020),
    spacing_k: (0.006, 0.005),
    bp1_freq_k: (1100.0, 700.0),
    bp2_gain_k: (0.45, 0.55),
    tail_k: (0.055, 0.120),
    bp1_q: 1.2,
    tail_level: 0.55,
    saturation: 0.85,
    drive_out: 1.4,
    gain_trim: 0.304,
    ..CLAP_WSC
};

/// air: a soft washy spread clap — wide scattered low-Q bursts, broad band,
/// long breathy tail. The room-clap reverb with barely any transient.
const CLAP_AIR: ClapPreset = ClapPreset {
    attack: 0.3,
    spread: 0.75,
    tone: 0.45,
    length: 0.75,
    burst_tau_k: (0.0070, 0.0015),
    spacing_k: (0.012, 0.007),
    bp1_freq_k: (800.0, 500.0),
    bp2_gain_k: (0.15, 0.30),
    tail_k: (0.110, 0.180),
    spacing_jitter: 0.10,
    bp1_q: 1.0,
    tail_level: 0.75,
    saturation: 0.15,
    velocity_tilt: 0.4,
    gain_trim: 0.8,
    ..CLAP_WSC
};

/// snap: a tight dry clap — only two fast bursts, bright band, very short
/// tail. A single sharp "pak", no spread.
const CLAP_SNAP: ClapPreset = ClapPreset {
    attack: 0.8,
    tone: 0.6,
    burst_tau_k: (0.0035, 0.0020),
    spacing_k: (0.005, 0.004),
    bp1_freq_k: (1250.0, 750.0),
    bp2_gain_k: (0.45, 0.55),
    tail_k: (0.030, 0.070),
    n_bursts: 2,
    bp1_q: 1.3,
    bp2_ratio: 2.6,
    tail_level: 0.35,
    saturation: 0.20,
    drive_out: 0.9,
    gain_trim: 1.782,
    ..CLAP_WSC
};

pub fn clap_preset(name: Option<&str>) -> &'static ClapPreset {
    match name {
        Some("808") => &CLAP_808,
        Some("909") => &CLAP_909,
        Some("trap") => &CLAP_TRAP,
        Some("dust") => &CLAP_DUST,
        Some("glass") => &CLAP_GLASS,
        Some("doom") => &CLAP_DOOM,
        Some("crush") => &CLAP_CRUSH,
        Some("air") => &CLAP_AIR,
        Some("snap") => &CLAP_SNAP,
        _ => &CLAP_WSC,
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ResolvedClap {
    pub tune: f64,
    pub burst_tau: f64,
    pub spacing: f64,
    pub bp1_freq: f64,
    pub bp2_gain: f64,
    pub tail_tau: f64,
    pub saturation: f64,
    pub velocity_tilt: f64,
    pub internal: &'static ClapPreset,
}

pub fn resolve_clap(params: Option<&ClapParams>) -> ResolvedClap {
    let pre = clap_preset(params.and_then(|p| p.preset.as_deref()));
    let attack = p(params.and_then(|x| x.attack), pre.attack);
    let spread = p(params.and_then(|x| x.spread), pre.spread);
    let tone = p(params.and_then(|x| x.tone), pre.tone);
    let length = p(params.and_then(|x| x.length), pre.length);
    ResolvedClap {
        tune: p(params.and_then(|x| x.tune), 1.0),
        burst_tau: (pre.burst_tau_k.0 - attack * pre.burst_tau_k.1).max(0.0008),
        spacing: pre.spacing_k.0 + spread * pre.spacing_k.1,
        bp1_freq: pre.bp1_freq_k.0 + tone * pre.bp1_freq_k.1,
        bp2_gain: pre.bp2_gain_k.0 + tone * pre.bp2_gain_k.1,
        tail_tau: pre.tail_k.0 + length * pre.tail_k.1,
        saturation: p(params.and_then(|x| x.saturation), pre.saturation),
        velocity_tilt: p(params.and_then(|x| x.velocity_tilt), pre.velocity_tilt),
        internal: pre,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// RIMSHOT — two inharmonic resonator rings + click. Short and dry.
// ═══════════════════════════════════════════════════════════════════════

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RimshotPreset {
    // Macro centers.
    pub tone: f64,
    pub length: f64,
    // Formula coefficients.
    /// Primary ring frequency (Hz) = k.0 + tone × k.1.
    pub ring_freq_k: (f64, f64),
    /// Ring decay time constant (s) = k.0 + length × k.1.
    pub ring_tau_k: (f64, f64),
    /// Upper ring gain = k.0 + tone × k.1.
    pub ring2_gain_k: (f64, f64),
    // Direct param defaults.
    pub attack_amount: f64,
    pub velocity_tilt: f64,
    // ── Internal-only knobs ──
    /// Upper ring frequency ratio (inharmonic — NOT an integer multiple).
    pub ring2_ratio: f64,
    pub ring_q: (f64, f64),
    /// Ring level balance (primary, upper) before ring2_gain.
    pub ring_mix: (f64, f64),
    pub gain_trim: f64,
}

const RIMSHOT_WSC: RimshotPreset = RimshotPreset {
    tone: 0.5,
    length: 0.5,
    ring_freq_k: (500.0, 900.0),
    ring_tau_k: (0.018, 0.045),
    ring2_gain_k: (0.35, 0.55),
    attack_amount: 0.55,
    velocity_tilt: 0.5,
    ring2_ratio: 2.47,
    ring_q: (9.0, 7.0),
    ring_mix: (1.0, 0.7),
    gain_trim: 1.0,
};

/// 808: the tonal ping — higher, purer, slightly longer.
const RIMSHOT_808: RimshotPreset = RimshotPreset {
    tone: 0.7,
    ring_freq_k: (800.0, 1100.0),
    ring_tau_k: (0.022, 0.050),
    ring2_gain_k: (0.20, 0.35),
    attack_amount: 0.40,
    ring2_ratio: 2.10,
    ring_q: (14.0, 9.0),
    gain_trim: 0.967,
    ..RIMSHOT_WSC
};

/// 909: brighter click-forward rim — more stick, less ring.
const RIMSHOT_909: RimshotPreset = RimshotPreset {
    ring_freq_k: (650.0, 1000.0),
    ring_tau_k: (0.012, 0.030),
    ring2_gain_k: (0.45, 0.55),
    attack_amount: 0.62,
    ring2_ratio: 2.85,
    ring_q: (7.0, 5.5),
    gain_trim: 0.841,
    ..RIMSHOT_WSC
};

/// acoustic: the woody side-stick — low "tock", fast, soft click.
const RIMSHOT_ACOUSTIC: RimshotPreset = RimshotPreset {
    tone: 0.3,
    ring_freq_k: (380.0, 500.0),
    ring_tau_k: (0.020, 0.040),
    ring2_gain_k: (0.30, 0.40),
    attack_amount: 0.35,
    velocity_tilt: 0.7,
    ring2_ratio: 3.20,
    ring_q: (8.0, 6.0),
    ring_mix: (1.0, 0.5),
    gain_trim: 1.472,
    ..RIMSHOT_WSC
};

/// glass: a high pure bell-ping — near-octave tonal rings, very high Q,
/// long clean ring. The most musical, pitched rim.
const RIMSHOT_GLASS: RimshotPreset = RimshotPreset {
    tone: 0.85,
    ring_freq_k: (1100.0, 1300.0),
    ring_tau_k: (0.025, 0.055),
    ring2_gain_k: (0.20, 0.30),
    attack_amount: 0.35,
    ring2_ratio: 2.01,
    ring_q: (18.0, 13.0),
    ring_mix: (1.0, 0.55),
    gain_trim: 0.983,
    ..RIMSHOT_WSC
};

/// doom: a deep low tock — very low rings, long, woody. The floor-tom rim.
const RIMSHOT_DOOM: RimshotPreset = RimshotPreset {
    tone: 0.2,
    ring_freq_k: (280.0, 380.0),
    ring_tau_k: (0.022, 0.050),
    ring2_gain_k: (0.35, 0.40),
    attack_amount: 0.45,
    ring2_ratio: 2.95,
    ring_q: (9.0, 7.0),
    ring_mix: (1.0, 0.7),
    gain_trim: 1.857,
    ..RIMSHOT_WSC
};

/// crush: a gnarly clangy rim — strong, very inharmonic upper ring (ratio
/// 3.4) and a hard click. Metallic and dissonant.
const RIMSHOT_CRUSH: RimshotPreset = RimshotPreset {
    tone: 0.55,
    ring_freq_k: (550.0, 850.0),
    ring_tau_k: (0.014, 0.032),
    ring2_gain_k: (0.55, 0.55),
    attack_amount: 0.70,
    ring2_ratio: 3.40,
    ring_q: (6.0, 4.5),
    ring_mix: (1.0, 0.9),
    gain_trim: 0.585,
    ..RIMSHOT_WSC
};

/// air: a soft woody low click — gentle attack, mid-low ring, recessed upper.
/// The brushed side-stick.
const RIMSHOT_AIR: RimshotPreset = RimshotPreset {
    tone: 0.35,
    ring_freq_k: (400.0, 550.0),
    ring_tau_k: (0.018, 0.038),
    ring2_gain_k: (0.20, 0.30),
    attack_amount: 0.25,
    velocity_tilt: 0.6,
    ring2_ratio: 2.80,
    ring_q: (7.0, 5.0),
    ring_mix: (1.0, 0.5),
    gain_trim: 1.4,
    ..RIMSHOT_WSC
};

/// snap: a tight bright click-rim — very short rings, click-forward. A dry
/// "tk" with almost no sustain.
const RIMSHOT_SNAP: RimshotPreset = RimshotPreset {
    tone: 0.7,
    ring_freq_k: (750.0, 1100.0),
    ring_tau_k: (0.008, 0.018),
    ring2_gain_k: (0.45, 0.50),
    attack_amount: 0.72,
    ring2_ratio: 2.75,
    ring_q: (8.0, 6.0),
    ring_mix: (1.0, 0.6),
    gain_trim: 0.73,
    ..RIMSHOT_WSC
};

pub fn rimshot_preset(name: Option<&str>) -> &'static RimshotPreset {
    match name {
        Some("808") => &RIMSHOT_808,
        Some("909") => &RIMSHOT_909,
        Some("acoustic") => &RIMSHOT_ACOUSTIC,
        Some("glass") => &RIMSHOT_GLASS,
        Some("doom") => &RIMSHOT_DOOM,
        Some("crush") => &RIMSHOT_CRUSH,
        Some("air") => &RIMSHOT_AIR,
        Some("snap") => &RIMSHOT_SNAP,
        _ => &RIMSHOT_WSC,
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ResolvedRimshot {
    pub tune: f64,
    pub ring_freq: f64,
    pub ring_tau: f64,
    pub ring2_gain: f64,
    pub attack_amount: f64,
    pub velocity_tilt: f64,
    pub internal: &'static RimshotPreset,
}

pub fn resolve_rimshot(params: Option<&RimshotParams>) -> ResolvedRimshot {
    let pre = rimshot_preset(params.and_then(|p| p.preset.as_deref()));
    let tone = p(params.and_then(|x| x.tone), pre.tone);
    let length = p(params.and_then(|x| x.length), pre.length);
    ResolvedRimshot {
        tune: p(params.and_then(|x| x.tune), 1.0),
        ring_freq: pre.ring_freq_k.0 + tone * pre.ring_freq_k.1,
        ring_tau: pre.ring_tau_k.0 + length * pre.ring_tau_k.1,
        ring2_gain: pre.ring2_gain_k.0 + tone * pre.ring2_gain_k.1,
        attack_amount: p(params.and_then(|x| x.attack_amount), pre.attack_amount),
        velocity_tilt: p(params.and_then(|x| x.velocity_tilt), pre.velocity_tilt),
        internal: pre,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use weresocool_ast::drum_presets::{CLAP_PRESETS, HIHAT_PRESETS, KICK_PRESETS, RIMSHOT_PRESETS, SNARE_PRESETS};

    /// The parser validates preset names against the lists in the ast
    /// crate; every listed name must resolve to a DISTINCT table here
    /// (except `wsc`, which is the fallback identity).
    #[test]
    fn tables_match_ast_names() {
        for name in KICK_PRESETS {
            let preset = kick_preset(Some(name));
            if *name != "wsc" {
                assert_ne!(preset, &KICK_WSC, "Kick preset `{}` is not distinct", name);
            }
        }
        for name in SNARE_PRESETS {
            let preset = snare_preset(Some(name));
            if *name != "wsc" {
                assert_ne!(preset, &SNARE_WSC, "Snare preset `{}` is not distinct", name);
            }
        }
        for name in HIHAT_PRESETS {
            let preset = hihat_preset(Some(name));
            if *name != "wsc" {
                assert_ne!(preset, &HIHAT_WSC, "HiHat preset `{}` is not distinct", name);
            }
        }
        for name in CLAP_PRESETS {
            let preset = clap_preset(Some(name));
            if *name != "wsc" {
                assert_ne!(preset, &CLAP_WSC, "Clap preset `{}` is not distinct", name);
            }
        }
        for name in RIMSHOT_PRESETS {
            let preset = rimshot_preset(Some(name));
            if *name != "wsc" {
                assert_ne!(preset, &RIMSHOT_WSC, "Rimshot preset `{}` is not distinct", name);
            }
        }
    }

    /// Bare drums (no params) resolve to exactly the wsc voicing.
    #[test]
    fn bare_drum_is_wsc() {
        let rk = resolve_kick(None);
        assert_eq!(rk.internal, &KICK_WSC);
        // Spot-check the formula evaluation at macro centers matches the
        // engine's historical defaults.
        assert!((rk.amp_decay - 0.625).abs() < 1e-12); // 0.35 + 0.5*0.55
        assert!((rk.click_amount - 0.275).abs() < 1e-12); // 0.10 + 0.5*0.35
        assert!((rk.click_freq - 2350.0).abs() < 1e-9); // 1700 + 0.5*1300
    }

    /// Explicit params override preset formulas.
    #[test]
    fn explicit_params_override_preset() {
        use num_rational::Rational64;
        let params = KickParams {
            preset: Some("808".to_string()),
            click_freq: Some(Rational64::new(2000, 1)),
            ..Default::default()
        };
        let rk = resolve_kick(Some(&params));
        assert_eq!(rk.internal, &KICK_808);
        assert!((rk.click_freq - 2000.0).abs() < 1e-12);
        // Un-overridden fields come from the 808 table.
        assert!((rk.pitch_decay - 0.060).abs() < 1e-12);
    }
}
