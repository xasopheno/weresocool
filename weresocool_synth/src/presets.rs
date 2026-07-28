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

use weresocool_ast::{
    ClapParams, CowbellParams, CrashParams, HiHatParams, KickParams, RideParams, RimshotParams,
    ShakerParams, SnareParams, TomParams,
};
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

// ═══════════════════════════════════════════════════════════════════════
// TOM — a struck membrane with no wires.
//
// The tom is the kick's tonal cousin: same waveguide + mode + shell
// architecture, but the pitch bend is a *musical* interval (~a whole tone,
// not two octaves), the decay is long, and there is no wire noise to hide
// behind. That leaves the mode tuning exposed, which is why the ratios
// here are the real circular-membrane values rather than anything rounder.
// ═══════════════════════════════════════════════════════════════════════

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TomPreset {
    // Macro centers.
    pub attack: f64,
    pub body: f64,
    pub tone: f64,
    pub length: f64,
    // Formula coefficients: value = k.0 + k.1 × macro.
    pub amp_decay_k: (f64, f64),     // on length
    pub saturation_k: (f64, f64),    // on body
    pub shell_k: (f64, f64),         // on body
    pub attack_amount_k: (f64, f64), // on attack
    // Direct param defaults.
    pub pitch_decay: f64,
    pub pitch_range: f64,
    pub ks_mix: f64,
    pub velocity_tilt: f64,
    // ── Internal-only knobs ──
    /// Where this drum sits at the canonical `f: 60` drum header, in Hz.
    /// `f_base = base_pitch · tune · (note/60)^pitch_track`. Toms are
    /// genuinely pitched instruments, so `pitch_track` is 1.0 across the
    /// family — `Tom | Fm 2` is an honest octave, and a three-tom kit is
    /// `Tom | Fm 2/3`, `Tom`, `Tom | Fm 3/2`.
    pub base_pitch: f64,
    pub pitch_track: f64,
    /// Membrane mode ratios, amplitudes, and decay multipliers. Ratios are
    /// the first circular-membrane Bessel zeros, pulled slightly toward
    /// harmonic by the shell coupling of a real tuned drum.
    pub mode_ratios: [f64; 4],
    pub mode_amps: [f64; 4],
    pub mode_decays: [f64; 4],
    /// Additive-mode level under the KS waveguide body.
    pub sine_level: f64,
    /// Body lowpass sweep, multiples of f_base: (transient hi, settled lo).
    pub cutoff_mult: (f64, f64),
    pub body_q: f64,
    /// Stick/mallet click bandpass (Hz, Q) and ring make-up gain.
    pub click_bp: (f64, f64),
    pub click_gain: f64,
    /// Broadband stick-contact noise level, mixed alongside the resonator
    /// ring. A bandpass at a usable Q rings for well under a millisecond,
    /// which is too narrow to read as a stick on a drum this low — the real
    /// sound of wood on a head is broadband. This is also the only part of
    /// the tom that is stereo-decorrelated: body centred, contact wide.
    pub stick_gain: f64,
    /// Wooden shell resonance: ratio above f_base, and Q.
    pub shell_ratio: f64,
    pub shell_q: f64,
    pub comp: Comp,
    pub drive_out: f64,
    pub gain_trim: f64,
}

const TOM_WSC: TomPreset = TomPreset {
    attack: 0.5,
    body: 0.5,
    tone: 0.5,
    length: 0.5,
    amp_decay_k: (0.30, 0.65),
    saturation_k: (0.22, 0.30),
    shell_k: (0.28, 0.20),
    attack_amount_k: (0.25, 0.45),
    pitch_decay: 0.100,
    pitch_range: 1.22,
    ks_mix: 0.60,
    velocity_tilt: 0.6,
    base_pitch: 110.0,
    pitch_track: 1.0,
    mode_ratios: [1.0, 1.58, 2.14, 2.65],
    mode_amps: [1.0, 0.42, 0.20, 0.10],
    mode_decays: [1.0, 1.7, 2.6, 3.6],
    sine_level: 0.75,
    cutoff_mult: (7.0, 2.0),
    body_q: 1.1,
    click_bp: (2200.0, 5.0),
    click_gain: 26.0,
    stick_gain: 0.34,
    shell_ratio: 2.55,
    shell_q: 6.0,
    comp: Comp { threshold: 0.30, ratio: 3.5, dry: 0.62, wet: 0.75, attack_s: 0.0012, release_s: 0.110 },
    drive_out: 1.0,
    gain_trim: 1.0,
};

/// 808: the long tuned sine boom — nearly pure fundamental, slow settle,
/// almost no stick. The synthetic tom, not a drum.
const TOM_808: TomPreset = TomPreset {
    amp_decay_k: (0.55, 1.10),
    saturation_k: (0.14, 0.20),
    shell_k: (0.06, 0.08),
    attack_amount_k: (0.04, 0.16),
    pitch_decay: 0.140,
    pitch_range: 1.35,
    ks_mix: 0.20,
    base_pitch: 100.0,
    mode_amps: [1.0, 0.16, 0.05, 0.02],
    sine_level: 1.0,
    cutoff_mult: (4.0, 1.4),
    click_bp: (1500.0, 4.0),
    click_gain: 10.0,
    stick_gain: 0.10,
    shell_ratio: 2.20,
    comp: Comp { threshold: 0.32, ratio: 3.0, dry: 0.70, wet: 0.60, attack_s: 0.0014, release_s: 0.140 },
    gain_trim: 0.742,
    ..TOM_WSC
};

/// 909: punchy synthetic tom — hard beater, deep fast bend, tight decay.
const TOM_909: TomPreset = TomPreset {
    attack: 0.65,
    amp_decay_k: (0.20, 0.42),
    saturation_k: (0.32, 0.36),
    shell_k: (0.18, 0.14),
    attack_amount_k: (0.30, 0.50),
    pitch_decay: 0.055,
    pitch_range: 1.55,
    ks_mix: 0.45,
    base_pitch: 125.0,
    mode_amps: [1.0, 0.30, 0.12, 0.05],
    cutoff_mult: (9.0, 2.4),
    click_bp: (2800.0, 5.0),
    click_gain: 31.0,
    stick_gain: 0.42,
    comp: Comp { threshold: 0.24, ratio: 5.0, dry: 0.52, wet: 0.95, attack_s: 0.0007, release_s: 0.070 },
    drive_out: 1.12,
    gain_trim: 1.018,
    ..TOM_WSC
};

/// acoustic: a real struck drum — full mode set, strong shell, waveguide
/// dominant, modest bend. The tom you'd mic in a room.
const TOM_ACOUSTIC: TomPreset = TomPreset {
    body: 0.6,
    amp_decay_k: (0.35, 0.75),
    saturation_k: (0.12, 0.18),
    shell_k: (0.40, 0.30),
    attack_amount_k: (0.18, 0.42),
    pitch_decay: 0.085,
    pitch_range: 1.15,
    ks_mix: 0.90,
    velocity_tilt: 0.75,
    base_pitch: 115.0,
    mode_ratios: [1.0, 1.594, 2.136, 2.296],
    mode_amps: [1.0, 0.52, 0.30, 0.20],
    mode_decays: [1.0, 1.5, 2.2, 2.5],
    sine_level: 0.55,
    cutoff_mult: (8.0, 2.2),
    body_q: 0.95,
    click_bp: (2600.0, 4.0),
    click_gain: 28.0,
    stick_gain: 0.38,
    shell_ratio: 2.80,
    shell_q: 7.5,
    comp: Comp { threshold: 0.34, ratio: 2.8, dry: 0.72, wet: 0.55, attack_s: 0.0016, release_s: 0.130 },
    gain_trim: 1.136,
    ..TOM_WSC
};

/// conga: a hand drum — high, dry, tight, almost no bend, sharp palm slap.
/// Same membrane physics an octave up with the sustain taken away.
const TOM_CONGA: TomPreset = TomPreset {
    attack: 0.7,
    body: 0.35,
    length: 0.3,
    amp_decay_k: (0.12, 0.30),
    saturation_k: (0.18, 0.22),
    shell_k: (0.20, 0.16),
    attack_amount_k: (0.35, 0.50),
    pitch_decay: 0.035,
    pitch_range: 1.08,
    ks_mix: 0.80,
    velocity_tilt: 0.8,
    base_pitch: 215.0,
    mode_ratios: [1.0, 1.62, 2.24, 2.90],
    mode_amps: [1.0, 0.48, 0.26, 0.14],
    mode_decays: [1.0, 1.9, 3.0, 4.0],
    sine_level: 0.50,
    cutoff_mult: (9.0, 2.6),
    click_bp: (3400.0, 4.5),
    click_gain: 30.0,
    stick_gain: 0.44,
    shell_ratio: 3.10,
    shell_q: 8.0,
    comp: Comp { threshold: 0.30, ratio: 3.2, dry: 0.66, wet: 0.70, attack_s: 0.0009, release_s: 0.060 },
    gain_trim: 1.344,
    ..TOM_WSC
};

