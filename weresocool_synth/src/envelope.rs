//! THE GAIN ENVELOPE — a pure function of position within the note.
//!
//! Nothing in this file may read a buffer length. That is the whole point:
//! the envelope used to be evaluated once per audio buffer with the index at
//! the buffer's END, which made it a staircase sampled at the block rate and,
//! on the unchunked path where the buffer IS the note, a single sample of
//! itself. It is now evaluated per sample at `op.sample_index() + i`, the
//! same note-scoped clock the drum oscillators have always used.
//!
//! WHY THERE IS A GATE. A textbook ADSR needs a note-off, and this language
//! does not have one — a note's length is known before it starts. So the
//! fifth term is the note-off: `gate` is how much of the note the key is
//! held. Everything after `gate + release` is silence, and that silence is
//! the articulation. It is the only way to say "short note" without inventing
//! events to be short in.
//!
//! WHY THE ATTACK STARTS WHERE THE VOICE IS. `at()` takes `start`, the gain
//! this voice was actually emitting when the note began, and ramps from
//! there. A note after a rest starts at zero and gets a true attack; a note
//! joined to its neighbour starts at the neighbour's gain and glides. This is
//! how a monophonic instrument behaves under legato, and it is also exactly
//! the envelope this language had before `Env` existed — which is why
//! identity parameters reproduce every existing piece.

use crate::gain::gain_at_index;

/// The envelope in samples and levels, already fitted to the note.
///
/// Build it with [`Envelope::new`], which applies the fit rules once, rather
/// than constructing the fields directly — the invariants (`attack + decay <=
/// gate`, `gate + release <= total`) are what make `at()` a total function.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Envelope {
    attack: usize,
    decay: usize,
    sustain: f64,
    release: usize,
    gate: usize,
    total: usize,
}

impl Envelope {
    /// `attack`, `decay` and `release` arrive in SAMPLES (the l-basis units
    /// from the score, already multiplied by `basis.l` and the sample rate).
    /// `gate` arrives as a FRACTION of the note. `total` is the note.
    pub fn new(
        attack: f64,
        decay: f64,
        sustain: f64,
        release: f64,
        gate: f64,
        total: usize,
    ) -> Self {
        let sustain = sustain.clamp(0.0, 1.0);
        let gate = ((total as f64) * gate.clamp(0.0, 1.0)).round() as usize;
        let gate = gate.min(total);

        let attack = attack.max(0.0) as usize;
        // A decay with nothing to decay TO consumes no time. Without this an
        // untouched envelope would spend a segment ramping from the peak to
        // the peak, and that segment would eat into the attack under the fit
        // rule below for no audible reason.
        let decay = if sustain >= 1.0 { 0 } else { decay.max(0.0) as usize };

        // The release has to fit in what the gate left behind. `gate == total`
        // — an unarticulated note — leaves nothing, so the note simply runs
        // into the next one. That is legato, and it is the default.
        let release = (release.max(0.0) as usize).min(total.saturating_sub(gate));

        // Everything that sounds has to fit inside the gate. Scaling both
        // rather than truncating one keeps the envelope's SHAPE when a note is
        // too short for it — the same thing `is_short` used to do, minus the
        // cliff at exactly two seconds.
        let (attack, decay) = if attack + decay > gate && attack + decay > 0 {
            let scale = gate as f64 / (attack + decay) as f64;
            (
                (attack as f64 * scale).round() as usize,
                (decay as f64 * scale).round() as usize,
            )
        } else {
            (attack, decay)
        };

        Self { attack, decay, sustain, release, gate, total }
    }

    /// The gain at sample `i` of the note, ramping from `start` (what the
    /// voice was emitting when the note began) toward `peak` (this note's
    /// target gain).
    pub fn at(&self, i: usize, start: f64, peak: f64) -> f64 {
        let sustained = peak * self.sustain;

        if i < self.attack {
            gain_at_index(start, peak, i, self.attack)
        } else if i < self.attack + self.decay {
            gain_at_index(peak, sustained, i - self.attack, self.decay)
        } else if i < self.gate {
            sustained
        } else if i < self.gate + self.release {
            gain_at_index(sustained, 0.0, i - self.gate, self.release)
        } else {
            // Only reachable once the gate has actually shut early — an
            // unarticulated note has `gate == total` and never gets here.
            0.0
        }
    }
}

