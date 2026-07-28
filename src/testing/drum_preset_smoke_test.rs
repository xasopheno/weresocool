//! Smoke + distinctness tests for the built-in drum presets.
//!
//! Every preset of every drum must render cleanly (no NaN/inf, non-silent,
//! sane peak) through the note→silence→note shape that historically broke
//! drum rendering. And every non-`wsc` preset must produce audibly
//! DIFFERENT output from `wsc` — a preset that doesn't change the sound is
//! a wiring bug (e.g. a name the synth tables don't know silently falling
//! back to the default).

#[cfg(test)]
mod drum_preset_smoke_tests {
    use crate::generation::{RenderReturn, RenderType};
    use crate::interpretable::{InputType::Language, Interpretable};
    use weresocool_ast::drum_presets::{
        CLAP_PRESETS, COWBELL_PRESETS, CRASH_PRESETS, HIHAT_PRESETS, KICK_PRESETS, RIDE_PRESETS,
        RIMSHOT_PRESETS, SHAKER_PRESETS, SNARE_PRESETS, TOM_PRESETS,
    };

    fn render_mono(drum: &str, preset: &str) -> Vec<f64> {
        let src = format!(
            "{{ f: 220, l: 1/2, g: 1, p: 0 }}\n\nmain = {{ Seq [{drum} {preset}, Fm 0, {drum} {preset}] }}\n"
        );
        let render_return = Language(&src)
            .make(RenderType::StereoWaveform, None)
            .unwrap_or_else(|e| panic!("{} {} failed to render: {:?}", drum, preset, e));
        let sw = match render_return {
            RenderReturn::StereoWaveform(sw) => sw,
            _ => panic!("Expected StereoWaveform"),
        };
        sw.l_buffer
            .iter()
            .zip(sw.r_buffer.iter())
            .map(|(l, r)| (l + r) / 2.0)
            .collect()
    }

    fn rms(buf: &[f64]) -> f64 {
        (buf.iter().map(|s| s * s).sum::<f64>() / buf.len() as f64).sqrt()
    }

    #[test]
    fn every_preset_renders_clean() {
        let drums: [(&str, &[&str]); 11] = [
            ("Kick", KICK_PRESETS),
            ("Snare", SNARE_PRESETS),
            ("HiHat", HIHAT_PRESETS),
            ("OpenHat", HIHAT_PRESETS),
            ("Clap", CLAP_PRESETS),
            ("Rimshot", RIMSHOT_PRESETS),
            ("Tom", TOM_PRESETS),
            ("Ride", RIDE_PRESETS),
            ("Crash", CRASH_PRESETS),
            ("Shaker", SHAKER_PRESETS),
            ("Cowbell", COWBELL_PRESETS),
        ];
        for (drum, presets) in drums {
            for preset in presets {
                let mono = render_mono(drum, preset);
                let peak = mono.iter().fold(0.0_f64, |m, s| m.max(s.abs()));
                assert!(
                    mono.iter().all(|s| s.is_finite()),
                    "{} {}: non-finite samples",
                    drum,
                    preset
                );
                assert!(peak > 0.01, "{} {}: silent (peak {})", drum, preset, peak);
                assert!(peak < 1.0, "{} {}: clipping (peak {})", drum, preset, peak);
            }
        }
    }

    /// A preset must actually change the sound. Compare each preset's
    /// render against wsc's via normalized sample-wise difference.
    #[test]
    fn every_preset_is_audibly_distinct_from_wsc() {
        let drums: [(&str, &[&str]); 10] = [
            ("Kick", KICK_PRESETS),
            ("Snare", SNARE_PRESETS),
            ("HiHat", HIHAT_PRESETS),
            ("Clap", CLAP_PRESETS),
            ("Rimshot", RIMSHOT_PRESETS),
            ("Tom", TOM_PRESETS),
            ("Ride", RIDE_PRESETS),
            ("Crash", CRASH_PRESETS),
            ("Shaker", SHAKER_PRESETS),
            ("Cowbell", COWBELL_PRESETS),
        ];
        for (drum, presets) in drums {
            let base = render_mono(drum, "wsc");
            let base_rms = rms(&base);
            for preset in presets.iter().filter(|p| **p != "wsc") {
                let other = render_mono(drum, preset);
                let n = base.len().min(other.len());
                let diff_rms = rms(&base[..n]
                    .iter()
                    .zip(&other[..n])
                    .map(|(a, b)| a - b)
                    .collect::<Vec<f64>>());
                assert!(
                    diff_rms > base_rms * 0.05,
                    "{} {} is not audibly distinct from wsc (diff rms {:.6} vs base rms {:.6})",
                    drum,
                    preset,
                    diff_rms,
                    base_rms
                );
            }
        }
    }
}