/// dust: lofi — dark, damped, short. A tom off a worn record.
const TOM_DUST: TomPreset = TomPreset {
    tone: 0.22,
    amp_decay_k: (0.20, 0.40),
    saturation_k: (0.30, 0.30),
    shell_k: (0.18, 0.14),
    attack_amount_k: (0.06, 0.20),
    pitch_decay: 0.090,
    pitch_range: 1.18,
    ks_mix: 0.70,
    base_pitch: 98.0,
    mode_amps: [1.0, 0.30, 0.10, 0.04],
    cutoff_mult: (3.4, 1.3),
    body_q: 0.9,
    click_bp: (1300.0, 3.5),
    click_gain: 9.0,
    stick_gain: 0.09,
    comp: Comp { threshold: 0.30, ratio: 3.0, dry: 0.70, wet: 0.65, attack_s: 0.0016, release_s: 0.130 },
    gain_trim: 1.021,
    ..TOM_WSC
};

/// glass: a bright crystalline tonal drum — near-harmonic modes, very high
/// mode content, long clean ring, almost no saturation.
const TOM_GLASS: TomPreset = TomPreset {
    tone: 0.9,
    body: 0.25,
    amp_decay_k: (0.50, 1.00),
    saturation_k: (0.02, 0.06),
    shell_k: (0.10, 0.10),
    attack_amount_k: (0.10, 0.30),
    pitch_decay: 0.040,
    pitch_range: 1.05,
    ks_mix: 0.35,
    base_pitch: 190.0,
    mode_ratios: [1.0, 2.00, 3.01, 4.02],
    mode_amps: [1.0, 0.55, 0.34, 0.22],
    mode_decays: [1.0, 1.2, 1.5, 1.8],
    sine_level: 1.0,
    cutoff_mult: (14.0, 5.0),
    body_q: 0.8,
    click_bp: (4200.0, 8.0),
    click_gain: 13.0,
    stick_gain: 0.14,
    shell_ratio: 4.05,
    shell_q: 14.0,
    comp: Comp { threshold: 0.40, ratio: 2.0, dry: 0.82, wet: 0.35, attack_s: 0.0020, release_s: 0.150 },
    gain_trim: 0.839,
    ..TOM_WSC
};

/// doom: a cavernous floor tom — very low, very long, dark filters, huge bend.
const TOM_DOOM: TomPreset = TomPreset {
    tone: 0.15,
    body: 0.8,
    length: 0.85,
    amp_decay_k: (0.80, 1.60),
    saturation_k: (0.30, 0.34),
    shell_k: (0.42, 0.26),
    attack_amount_k: (0.05, 0.20),
    pitch_decay: 0.180,
    pitch_range: 1.45,
    ks_mix: 0.75,
    base_pitch: 55.0,
    mode_amps: [1.0, 0.26, 0.09, 0.03],
    cutoff_mult: (3.0, 1.2),
    body_q: 1.4,
    click_bp: (900.0, 3.0),
    click_gain: 9.0,
    stick_gain: 0.08,
    shell_ratio: 2.10,
    comp: Comp { threshold: 0.30, ratio: 3.0, dry: 0.68, wet: 0.70, attack_s: 0.0018, release_s: 0.180 },
    gain_trim: 0.518,
    ..TOM_WSC
};

/// crush: distorted industrial tom — driven hard into the output stage,
/// clangy inharmonic modes, aggressive stick.
const TOM_CRUSH: TomPreset = TomPreset {
    attack: 0.75,
    body: 0.9,
    amp_decay_k: (0.18, 0.40),
    saturation_k: (0.75, 0.55),
    shell_k: (0.30, 0.22),
    attack_amount_k: (0.40, 0.55),
    pitch_decay: 0.060,
    pitch_range: 1.40,
    ks_mix: 0.55,
    base_pitch: 118.0,
    mode_ratios: [1.0, 1.71, 2.47, 3.31],
    mode_amps: [1.0, 0.55, 0.34, 0.22],
    mode_decays: [1.0, 1.4, 1.9, 2.4],
    cutoff_mult: (11.0, 3.0),
    body_q: 1.6,
    click_bp: (3200.0, 4.0),
    click_gain: 33.0,
    stick_gain: 0.50,
    comp: Comp { threshold: 0.18, ratio: 7.0, dry: 0.45, wet: 1.05, attack_s: 0.0006, release_s: 0.060 },
    drive_out: 1.55,
    gain_trim: 0.704,
    ..TOM_WSC
};

/// air: a soft brushed tom — no stick at all, broad breathy body, long soft tail.
const TOM_AIR: TomPreset = TomPreset {
    attack: 0.1,
    body: 0.3,
    length: 0.7,
    amp_decay_k: (0.45, 0.90),
    saturation_k: (0.04, 0.10),
    shell_k: (0.34, 0.24),
    attack_amount_k: (0.00, 0.10),
    pitch_decay: 0.120,
    pitch_range: 1.10,
    ks_mix: 0.85,
    velocity_tilt: 0.8,
    base_pitch: 105.0,
    mode_amps: [1.0, 0.36, 0.16, 0.07],
    mode_decays: [1.0, 1.9, 3.0, 4.2],
    sine_level: 0.60,
    cutoff_mult: (4.5, 1.6),
    body_q: 0.85,
    click_bp: (1200.0, 3.0),
    click_gain: 6.0,
    stick_gain: 0.03,
    comp: Comp { threshold: 0.38, ratio: 2.2, dry: 0.78, wet: 0.40, attack_s: 0.0025, release_s: 0.160 },
    gain_trim: 1.162,
    ..TOM_WSC
};

/// snap: a gated tom — all transient, tail cut short. Very dry "tok".
const TOM_SNAP: TomPreset = TomPreset {
    attack: 0.85,
    length: 0.1,
    amp_decay_k: (0.055, 0.11),
    saturation_k: (0.28, 0.30),
    shell_k: (0.14, 0.12),
    attack_amount_k: (0.45, 0.55),
    pitch_decay: 0.028,
    pitch_range: 1.50,
    ks_mix: 0.40,
    base_pitch: 135.0,
    mode_amps: [1.0, 0.34, 0.14, 0.06],
    mode_decays: [1.0, 2.2, 3.4, 4.6],
    cutoff_mult: (10.0, 3.0),
    click_bp: (3000.0, 5.0),
    click_gain: 20.0,
    stick_gain: 0.46,
    comp: Comp { threshold: 0.22, ratio: 6.0, dry: 0.50, wet: 1.00, attack_s: 0.0005, release_s: 0.035 },
    drive_out: 1.1,
    gain_trim: 1.96,
    ..TOM_WSC
};