#[cfg(test)]
mod test {
    use super::Envelope;

    const SR: f64 = 44100.0;

    /// What every note gets when the piece says nothing: one l-unit of
    /// attack, no decay, full sustain, gate wide open.
    fn identity(total: usize) -> Envelope {
        Envelope::new(SR, SR, 1.0, SR, 1.0, total)
    }

    #[test]
    fn identity_on_a_short_note_is_a_ramp_across_the_whole_note() {
        // The note is shorter than the attack, so the fit scales the attack
        // down to the note — a full-length swell, which is what this language
        // did before `Env` existed.
        let total = 7350; // 1/6 s
        let e = identity(total);
        assert_eq!(e.at(0, 0.0, 1.0), 0.0);
        assert!((e.at(total / 2, 0.0, 1.0) - 0.5).abs() < 0.001);
        assert!(e.at(total - 1, 0.0, 1.0) > 0.999);
    }

    #[test]
    fn identity_starts_where_the_voice_already_is() {
        let e = identity(7350);
        // Joined to a note that was sounding at 0.8: a glide, not a restrike.
        assert!((e.at(0, 0.8, 1.0) - 0.8).abs() < 1e-9);
    }

    #[test]
    fn identity_on_a_long_note_attacks_then_holds() {
        let total = 3 * SR as usize;
        let e = identity(total);
        assert!((e.at(SR as usize / 2, 0.0, 1.0) - 0.5).abs() < 0.001);
        assert!((e.at(2 * SR as usize, 0.0, 1.0) - 1.0).abs() < 1e-9);
        assert!((e.at(total - 1, 0.0, 1.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn a_gate_leaves_silence_and_that_is_the_articulation() {
        let total = 6000;
        // attack 1/3 of the note, release 1/3, gate 1/3 — lulupea's idiom.
        let third = total as f64 / 3.0;
        let e = Envelope::new(third, 0.0, 1.0, third, 1.0 / 3.0, total);
        assert!(e.at(0, 0.0, 1.0) < 1e-9);
        assert!((e.at(1999, 0.0, 1.0) - 1.0).abs() < 0.001); // peak at the gate
        assert!((e.at(3000, 0.0, 1.0) - 0.5).abs() < 0.01); // mid-release
        assert_eq!(e.at(4001, 0.0, 1.0), 0.0); // and then nothing
        assert_eq!(e.at(total - 1, 0.0, 1.0), 0.0);
    }

    #[test]
    fn the_release_cannot_outlive_the_note() {
        let total = 1000;
        // A release far longer than what the gate left: clamped, not wrapped
        // into the next note (a voice renders one op at a time).
        let e = Envelope::new(0.0, 0.0, 1.0, 100_000.0, 0.5, total);
        assert!(e.at(999, 0.0, 1.0) >= 0.0);
        assert!(e.at(500, 0.0, 1.0) > 0.0);
    }

    #[test]
    fn sustain_is_a_level_not_a_time() {
        let total = 10_000;
        let e = Envelope::new(1000.0, 1000.0, 0.5, 0.0, 1.0, total);
        assert!((e.at(999, 0.0, 1.0) - 1.0).abs() < 0.01); // peak at attack end
        assert!((e.at(2000, 0.0, 1.0) - 0.5).abs() < 0.01); // fallen to sustain
        assert!((e.at(9999, 0.0, 1.0) - 0.5).abs() < 1e-9); // and holds there
    }

    #[test]
    fn a_shut_gate_makes_no_sound() {
        let e = Envelope::new(SR, SR, 1.0, 0.0, 0.0, 5000);
        for i in [0usize, 1, 2500, 4999] {
            assert_eq!(e.at(i, 0.0, 1.0), 0.0);
        }
    }
}
