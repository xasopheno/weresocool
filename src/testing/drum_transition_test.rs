//! Regression tests for drum note → silence transitions.
//!
//! Drum envelopes are functions of time-since-note-on, but ops restart their
//! sample clock at 0. Before the persistent drum note clock, a drum followed
//! by a silence op (`Seq [bd, Fm 0, bd]`) replayed its entire attack inside
//! the "silence" — a loud ghost hit fading over the ~11 ms gain smoothing —
//! while a spurious self-crossfade double-ran every stateful drum filter.
//! These tests render that exact shape and assert the silence region contains
//! only a smoothly decaying tail.

#[cfg(test)]
mod drum_transition_tests {
    use crate::generation::{RenderReturn, RenderType};
    use crate::interpretable::{InputType::Language, Interpretable};
    use weresocool_shared::Settings;

    const TRANSITION_COMPOSITION: &str = "
        { f: 55, l: 1/2, g: 1, p: 0 }

        bd = {
            Kick {
                pitch_decay: 0.015,
                pitch_range: 2.2,
                amp_decay: 0.55,
                hump: 2.85,
                ks_mix: 0.95
            }
        }

        main = {
            Seq [bd, Fm 0, bd, Silence 1, bd]
        }
    ";

    fn render_mono(language: &str) -> (Vec<f64>, f64) {
        let render_return = Language(language)
            .make(RenderType::StereoWaveform, None)
            .expect("Failed to render test composition");

        let sw = match render_return {
            RenderReturn::StereoWaveform(sw) => sw,
            _ => panic!("Expected StereoWaveform from render"),
        };

        let mono: Vec<f64> = sw
            .l_buffer
            .iter()
            .zip(sw.r_buffer.iter())
            .map(|(l, r)| (l + r) / 2.0)
            .collect();
        (mono, Settings::global().sample_rate)
    }

    /// The silence after a drum hit must contain only the hit's decaying
    /// tail: no retriggered attack (peak spike) and no discontinuities
    /// (sample-to-sample jumps).
    #[test]
    fn drum_tail_rings_through_silence_without_retrigger() {
        let (mono, sample_rate) = render_mono(TRANSITION_COMPOSITION);

        // Note grid: bd 0–0.5s, Fm 0 0.5–1.0s, bd 1.0–1.5s,
        // Silence 1.5–2.0s, bd 2.0–2.5s.
        for (start, end, label) in [(0.5, 1.0, "Fm 0"), (1.5, 2.0, "Silence")] {
            let a = (start * sample_rate) as usize;
            let b = (end * sample_rate) as usize;
            let region = &mono[a..b];

            // Tail level just before the boundary — the silence region may
            // never exceed it. A retriggered attack was ~10x louder.
            let pre_window = &mono[a.saturating_sub((0.01 * sample_rate) as usize)..a];
            let pre_peak = pre_window.iter().fold(0.0_f64, |m, s| m.max(s.abs()));
            let region_peak = region.iter().fold(0.0_f64, |m, s| m.max(s.abs()));
            assert!(
                region_peak <= pre_peak * 1.5 + 1e-6,
                "Drum retriggered inside {label} region: \
                 peak {region_peak:.4} vs pre-boundary tail {pre_peak:.4}"
            );

            // No clicks: max sample-to-sample jump stays tiny. The old
            // retrigger produced jumps > 0.02 at the boundary.
            let max_jump = region
                .windows(2)
                .map(|w| (w[1] - w[0]).abs())
                .fold(0.0_f64, f64::max);
            assert!(
                max_jump < 0.005,
                "Click in {label} region: max sample-to-sample jump {max_jump:.5}"
            );
        }
    }

    /// Note-ons out of silence must keep their instant transient — the
    /// fade machinery added for tails must not soften real hits.
    #[test]
    fn drum_note_on_keeps_transient_after_silence() {
        let (mono, sample_rate) = render_mono(TRANSITION_COMPOSITION);

        for note_start in [1.0, 2.0] {
            let a = (note_start * sample_rate) as usize;
            let attack = &mono[a..a + (0.01 * sample_rate) as usize];
            let attack_peak = attack.iter().fold(0.0_f64, |m, s| m.max(s.abs()));
            assert!(
                attack_peak > 0.05,
                "Drum transient buried at {note_start}s: \
                 first 10 ms peak only {attack_peak:.4}"
            );
        }
    }
}

#[cfg(test)]
mod drum_choke_tests {
    use crate::generation::{RenderReturn, RenderType};
    use crate::interpretable::{InputType::Language, Interpretable};
    use weresocool_shared::Settings;

    /// A note-on that cuts a still-ringing drum must not produce a
    /// one-sample step (audible pop). The soft choke decays the cut
    /// tail over ~3 ms instead.
    #[test]
    fn choked_drums_do_not_pop() {
        let src = "
            { f: 55, l: 1/6, g: 1, p: 0 }
            bd = { Kick wsc }
            sn = { Snare wsc | Fm 3 }
            main = { Seq [bd, bd, bd, sn | Lm 1/4 | Repeat 4] }
        ";
        let render_return = Language(src)
            .make(RenderType::StereoWaveform, None)
            .expect("render failed");
        let sw = match render_return {
            RenderReturn::StereoWaveform(sw) => sw,
            _ => panic!("expected StereoWaveform"),
        };
        let mono: Vec<f64> = sw
            .l_buffer
            .iter()
            .zip(sw.r_buffer.iter())
            .map(|(l, r)| (l + r) / 2.0)
            .collect();
        let sr = Settings::global().sample_rate;
        // Kick-on-kick boundaries at 1/6 s; rolls retrigger inside slot 4.
        for k in 1..3 {
            let b = (k as f64 / 6.0 * sr) as usize;
            let jump = (mono[b] - mono[b - 1]).abs();
            assert!(
                jump < 0.02,
                "kick choke pop at boundary {}: one-sample jump {:.4}",
                k,
                jump
            );
        }
    }
}