pub fn tom_preset(name: Option<&str>) -> &'static TomPreset {
    match name {
        Some("808") => &TOM_808,
        Some("909") => &TOM_909,
        Some("acoustic") => &TOM_ACOUSTIC,
        Some("conga") => &TOM_CONGA,
        Some("dust") => &TOM_DUST,
        Some("glass") => &TOM_GLASS,
        Some("doom") => &TOM_DOOM,
        Some("crush") => &TOM_CRUSH,
        Some("air") => &TOM_AIR,
        Some("snap") => &TOM_SNAP,
        _ => &TOM_WSC,
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ResolvedTom {
    pub tune: f64,
    /// Fully resolved fundamental in Hz (register anchor + note tracking).
    pub base_freq: f64,
    pub pitch_decay: f64,
    pub pitch_range: f64,
    pub amp_decay: f64,
    pub saturation: f64,
    pub ks_mix: f64,
    pub attack_amount: f64,
    pub shell: f64,
    pub velocity_tilt: f64,
    /// Body cutoff scaling from `tone` — bends the preset's sweep range.
    pub tone_scale: f64,
    pub internal: &'static TomPreset,
}

pub fn resolve_tom(params: Option<&TomParams>) -> ResolvedTom {
    let pre = tom_preset(params.and_then(|p| p.preset.as_deref()));
    let attack = p(params.and_then(|x| x.attack), pre.attack);
    let body = p(params.and_then(|x| x.body), pre.body);
    let tone = p(params.and_then(|x| x.tone), pre.tone);
    let length = p(params.and_then(|x| x.length), pre.length);
    ResolvedTom {
        tune: p(params.and_then(|x| x.tune), 1.0),
        base_freq: 0.0, // filled in per note by the engine (needs `info.frequency`)
        pitch_decay: p(params.and_then(|x| x.pitch_decay), pre.pitch_decay),
        pitch_range: p(params.and_then(|x| x.pitch_range), pre.pitch_range),
        amp_decay: p(params.and_then(|x| x.amp_decay), pre.amp_decay_k.0 + length * pre.amp_decay_k.1),
        saturation: p(params.and_then(|x| x.saturation), pre.saturation_k.0 + body * pre.saturation_k.1),
        ks_mix: p(params.and_then(|x| x.ks_mix), pre.ks_mix),
        attack_amount: p(params.and_then(|x| x.attack_amount), pre.attack_amount_k.0 + attack * pre.attack_amount_k.1),
        shell: p(params.and_then(|x| x.shell), pre.shell_k.0 + body * pre.shell_k.1),
        velocity_tilt: p(params.and_then(|x| x.velocity_tilt), pre.velocity_tilt),
        // `tone` at its center leaves the preset sweep untouched; the ends
        // halve or double the open-cutoff range.
        tone_scale: 0.5 + tone,
        internal: pre,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// RIDE — a large plate played with the tip of a stick.
//
// What identifies a ride is not the wash, it's the ARTICULATION: a clear,
// repeatable, pitched *ping* on every hit, sitting on a bed of wash that
// accumulates over a phrase. Get the ping wrong and it reads as an open
// hi-hat no matter how long the tail is.
//
// Three layers:
//   1. Ping   — a high-Q bandpass rung by a sub-ms noise burst (~2.5-3.5 kHz)
//   2. Bell   — four low inharmonic partials that ring far longer than the
//               plate modes; the sustained "gong" you hear under a ride
//   3. Modes + wash — the shared 22-mode Bessel plate bank run at a much
//               lower `mode_freq_scale` (bigger cymbal) and much slower
//               decay, plus two noise bands whose level BUILDS over the
//               first ~40 ms instead of starting full.
// ═══════════════════════════════════════════════════════════════════════

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RidePreset {
    // Macro centers.
    pub attack: f64,
    pub metal: f64,
    pub length: f64,
    // Formula coefficients.
    /// decay_rate = k.0 + (1 − length) × k.1 — per second.
    pub decay_k: (f64, f64),
    pub shimmer_k: (f64, f64),       // on metal
    pub brightness_k: (f64, f64),    // on metal
    pub attack_amount_k: (f64, f64), // on attack — the ping
    pub wash_k: (f64, f64),          // on metal
    // Direct param defaults.
    pub bell_amount: f64,
    pub velocity_tilt: f64,
    // ── Internal-only knobs ──
    /// Register anchor: `f_base = base_pitch · tune · (note/60)^pitch_track`.
    /// Cymbals are unpitched, so `pitch_track` is small — `Fm` colours the
    /// plate rather than transposing it, and a bass-register header never
    /// drags the cymbal into the mud.
    pub base_pitch: f64,
    pub pitch_track: f64,
    /// Plate "size": multiplies all 22 mode ratios (<1 = bigger/lower).
    pub mode_freq_scale: f64,
    /// Spectral tilt across the mode bank: amp ×= tilt^(i/21).
    pub mode_amp_tilt: f64,
    /// Nonlinear modal coupling drive — large plates couple harder.
    pub coupling: f64,
    /// Bell partials: inharmonic ratios on f_base, amplitudes, and the
    /// decay multiplier applied to the whole group (<1 = rings longest).
    pub bell_ratios: [f64; 4],
    pub bell_amps: [f64; 4],
    pub bell_decay: f64,
    /// Stick ping: bandpass (Hz, Q), ring make-up gain, decay tau (s).
    pub ping_bp: (f64, f64),
    pub ping_gain: f64,
    pub ping_tau: f64,
    /// Wash noise bands: (center Hz, Q, gain, decay multiplier) × 2.
    pub wash_lo: (f64, f64, f64, f64),
    pub wash_hi: (f64, f64, f64, f64),
    /// Time constant (s) over which the wash builds to full level.
    pub wash_build: f64,
    /// Highpass keeping the plate out of the kick band (Hz).
    pub hp_cutoff: f64,
    /// (modes, wash) blend.
    pub mix: (f64, f64),
    pub gain_trim: f64,
}

const RIDE_WSC: RidePreset = RidePreset {
    attack: 0.5,
    metal: 0.5,
    length: 0.5,
    decay_k: (0.85, 1.55),
    shimmer_k: (0.90, 0.30),
    brightness_k: (0.45, 0.60),
    attack_amount_k: (0.30, 0.55),
    wash_k: (0.30, 0.25),
    bell_amount: 0.35,
    velocity_tilt: 0.55,
    base_pitch: 340.0,
    pitch_track: 0.30,
    mode_freq_scale: 0.55,
    mode_amp_tilt: 1.0,
    coupling: 0.10,
    bell_ratios: [1.0, 1.47, 2.09, 2.83],
    bell_amps: [0.55, 0.34, 0.20, 0.12],
    bell_decay: 0.55,
    ping_bp: (2900.0, 9.0),
    ping_gain: 25.0,
    ping_tau: 0.012,
    wash_lo: (4200.0, 3.0, 1.30, 0.80),
    wash_hi: (9000.0, 2.6, 0.85, 1.70),
    wash_build: 0.030,
    hp_cutoff: 900.0,
    mix: (0.55, 0.45),
    gain_trim: 1.0,
};

/// 909: the bright machine ride — sizzly, air-forward, shorter.
const RIDE_909: RidePreset = RidePreset {
    metal: 0.7,
    decay_k: (1.30, 1.90),
    shimmer_k: (0.95, 0.35),
    brightness_k: (0.65, 0.60),
    attack_amount_k: (0.35, 0.55),
    wash_k: (0.42, 0.30),
    bell_amount: 0.22,
    mode_freq_scale: 0.62,
    mode_amp_tilt: 1.55,
    coupling: 0.08,
    ping_bp: (3400.0, 8.0),
    ping_gain: 27.0,
    ping_tau: 0.008,
    wash_lo: (5200.0, 2.8, 1.55, 0.85),
    wash_hi: (11000.0, 2.4, 1.30, 1.55),
    wash_build: 0.018,
    mix: (0.42, 0.58),
    gain_trim: 0.502,
    ..RIDE_WSC
};

/// acoustic: the jazz ride — a very defined ping over a complex, slowly
/// blooming wash. Strong coupling, dense beating, long bell.
const RIDE_ACOUSTIC: RidePreset = RidePreset {
    attack: 0.7,
    decay_k: (0.62, 1.20),
    shimmer_k: (0.88, 0.28),
    brightness_k: (0.42, 0.55),
    attack_amount_k: (0.45, 0.60),
    wash_k: (0.24, 0.22),
    bell_amount: 0.40,
    velocity_tilt: 0.75,
    base_pitch: 310.0,
    mode_freq_scale: 0.50,
    mode_amp_tilt: 0.90,
    coupling: 0.15,
    bell_ratios: [1.0, 1.51, 2.17, 2.94],
    bell_amps: [0.60, 0.38, 0.24, 0.15],
    bell_decay: 0.45,
    ping_bp: (2600.0, 12.0),
    ping_gain: 28.0,
    ping_tau: 0.016,
    wash_lo: (3600.0, 3.2, 1.20, 0.72),
    wash_hi: (8200.0, 2.8, 0.70, 1.80),
    wash_build: 0.045,
    mix: (0.62, 0.38),
    gain_trim: 1.015,
    ..RIDE_WSC
};

/// bell: played on the bell of the cymbal — the partials dominate, the
/// wash recedes, and the whole thing is unmistakably pitched.
const RIDE_BELL: RidePreset = RidePreset {
    attack: 0.8,
    decay_k: (0.60, 1.00),
    brightness_k: (0.35, 0.45),
    attack_amount_k: (0.50, 0.60),
    wash_k: (0.10, 0.12),
    bell_amount: 1.0,
    base_pitch: 420.0,
    mode_freq_scale: 0.60,
    mode_amp_tilt: 0.70,
    coupling: 0.07,
    bell_ratios: [1.0, 1.50, 2.24, 3.05],
    bell_amps: [0.85, 0.50, 0.30, 0.18],
    bell_decay: 0.30,
    ping_bp: (3100.0, 14.0),
    ping_gain: 26.0,
    ping_tau: 0.014,
    wash_lo: (4000.0, 3.5, 0.70, 0.85),
    wash_hi: (8500.0, 3.0, 0.35, 1.70),
    wash_build: 0.040,
    mix: (0.78, 0.22),
    gain_trim: 0.303,
    ..RIDE_WSC
};

/// dark: a low, dry, washy ride — soft ping, dark plate, no top sizzle.
const RIDE_DARK: RidePreset = RidePreset {
    metal: 0.25,
    decay_k: (0.75, 1.30),
    shimmer_k: (0.82, 0.24),
    brightness_k: (0.22, 0.35),
    attack_amount_k: (0.20, 0.40),
    wash_k: (0.34, 0.24),
    bell_amount: 0.28,
    base_pitch: 265.0,
    mode_freq_scale: 0.46,
    mode_amp_tilt: 0.45,
    coupling: 0.13,
    bell_decay: 0.50,
    ping_bp: (1900.0, 8.0),
    ping_gain: 21.0,
    ping_tau: 0.014,
    wash_lo: (2900.0, 3.2, 1.30, 0.75),
    wash_hi: (6200.0, 3.0, 0.55, 1.85),
    wash_build: 0.038,
    hp_cutoff: 700.0,
    mix: (0.58, 0.42),
    gain_trim: 1.435,
    ..RIDE_WSC
};

/// glass: a pure crystalline bell-ride — near-tonal partials, very high Q,
/// enormous ring, noise all but gone.
const RIDE_GLASS: RidePreset = RidePreset {
    metal: 0.85,
    length: 0.85,
    decay_k: (0.70, 1.10),
    shimmer_k: (0.98, 0.30),
    brightness_k: (0.55, 0.55),
    attack_amount_k: (0.30, 0.45),
    wash_k: (0.05, 0.08),
    bell_amount: 0.85,
    base_pitch: 520.0,
    mode_freq_scale: 0.72,
    mode_amp_tilt: 1.20,
    coupling: 0.03,
    bell_ratios: [1.0, 2.00, 2.99, 4.01],
    bell_amps: [0.80, 0.46, 0.28, 0.16],
    bell_decay: 0.22,
    ping_bp: (4200.0, 18.0),
    ping_gain: 23.0,
    ping_tau: 0.010,
    wash_lo: (7000.0, 5.0, 0.45, 0.90),
    wash_hi: (13000.0, 4.0, 0.30, 1.40),
    wash_build: 0.025,
    hp_cutoff: 1400.0,
    mix: (0.85, 0.15),
    gain_trim: 0.25,
    ..RIDE_WSC
};

/// doom: a vast dark plate — very low, very long, almost no articulation.
const RIDE_DOOM: RidePreset = RidePreset {
    metal: 0.15,
    length: 0.9,
    decay_k: (0.55, 0.90),
    shimmer_k: (0.78, 0.20),
    brightness_k: (0.15, 0.28),
    attack_amount_k: (0.12, 0.28),
    wash_k: (0.40, 0.26),
    bell_amount: 0.42,
    base_pitch: 180.0,
    mode_freq_scale: 0.34,
    mode_amp_tilt: 0.35,
    coupling: 0.18,
    bell_decay: 0.35,
    ping_bp: (1300.0, 7.0),
    ping_gain: 18.0,
    ping_tau: 0.018,
    wash_lo: (2000.0, 3.4, 1.35, 0.65),
    wash_hi: (4600.0, 3.2, 0.45, 1.90),
    wash_build: 0.055,
    hp_cutoff: 450.0,
    mix: (0.60, 0.40),
    gain_trim: 1.425,
    ..RIDE_WSC
};

/// crush: a trashy, distorted, heavily-coupled plate — chaotic and
/// dissonant, closer to sheet metal than a cymbal.
const RIDE_CRUSH: RidePreset = RidePreset {
    attack: 0.7,
    decay_k: (2.2, 2.8),
    shimmer_k: (1.10, 0.35),
    brightness_k: (0.70, 0.60),
    attack_amount_k: (0.45, 0.60),
    wash_k: (0.50, 0.30),
    bell_amount: 0.18,
    base_pitch: 395.0,
    mode_freq_scale: 0.67,
    mode_amp_tilt: 1.70,
    coupling: 0.34,
    bell_ratios: [1.0, 1.63, 2.41, 3.37],
    bell_amps: [0.40, 0.34, 0.26, 0.20],
    bell_decay: 0.75,
    ping_bp: (3800.0, 5.0),
    ping_gain: 29.0,
    ping_tau: 0.007,
    wash_lo: (5600.0, 2.2, 1.70, 0.90),
    wash_hi: (11500.0, 2.0, 1.45, 1.35),
    wash_build: 0.012,
    mix: (0.45, 0.55),
    gain_trim: 0.505,
    ..RIDE_WSC
};

/// air: a bowed/brushed cymbal — pure wash, no stick contact at all.
const RIDE_AIR: RidePreset = RidePreset {
    attack: 0.05,
    length: 0.8,
    decay_k: (0.90, 1.30),
    shimmer_k: (0.85, 0.25),
    brightness_k: (0.35, 0.45),
    attack_amount_k: (0.00, 0.08),
    wash_k: (0.55, 0.30),
    bell_amount: 0.30,
    velocity_tilt: 0.8,
    base_pitch: 300.0,
    mode_freq_scale: 0.50,
    mode_amp_tilt: 0.80,
    coupling: 0.09,
    bell_decay: 0.42,
    ping_bp: (2400.0, 6.0),
    ping_gain: 10.0,
    ping_tau: 0.030,
    wash_lo: (3800.0, 2.4, 1.60, 0.62),
    wash_hi: (8000.0, 2.2, 0.95, 1.35),
    wash_build: 0.110,
    mix: (0.35, 0.65),
    gain_trim: 0.39,
    ..RIDE_WSC
};

/// snap: a choked ride — all ping, tail gated off. The stab, not the ring.
const RIDE_SNAP: RidePreset = RidePreset {
    attack: 0.9,
    length: 0.05,
    decay_k: (16.0, 14.0),
    brightness_k: (0.55, 0.55),
    attack_amount_k: (0.60, 0.55),
    wash_k: (0.22, 0.18),
    bell_amount: 0.12,
    base_pitch: 380.0,
    mode_freq_scale: 0.62,
    mode_amp_tilt: 1.20,
    coupling: 0.05,
    bell_decay: 1.6,
    ping_bp: (3200.0, 10.0),
    ping_gain: 28.0,
    ping_tau: 0.005,
    wash_lo: (5000.0, 3.0, 1.30, 0.90),
    wash_hi: (10500.0, 2.6, 1.00, 1.30),
    wash_build: 0.004,
    mix: (0.50, 0.50),
    gain_trim: 0.794,
    ..RIDE_WSC
};

pub fn ride_preset(name: Option<&str>) -> &'static RidePreset {
    match name {
        Some("909") => &RIDE_909,
        Some("acoustic") => &RIDE_ACOUSTIC,
        Some("bell") => &RIDE_BELL,
        Some("dark") => &RIDE_DARK,
        Some("glass") => &RIDE_GLASS,
        Some("doom") => &RIDE_DOOM,
        Some("crush") => &RIDE_CRUSH,
        Some("air") => &RIDE_AIR,
        Some("snap") => &RIDE_SNAP,
        _ => &RIDE_WSC,
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ResolvedRide {
    pub tune: f64,
    pub base_freq: f64,
    pub decay_rate: f64,
    pub shimmer: f64,
    pub brightness: f64,
    pub attack_amount: f64,
    pub bell_amount: f64,
    pub wash: f64,
    pub velocity_tilt: f64,
    /// Per-mode amplitudes with the preset's spectral tilt baked in.
    pub mode_amp: [f64; 22],
    pub internal: &'static RidePreset,
}

pub fn resolve_ride(params: Option<&RideParams>) -> ResolvedRide {
    let pre = ride_preset(params.and_then(|p| p.preset.as_deref()));
    let attack = p(params.and_then(|x| x.attack), pre.attack);
    let metal = p(params.and_then(|x| x.metal), pre.metal);
    let length = p(params.and_then(|x| x.length), pre.length);
    let mut mode_amp = [0.0; 22];
    for (i, (_, base_amp, _)) in HIHAT_MODES.iter().enumerate() {
        mode_amp[i] = base_amp * pre.mode_amp_tilt.powf(i as f64 / 21.0);
    }
    ResolvedRide {
        tune: p(params.and_then(|x| x.tune), 1.0),
        base_freq: 0.0, // filled in per note by the engine
        decay_rate: p(params.and_then(|x| x.decay_rate), pre.decay_k.0 + (1.0 - length) * pre.decay_k.1),
        shimmer: p(params.and_then(|x| x.shimmer), pre.shimmer_k.0 + metal * pre.shimmer_k.1),
        brightness: p(params.and_then(|x| x.brightness), pre.brightness_k.0 + metal * pre.brightness_k.1),
        attack_amount: p(params.and_then(|x| x.attack_amount), pre.attack_amount_k.0 + attack * pre.attack_amount_k.1),
        bell_amount: p(params.and_then(|x| x.bell_amount), pre.bell_amount),
        wash: p(params.and_then(|x| x.wash), pre.wash_k.0 + metal * pre.wash_k.1),
        velocity_tilt: p(params.and_then(|x| x.velocity_tilt), pre.velocity_tilt),
        mode_amp,
        internal: pre,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// CRASH — the same plate physics as the ride, played to explode.
//
// The defining feature is the SWELL. A crash does not start at full level:
// energy spreads across the plate over 5-20 ms, so the level rises into a
// peak and only then decays. Skipping that is what makes synthetic crashes
// sound like someone turned up a hi-hat.
//
// The second feature is that the tail gets DARKER, fast. High modes shed
// energy several times quicker than low ones (already in `HIHAT_MODES`'s
// per-mode decay multipliers), and on top of that the whole cymbal runs
// through a lowpass whose cutoff falls from `lp_sweep.0` to `lp_sweep.1`.
// ═══════════════════════════════════════════════════════════════════════

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CrashPreset {
    // Macro centers.
    pub attack: f64,
    pub metal: f64,
    pub length: f64,
    // Formula coefficients.
    /// decay_rate = k.0 + (1 − length) × k.1 — per second.
    pub decay_k: (f64, f64),
    pub shimmer_k: (f64, f64),    // on metal
    pub brightness_k: (f64, f64), // on metal
    /// Swell rise time (s) = k.0 + (1 − attack) × k.1 — more attack, less swell.
    pub swell_k: (f64, f64),
    pub wash_k: (f64, f64), // on metal
    // Direct param defaults.
    pub velocity_tilt: f64,
    // ── Internal-only knobs ──
    pub base_pitch: f64,
    pub pitch_track: f64,
    pub mode_freq_scale: f64,
    pub mode_amp_tilt: f64,
    pub coupling: f64,
    /// Wash noise bands: (center Hz, Q, gain, decay multiplier) × 2.
    pub wash_lo: (f64, f64, f64, f64),
    pub wash_hi: (f64, f64, f64, f64),
    pub hp_cutoff: f64,
    /// (modes, wash) blend.
    pub mix: (f64, f64),
    /// Whole-cymbal lowpass sweep (onset Hz → tail Hz) — the tail darkening.
    pub lp_sweep: (f64, f64),
    pub gain_trim: f64,
}

const CRASH_WSC: CrashPreset = CrashPreset {
    attack: 0.5,
    metal: 0.5,
    length: 0.5,
    decay_k: (0.90, 1.70),
    shimmer_k: (0.85, 0.30),
    brightness_k: (0.55, 0.65),
    swell_k: (0.0015, 0.0070),
    wash_k: (0.55, 0.30),
    velocity_tilt: 0.5,
    base_pitch: 300.0,
    pitch_track: 0.30,
    mode_freq_scale: 0.44,
    mode_amp_tilt: 1.25,
    coupling: 0.14,
    wash_lo: (4000.0, 2.2, 1.50, 0.75),
    wash_hi: (9500.0, 2.0, 1.20, 1.90),
    hp_cutoff: 700.0,
    mix: (0.42, 0.58),
    lp_sweep: (16000.0, 3200.0),
    gain_trim: 1.0,
};

/// 909: the bright machine crash — fast, sizzly, air-forward.
const CRASH_909: CrashPreset = CrashPreset {
    metal: 0.7,
    decay_k: (1.30, 2.00),
    brightness_k: (0.70, 0.60),
    swell_k: (0.002, 0.008),
    wash_k: (0.65, 0.30),
    mode_freq_scale: 0.52,
    mode_amp_tilt: 1.70,
    coupling: 0.10,
    wash_lo: (5000.0, 2.0, 1.65, 0.80),
    wash_hi: (11500.0, 1.9, 1.55, 1.70),
    mix: (0.34, 0.66),
    lp_sweep: (18000.0, 4200.0),
    gain_trim: 0.659,
    ..CRASH_WSC
};

/// acoustic: a big struck plate — slow bloom, dense beating, very long
/// dark-tilting tail. The cymbal in the room.
const CRASH_ACOUSTIC: CrashPreset = CrashPreset {
    attack: 0.35,
    length: 0.7,
    decay_k: (0.60, 1.20),
    shimmer_k: (0.82, 0.26),
    brightness_k: (0.48, 0.55),
    swell_k: (0.0035, 0.0130),
    wash_k: (0.48, 0.26),
    velocity_tilt: 0.7,
    base_pitch: 260.0,
    mode_freq_scale: 0.38,
    mode_amp_tilt: 1.05,
    coupling: 0.20,
    wash_lo: (3400.0, 2.4, 1.40, 0.66),
    wash_hi: (8000.0, 2.2, 0.95, 2.00),
    mix: (0.50, 0.50),
    lp_sweep: (15000.0, 2400.0),
    gain_trim: 1.262,
    ..CRASH_WSC
};

/// splash: small, fast, bright — a crash with the size taken out. Instant
/// hit, high plate, short tail.
const CRASH_SPLASH: CrashPreset = CrashPreset {
    attack: 0.85,
    metal: 0.7,
    length: 0.18,
    decay_k: (3.20, 3.60),
    brightness_k: (0.72, 0.60),
    swell_k: (0.001, 0.005),
    wash_k: (0.60, 0.28),
    base_pitch: 520.0,
    mode_freq_scale: 0.78,
    mode_amp_tilt: 1.55,
    coupling: 0.08,
    wash_lo: (6200.0, 2.4, 1.55, 0.85),
    wash_hi: (12500.0, 2.2, 1.35, 1.60),
    hp_cutoff: 1200.0,
    mix: (0.40, 0.60),
    lp_sweep: (18000.0, 5000.0),
    gain_trim: 1.135,
    ..CRASH_WSC
};

/// china: trashy and violently inharmonic — the mode bank stretched far off
/// the Bessel ratios, hard coupling, an abrupt attack and a rude tail.
const CRASH_CHINA: CrashPreset = CrashPreset {
    attack: 0.9,
    metal: 0.8,
    decay_k: (1.60, 2.20),
    shimmer_k: (1.15, 0.35),
    brightness_k: (0.85, 0.55),
    swell_k: (0.0005, 0.004),
    wash_k: (0.62, 0.28),
    base_pitch: 430.0,
    mode_freq_scale: 0.91,
    mode_amp_tilt: 2.10,
    coupling: 0.30,
    wash_lo: (5800.0, 1.8, 1.60, 0.90),
    wash_hi: (12000.0, 1.6, 1.55, 1.45),
    hp_cutoff: 1000.0,
    mix: (0.48, 0.52),
    lp_sweep: (18000.0, 5500.0),
    gain_trim: 0.758,
    ..CRASH_WSC
};

/// glass: a shimmering crystalline wash — tonal, high, very long, quiet noise.
const CRASH_GLASS: CrashPreset = CrashPreset {
    metal: 0.85,
    length: 0.9,
    decay_k: (0.35, 0.70),
    shimmer_k: (0.95, 0.30),
    brightness_k: (0.60, 0.55),
    swell_k: (0.0030, 0.0110),
    wash_k: (0.14, 0.12),
    base_pitch: 560.0,
    mode_freq_scale: 0.66,
    mode_amp_tilt: 1.30,
    coupling: 0.04,
    wash_lo: (7500.0, 4.5, 0.55, 0.85),
    wash_hi: (14000.0, 3.6, 0.40, 1.35),
    hp_cutoff: 1500.0,
    mix: (0.80, 0.20),
    lp_sweep: (18000.0, 7000.0),
    gain_trim: 0.942,
    ..CRASH_WSC
};

/// doom: a cavernous gong-crash — very low plate, enormous decay, dark.
const CRASH_DOOM: CrashPreset = CrashPreset {
    metal: 0.15,
    length: 0.95,
    decay_k: (0.28, 0.55),
    shimmer_k: (0.75, 0.20),
    brightness_k: (0.18, 0.30),
    swell_k: (0.0090, 0.0280),
    wash_k: (0.50, 0.26),
    base_pitch: 150.0,
    mode_freq_scale: 0.26,
    mode_amp_tilt: 0.40,
    coupling: 0.24,
    wash_lo: (1900.0, 2.6, 1.45, 0.58),
    wash_hi: (4400.0, 2.4, 0.55, 2.10),
    hp_cutoff: 320.0,
    mix: (0.52, 0.48),
    lp_sweep: (9000.0, 1200.0),
    gain_trim: 1.787,
    ..CRASH_WSC
};

/// crush: a distorted metallic blast — hard onset, saturated bands, chaotic.
const CRASH_CRUSH: CrashPreset = CrashPreset {
    attack: 0.95,
    metal: 0.8,
    decay_k: (1.50, 2.10),
    shimmer_k: (1.12, 0.35),
    brightness_k: (0.80, 0.60),
    swell_k: (0.0003, 0.003),
    wash_k: (0.75, 0.30),
    base_pitch: 380.0,
    mode_freq_scale: 0.72,
    mode_amp_tilt: 1.95,
    coupling: 0.38,
    wash_lo: (5400.0, 1.7, 1.80, 0.92),
    wash_hi: (11000.0, 1.6, 1.70, 1.40),
    mix: (0.38, 0.62),
    lp_sweep: (18000.0, 4800.0),
    gain_trim: 0.5,
    ..CRASH_WSC
};

/// air: a bowed swell — the longest possible rise, no impact whatsoever.
const CRASH_AIR: CrashPreset = CrashPreset {
    attack: 0.0,
    length: 0.85,
    decay_k: (0.45, 0.85),
    shimmer_k: (0.80, 0.24),
    brightness_k: (0.40, 0.45),
    swell_k: (0.060, 0.140),
    wash_k: (0.70, 0.28),
    velocity_tilt: 0.8,
    base_pitch: 290.0,
    mode_freq_scale: 0.42,
    mode_amp_tilt: 0.85,
    coupling: 0.11,
    wash_lo: (3800.0, 2.0, 1.60, 0.60),
    wash_hi: (8400.0, 1.9, 1.05, 1.45),
    mix: (0.28, 0.72),
    lp_sweep: (13000.0, 2600.0),
    gain_trim: 0.588,
    ..CRASH_WSC
};

/// snap: a gated crash — the explosion with the tail sliced off.
const CRASH_SNAP: CrashPreset = CrashPreset {
    attack: 1.0,
    length: 0.04,
    decay_k: (14.0, 12.0),
    brightness_k: (0.68, 0.55),
    swell_k: (0.0003, 0.002),
    wash_k: (0.62, 0.28),
    base_pitch: 400.0,
    mode_freq_scale: 0.62,
    mode_amp_tilt: 1.45,
    coupling: 0.06,
    wash_lo: (5200.0, 2.2, 1.55, 0.92),
    wash_hi: (11000.0, 2.0, 1.35, 1.30),
    mix: (0.40, 0.60),
    lp_sweep: (18000.0, 6000.0),
    gain_trim: 1.502,
    ..CRASH_WSC
};

pub fn crash_preset(name: Option<&str>) -> &'static CrashPreset {
    match name {
        Some("909") => &CRASH_909,
        Some("acoustic") => &CRASH_ACOUSTIC,
        Some("splash") => &CRASH_SPLASH,
        Some("china") => &CRASH_CHINA,
        Some("glass") => &CRASH_GLASS,
        Some("doom") => &CRASH_DOOM,
        Some("crush") => &CRASH_CRUSH,
        Some("air") => &CRASH_AIR,
        Some("snap") => &CRASH_SNAP,
        _ => &CRASH_WSC,
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ResolvedCrash {
    pub tune: f64,
    pub base_freq: f64,
    pub decay_rate: f64,
    pub shimmer: f64,
    pub brightness: f64,
    pub swell: f64,
    pub wash: f64,
    pub velocity_tilt: f64,
    pub mode_amp: [f64; 22],
    pub internal: &'static CrashPreset,
}

pub fn resolve_crash(params: Option<&CrashParams>) -> ResolvedCrash {
    let pre = crash_preset(params.and_then(|p| p.preset.as_deref()));
    let attack = p(params.and_then(|x| x.attack), pre.attack);
    let metal = p(params.and_then(|x| x.metal), pre.metal);
    let length = p(params.and_then(|x| x.length), pre.length);
    let mut mode_amp = [0.0; 22];
    for (i, (_, base_amp, _)) in HIHAT_MODES.iter().enumerate() {
        mode_amp[i] = base_amp * pre.mode_amp_tilt.powf(i as f64 / 21.0);
    }
    ResolvedCrash {
        tune: p(params.and_then(|x| x.tune), 1.0),
        base_freq: 0.0, // filled in per note by the engine
        decay_rate: p(params.and_then(|x| x.decay_rate), pre.decay_k.0 + (1.0 - length) * pre.decay_k.1),
        shimmer: p(params.and_then(|x| x.shimmer), pre.shimmer_k.0 + metal * pre.shimmer_k.1),
        brightness: p(params.and_then(|x| x.brightness), pre.brightness_k.0 + metal * pre.brightness_k.1),
        swell: p(params.and_then(|x| x.swell), pre.swell_k.0 + (1.0 - attack) * pre.swell_k.1),
        wash: p(params.and_then(|x| x.wash), pre.wash_k.0 + metal * pre.wash_k.1),
        velocity_tilt: p(params.and_then(|x| x.velocity_tilt), pre.velocity_tilt),
        mode_amp,
        internal: pre,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SHAKER — many small particles striking a shell.
//
// A shaker is not "noise with an envelope." It is a *cloud of impacts*,
// and the ear hears the granularity directly: the density of the cloud is
// what separates a maraca (few big seeds) from a cabasa (hundreds of tiny
// beads). Two mechanisms model that here, both cheap:
//
//   1. GRAIN — the band noise is amplitude-modulated by a sample-and-hold
//      random signal at `grain_period` samples. That is the micro-texture.
//   2. BURSTS — a handful of jittered exponential sub-hits inside the one
//      stroke. That is the macro-texture (particles do not land together).
//
// `tambourine` layers two high-Q ringing bands on top: the jingles.
// ═══════════════════════════════════════════════════════════════════════

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ShakerPreset {
    // Macro centers.
    pub attack: f64,
    pub tone: f64,
    pub length: f64,
    // Formula coefficients.
    pub decay_k: (f64, f64),       // on length
    pub brightness_k: (f64, f64),  // on tone
    /// Multiplier on both band centers = k.0 + tone × k.1.
    pub band_shift_k: (f64, f64),
    // Direct param defaults.
    pub density: f64,
    pub jingle: f64,
    pub velocity_tilt: f64,
    // ── Internal-only knobs ──
    /// How much the band centers follow the played note. Shakers are
    /// unpitched — `Fm` colours them, it does not transpose them.
    pub pitch_track: f64,
    /// Particle noise bands: (center Hz, Q, gain, decay multiplier) × 2.
    pub band_lo: (f64, f64, f64, f64),
    pub band_hi: (f64, f64, f64, f64),
    /// Shell resonance under the particles: (Hz, Q, gain).
    pub shell: (f64, f64, f64),
    /// Sample-and-hold grain period in samples (at 48 kHz) and its depth.
    /// Short period = fine sand; long period = big rattling seeds.
    pub grain_period: usize,
    pub grain_depth: f64,
    /// Macro sub-hits inside one stroke: count, spacing (s), spacing
    /// jitter (fraction), and per-burst level falloff.
    pub bursts: usize,
    pub burst_spacing: f64,
    pub burst_jitter: f64,
    pub burst_falloff: f64,
    /// Onset time constant (s) = k.0 + (1 − attack) × k.1.
    pub attack_tau_k: (f64, f64),
    /// Jingle rings: two (Hz, Q) bands and the decay multiplier for both.
    pub jingle_bp: ((f64, f64), (f64, f64)),
    pub jingle_decay: f64,
    pub gain_trim: f64,
}

const SHAKER_WSC: ShakerPreset = ShakerPreset {
    attack: 0.5,
    tone: 0.5,
    length: 0.5,
    decay_k: (0.045, 0.115),
    brightness_k: (0.35, 0.60),
    band_shift_k: (0.75, 0.50),
    density: 0.55,
    jingle: 0.0,
    velocity_tilt: 0.45,
    pitch_track: 0.25,
    band_lo: (5200.0, 1.6, 1.00, 1.00),
    band_hi: (9800.0, 1.4, 0.85, 1.55),
    shell: (1500.0, 2.5, 0.22),
    grain_period: 9,
    grain_depth: 0.70,
    bursts: 3,
    burst_spacing: 0.0026,
    burst_jitter: 0.45,
    burst_falloff: 0.45,
    attack_tau_k: (0.0004, 0.0022),
    jingle_bp: ((7400.0, 26.0), (10600.0, 22.0)),
    jingle_decay: 0.30,
    gain_trim: 1.0,
};

/// tambourine: the jingles are the instrument. Two high-Q rings that carry
/// far longer than the particle cloud that excites them.
const SHAKER_TAMBOURINE: ShakerPreset = ShakerPreset {
    tone: 0.65,
    length: 0.6,
    decay_k: (0.030, 0.075),
    brightness_k: (0.50, 0.60),
    band_shift_k: (0.85, 0.50),
    density: 0.80,
    jingle: 1.0,
    band_lo: (5800.0, 1.5, 0.80, 1.10),
    band_hi: (10500.0, 1.3, 0.75, 1.60),
    shell: (2400.0, 3.0, 0.16),
    grain_period: 6,
    grain_depth: 0.85,
    bursts: 4,
    burst_spacing: 0.0022,
    burst_jitter: 0.60,
    burst_falloff: 0.50,
    attack_tau_k: (0.0002, 0.0012),
    jingle_bp: ((7100.0, 30.0), (10200.0, 26.0)),
    jingle_decay: 0.22,
    gain_trim: 0.932,
    ..SHAKER_WSC
};

/// maraca: few big seeds — coarse grain, dry, woody, short.
const SHAKER_MARACA: ShakerPreset = ShakerPreset {
    tone: 0.3,
    length: 0.35,
    decay_k: (0.030, 0.070),
    brightness_k: (0.22, 0.45),
    band_shift_k: (0.60, 0.45),
    density: 0.25,
    band_lo: (3600.0, 2.2, 1.10, 1.00),
    band_hi: (7200.0, 2.0, 0.55, 1.70),
    shell: (900.0, 2.2, 0.40),
    grain_period: 22,
    grain_depth: 0.95,
    bursts: 2,
    burst_spacing: 0.0042,
    burst_jitter: 0.50,
    burst_falloff: 0.45,
    attack_tau_k: (0.0006, 0.0030),
    gain_trim: 2.337,
    ..SHAKER_WSC
};

/// cabasa: hundreds of steel beads on a ribbed shell — the finest grain in
/// the family, bright, gritty, and slightly longer than a shaker.
const SHAKER_CABASA: ShakerPreset = ShakerPreset {
    attack: 0.75,
    tone: 0.7,
    decay_k: (0.055, 0.130),
    brightness_k: (0.55, 0.60),
    band_shift_k: (0.90, 0.50),
    density: 0.95,
    band_lo: (6400.0, 1.4, 0.95, 1.05),
    band_hi: (11500.0, 1.2, 1.00, 1.45),
    shell: (2000.0, 2.0, 0.14),
    grain_period: 4,
    grain_depth: 0.60,
    bursts: 4,
    burst_spacing: 0.0019,
    burst_jitter: 0.55,
    burst_falloff: 0.55,
    attack_tau_k: (0.0002, 0.0014),
    gain_trim: 0.669,
    ..SHAKER_WSC
};

/// 808: the machine's tick — narrowly band-limited, smooth (barely any
/// grain), very short. Closer to a filtered noise blip than a real shaker,
/// which is exactly what it was.
const SHAKER_808: ShakerPreset = ShakerPreset {
    tone: 0.55,
    length: 0.2,
    decay_k: (0.022, 0.048),
    brightness_k: (0.40, 0.50),
    band_shift_k: (0.80, 0.45),
    density: 0.05,
    band_lo: (5600.0, 3.2, 1.05, 1.05),
    band_hi: (9200.0, 3.0, 0.70, 1.50),
    shell: (1800.0, 3.5, 0.10),
    grain_period: 3,
    grain_depth: 0.10,
    bursts: 1,
    burst_spacing: 0.0040,
    burst_jitter: 0.10,
    burst_falloff: 1.0,
    attack_tau_k: (0.0002, 0.0010),
    gain_trim: 2.415,
    ..SHAKER_WSC
};

/// glass: a crystalline sprinkle — high tonal rings over a fine cloud.
const SHAKER_GLASS: ShakerPreset = ShakerPreset {
    tone: 0.9,
    length: 0.7,
    decay_k: (0.050, 0.120),
    brightness_k: (0.65, 0.55),
    band_shift_k: (1.05, 0.50),
    density: 0.7,
    jingle: 0.75,
    band_lo: (8000.0, 3.0, 0.70, 1.05),
    band_hi: (13500.0, 2.6, 0.85, 1.35),
    shell: (3600.0, 5.0, 0.18),
    grain_period: 5,
    grain_depth: 0.55,
    bursts: 3,
    burst_spacing: 0.0024,
    burst_jitter: 0.5,
    burst_falloff: 0.50,
    attack_tau_k: (0.0002, 0.0012),
    jingle_bp: ((9600.0, 34.0), (14200.0, 30.0)),
    jingle_decay: 0.18,
    gain_trim: 1.065,
    ..SHAKER_WSC
};

/// doom: a low dark rattle — bones in a box, not beads in a tube.
const SHAKER_DOOM: ShakerPreset = ShakerPreset {
    tone: 0.1,
    length: 0.75,
    decay_k: (0.070, 0.180),
    brightness_k: (0.10, 0.28),
    band_shift_k: (0.40, 0.35),
    density: 0.2,
    band_lo: (1600.0, 2.4, 1.20, 0.90),
    band_hi: (3400.0, 2.2, 0.45, 1.80),
    shell: (450.0, 2.4, 0.55),
    grain_period: 30,
    grain_depth: 1.0,
    bursts: 3,
    burst_spacing: 0.0060,
    burst_jitter: 0.55,
    burst_falloff: 0.45,
    attack_tau_k: (0.0010, 0.0040),
    gain_trim: 2.327,
    ..SHAKER_WSC
};

/// crush: a distorted industrial hiss — dense, saturated, harsh.
const SHAKER_CRUSH: ShakerPreset = ShakerPreset {
    attack: 0.9,
    tone: 0.75,
    decay_k: (0.040, 0.100),
    brightness_k: (0.70, 0.55),
    band_shift_k: (0.95, 0.50),
    density: 0.85,
    band_lo: (6000.0, 1.1, 1.35, 1.05),
    band_hi: (10800.0, 1.0, 1.30, 1.35),
    shell: (2200.0, 1.6, 0.30),
    grain_period: 7,
    grain_depth: 1.0,
    bursts: 4,
    burst_spacing: 0.0018,
    burst_jitter: 0.7,
    burst_falloff: 0.58,
    attack_tau_k: (0.0001, 0.0008),
    gain_trim: 0.457,
    ..SHAKER_WSC
};

/// air: a brushed wash — no grain, no impact, just breath.
const SHAKER_AIR: ShakerPreset = ShakerPreset {
    attack: 0.0,
    tone: 0.45,
    length: 0.8,
    decay_k: (0.090, 0.220),
    brightness_k: (0.30, 0.45),
    band_shift_k: (0.70, 0.45),
    density: 0.0,
    velocity_tilt: 0.7,
    band_lo: (4400.0, 1.2, 1.10, 0.85),
    band_hi: (8600.0, 1.1, 0.70, 1.30),
    shell: (1200.0, 1.8, 0.20),
    grain_period: 3,
    grain_depth: 0.05,
    bursts: 1,
    burst_spacing: 0.0060,
    burst_jitter: 0.0,
    burst_falloff: 1.0,
    attack_tau_k: (0.0090, 0.0250),
    gain_trim: 1.23,
    ..SHAKER_WSC
};

/// snap: a single ultra-tight tick — one burst, no tail.
const SHAKER_SNAP: ShakerPreset = ShakerPreset {
    attack: 1.0,
    tone: 0.7,
    length: 0.0,
    decay_k: (0.008, 0.020),
    brightness_k: (0.60, 0.50),
    band_shift_k: (0.95, 0.45),
    density: 0.35,
    band_lo: (6600.0, 1.8, 1.10, 1.10),
    band_hi: (11800.0, 1.6, 1.05, 1.30),
    shell: (2600.0, 2.4, 0.10),
    grain_period: 4,
    grain_depth: 0.40,
    bursts: 1,
    burst_spacing: 0.0020,
    burst_jitter: 0.0,
    burst_falloff: 1.0,
    attack_tau_k: (0.0001, 0.0006),
    gain_trim: 3.062,
    ..SHAKER_WSC
};

pub fn shaker_preset(name: Option<&str>) -> &'static ShakerPreset {
    match name {
        Some("tambourine") => &SHAKER_TAMBOURINE,
        Some("maraca") => &SHAKER_MARACA,
        Some("cabasa") => &SHAKER_CABASA,
        Some("808") => &SHAKER_808,
        Some("glass") => &SHAKER_GLASS,
        Some("doom") => &SHAKER_DOOM,
        Some("crush") => &SHAKER_CRUSH,
        Some("air") => &SHAKER_AIR,
        Some("snap") => &SHAKER_SNAP,
        _ => &SHAKER_WSC,
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ResolvedShaker {
    pub tune: f64,
    pub decay: f64,
    pub density: f64,
    pub brightness: f64,
    pub jingle: f64,
    pub velocity_tilt: f64,
    /// Multiplier applied to both band centers (tone + note tracking + tune).
    pub band_shift: f64,
    pub attack_tau: f64,
    pub internal: &'static ShakerPreset,
}

pub fn resolve_shaker(params: Option<&ShakerParams>) -> ResolvedShaker {
    let pre = shaker_preset(params.and_then(|p| p.preset.as_deref()));
    let attack = p(params.and_then(|x| x.attack), pre.attack);
    let tone = p(params.and_then(|x| x.tone), pre.tone);
    let length = p(params.and_then(|x| x.length), pre.length);
    ResolvedShaker {
        tune: p(params.and_then(|x| x.tune), 1.0),
        decay: p(params.and_then(|x| x.decay), pre.decay_k.0 + length * pre.decay_k.1),
        density: p(params.and_then(|x| x.density), pre.density),
        brightness: p(params.and_then(|x| x.brightness), pre.brightness_k.0 + tone * pre.brightness_k.1),
        jingle: p(params.and_then(|x| x.jingle), pre.jingle),
        velocity_tilt: p(params.and_then(|x| x.velocity_tilt), pre.velocity_tilt),
        band_shift: pre.band_shift_k.0 + tone * pre.band_shift_k.1,
        attack_tau: pre.attack_tau_k.0 + (1.0 - attack) * pre.attack_tau_k.1,
        internal: pre,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// COWBELL — struck metal with two strong, deliberately unrelated partials.
//
// The 808 recipe is two detuned square oscillators (~540 Hz and ~800 Hz,
// a ratio of about 1.48 — near a fifth but not one) through a bandpass
// with a fast attack and a two-stage decay. Squares would alias badly at
// these frequencies, so each partial is a soft-clipped sine instead: the
// shaper adds the odd harmonics that make it read as "square" while the
// series still dies off fast enough to stay clean.
//
// `acoustic` swaps the pure pair for four struck-metal partials, which is
// the difference between a drum machine and a real cowbell on a stand.
// ═══════════════════════════════════════════════════════════════════════

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CowbellPreset {
    // Macro centers.
    pub tone: f64,
    pub length: f64,
    // Formula coefficients.
    pub decay_k: (f64, f64),          // on length
    pub attack_amount_k: (f64, f64),  // on tone
    // Direct param defaults.
    pub ratio: f64,
    pub velocity_tilt: f64,
    // ── Internal-only knobs ──
    /// Register anchor at the canonical `f: 60` header, and note tracking.
    /// A cowbell IS pitched, but not chromatically played — `pitch_track`
    /// 0.5 lets `Fm` move it musically without it chasing a bass line.
    pub base_pitch: f64,
    pub pitch_track: f64,
    /// Partial ratios (index 1 is replaced by the `ratio` param),
    /// amplitudes, and decay multipliers.
    pub partial_ratios: [f64; 4],
    pub partial_amps: [f64; 4],
    pub partial_decays: [f64; 4],
    /// Soft-clip drive on each partial. 0 = pure sine, higher = squarer.
    pub shape_drive: f64,
    /// Body bandpass (Hz, Q) — the hollow "cup" of the bell.
    pub bp: (f64, f64),
    /// Mallet click bandpass (Hz, Q) and ring make-up gain.
    pub click_bp: (f64, f64),
    pub click_gain: f64,
    /// Fast initial decay segment layered over the main one: (level, tau s).
    pub snap: (f64, f64),
    pub drive_out: f64,
    pub gain_trim: f64,
}

const COWBELL_WSC: CowbellPreset = CowbellPreset {
    tone: 0.5,
    length: 0.5,
    decay_k: (0.15, 0.44),
    attack_amount_k: (0.30, 0.40),
    ratio: 1.48,
    velocity_tilt: 0.5,
    base_pitch: 560.0,
    pitch_track: 0.50,
    partial_ratios: [1.0, 1.48, 2.31, 3.17],
    partial_amps: [1.0, 0.85, 0.14, 0.06],
    partial_decays: [1.0, 1.15, 2.0, 2.8],
    shape_drive: 1.8,
    bp: (1250.0, 1.6),
    click_bp: (3600.0, 5.0),
    click_gain: 20.0,
    snap: (0.55, 0.012),
    drive_out: 1.0,
    gain_trim: 1.0,
};

/// 808: the iconic pair — hard square shaping, narrow band, nothing else.
const COWBELL_808: CowbellPreset = CowbellPreset {
    tone: 0.45,
    decay_k: (0.14, 0.40),
    attack_amount_k: (0.22, 0.32),
    ratio: 1.4815, // 800 / 540 — the original oscillator pair
    base_pitch: 540.0,
    partial_ratios: [1.0, 1.4815, 2.30, 3.15],
    partial_amps: [1.0, 0.95, 0.06, 0.02],
    partial_decays: [1.0, 1.0, 2.4, 3.2],
    shape_drive: 3.0,
    bp: (1100.0, 2.2),
    click_bp: (3200.0, 5.5),
    click_gain: 18.0,
    snap: (0.45, 0.010),
    gain_trim: 0.988,
    ..COWBELL_WSC
};

/// acoustic: real struck metal — four inharmonic partials, clangier, longer,
/// with an audible mallet.
const COWBELL_ACOUSTIC: CowbellPreset = CowbellPreset {
    tone: 0.6,
    length: 0.65,
    decay_k: (0.14, 0.40),
    attack_amount_k: (0.42, 0.45),
    ratio: 1.53,
    velocity_tilt: 0.7,
    base_pitch: 610.0,
    partial_ratios: [1.0, 1.53, 2.41, 3.62],
    partial_amps: [1.0, 0.72, 0.38, 0.22],
    partial_decays: [1.0, 1.3, 1.9, 2.6],
    shape_drive: 0.9,
    bp: (1600.0, 1.1),
    click_bp: (4400.0, 4.0),
    click_gain: 25.0,
    snap: (0.70, 0.016),
    gain_trim: 0.93,
    ..COWBELL_WSC
};

/// glass: a pure high bell — near-tonal partials, long clean ring, no shaping.
const COWBELL_GLASS: CowbellPreset = CowbellPreset {
    tone: 0.85,
    length: 0.9,
    decay_k: (0.22, 0.60),
    attack_amount_k: (0.25, 0.30),
    ratio: 2.0,
    base_pitch: 880.0,
    partial_ratios: [1.0, 2.0, 3.01, 4.02],
    partial_amps: [1.0, 0.55, 0.30, 0.18],
    partial_decays: [1.0, 1.15, 1.4, 1.7],
    shape_drive: 0.0,
    bp: (2600.0, 0.8),
    click_bp: (6000.0, 7.0),
    click_gain: 18.0,
    snap: (0.35, 0.008),
    gain_trim: 0.737,
    ..COWBELL_WSC
};

/// doom: a low tolling bell — deep partials, very long, dark band.
const COWBELL_DOOM: CowbellPreset = CowbellPreset {
    tone: 0.2,
    length: 0.95,
    decay_k: (0.30, 0.85),
    attack_amount_k: (0.14, 0.24),
    ratio: 1.41,
    base_pitch: 190.0,
    partial_ratios: [1.0, 1.41, 2.12, 2.94],
    partial_amps: [1.0, 0.62, 0.30, 0.16],
    partial_decays: [1.0, 1.25, 1.8, 2.4],
    shape_drive: 1.2,
    bp: (520.0, 1.2),
    click_bp: (1600.0, 4.0),
    click_gain: 16.0,
    snap: (0.40, 0.018),
    gain_trim: 0.898,
    ..COWBELL_WSC
};

/// crush: an anvil — hard drive, violently inharmonic, clipped output.
const COWBELL_CRUSH: CowbellPreset = CowbellPreset {
    tone: 0.7,
    length: 0.4,
    decay_k: (0.07, 0.22),
    attack_amount_k: (0.50, 0.45),
    ratio: 1.71,
    base_pitch: 640.0,
    partial_ratios: [1.0, 1.71, 2.63, 3.94],
    partial_amps: [1.0, 0.88, 0.55, 0.34],
    partial_decays: [1.0, 1.2, 1.6, 2.1],
    shape_drive: 5.0,
    bp: (1900.0, 0.9),
    click_bp: (4800.0, 3.5),
    click_gain: 29.0,
    snap: (0.80, 0.010),
    drive_out: 1.6,
    gain_trim: 0.519,
    ..COWBELL_WSC
};

/// air: a soft struck bell — no mallet, gentle onset, breathy body.
const COWBELL_AIR: CowbellPreset = CowbellPreset {
    tone: 0.4,
    length: 0.7,
    decay_k: (0.18, 0.45),
    attack_amount_k: (0.00, 0.08),
    ratio: 1.45,
    velocity_tilt: 0.75,
    base_pitch: 500.0,
    partial_amps: [1.0, 0.60, 0.18, 0.08],
    partial_decays: [1.0, 1.3, 2.1, 3.0],
    shape_drive: 0.3,
    bp: (1150.0, 1.0),
    click_bp: (2600.0, 3.0),
    click_gain: 10.0,
    snap: (0.18, 0.020),
    gain_trim: 1.105,
    ..COWBELL_WSC
};

/// snap: a metallic tick — the strike with the bell removed.
const COWBELL_SNAP: CowbellPreset = CowbellPreset {
    tone: 0.75,
    length: 0.03,
    decay_k: (0.016, 0.040),
    attack_amount_k: (0.55, 0.45),
    ratio: 1.62,
    base_pitch: 700.0,
    partial_amps: [1.0, 0.70, 0.22, 0.10],
    partial_decays: [1.0, 1.4, 2.2, 3.0],
    shape_drive: 2.4,
    bp: (2100.0, 1.4),
    click_bp: (4600.0, 5.0),
    click_gain: 27.0,
    snap: (0.85, 0.005),
    gain_trim: 2.355,
    ..COWBELL_WSC
};

pub fn cowbell_preset(name: Option<&str>) -> &'static CowbellPreset {
    match name {
        Some("808") => &COWBELL_808,
        Some("acoustic") => &COWBELL_ACOUSTIC,
        Some("glass") => &COWBELL_GLASS,
        Some("doom") => &COWBELL_DOOM,
        Some("crush") => &COWBELL_CRUSH,
        Some("air") => &COWBELL_AIR,
        Some("snap") => &COWBELL_SNAP,
        _ => &COWBELL_WSC,
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ResolvedCowbell {
    pub tune: f64,
    pub base_freq: f64,
    pub decay: f64,
    pub ratio: f64,
    pub attack_amount: f64,
    pub velocity_tilt: f64,
    pub internal: &'static CowbellPreset,
}

pub fn resolve_cowbell(params: Option<&CowbellParams>) -> ResolvedCowbell {
    let pre = cowbell_preset(params.and_then(|p| p.preset.as_deref()));
    let tone = p(params.and_then(|x| x.tone), pre.tone);
    let length = p(params.and_then(|x| x.length), pre.length);
    ResolvedCowbell {
        tune: p(params.and_then(|x| x.tune), 1.0),
        base_freq: 0.0, // filled in per note by the engine
        decay: p(params.and_then(|x| x.decay), pre.decay_k.0 + length * pre.decay_k.1),
        ratio: p(params.and_then(|x| x.ratio), pre.ratio),
        attack_amount: p(params.and_then(|x| x.attack_amount), pre.attack_amount_k.0 + tone * pre.attack_amount_k.1),
        velocity_tilt: p(params.and_then(|x| x.velocity_tilt), pre.velocity_tilt),
        internal: pre,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use weresocool_ast::drum_presets::{
        CLAP_PRESETS, COWBELL_PRESETS, CRASH_PRESETS, HIHAT_PRESETS, KICK_PRESETS, RIDE_PRESETS,
        RIMSHOT_PRESETS, SHAKER_PRESETS, SNARE_PRESETS, TOM_PRESETS,
    };

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
        for name in TOM_PRESETS {
            let preset = tom_preset(Some(name));
            if *name != "wsc" {
                assert_ne!(preset, &TOM_WSC, "Tom preset `{}` is not distinct", name);
            }
        }
        for name in RIDE_PRESETS {
            let preset = ride_preset(Some(name));
            if *name != "wsc" {
                assert_ne!(preset, &RIDE_WSC, "Ride preset `{}` is not distinct", name);
            }
        }
        for name in CRASH_PRESETS {
            let preset = crash_preset(Some(name));
            if *name != "wsc" {
                assert_ne!(preset, &CRASH_WSC, "Crash preset `{}` is not distinct", name);
            }
        }
        for name in SHAKER_PRESETS {
            let preset = shaker_preset(Some(name));
            if *name != "wsc" {
                assert_ne!(preset, &SHAKER_WSC, "Shaker preset `{}` is not distinct", name);
            }
        }
        for name in COWBELL_PRESETS {
            let preset = cowbell_preset(Some(name));
            if *name != "wsc" {
                assert_ne!(preset, &COWBELL_WSC, "Cowbell preset `{}` is not distinct", name);
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
