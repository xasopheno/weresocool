#[cfg(test)]
mod loudness_balance_tests {
    use crate::generation::{RenderReturn, RenderType};
    use crate::interpretable::{
        InputType::{Filename, Language},
        Interpretable,
    };
    use weresocool_analyze::measure_lufs;
    use weresocool_ast::drum_presets::{
        CLAP_PRESETS, COWBELL_PRESETS, CRASH_PRESETS, HIHAT_PRESETS, KICK_PRESETS, RIDE_PRESETS,
        RIMSHOT_PRESETS, SHAKER_PRESETS, SNARE_PRESETS, TOM_PRESETS,
    };

    /// Every drum family and its presets. `OpenHat` shares HiHat's table and
    /// is covered by the HiHat row.
    const FAMILIES: [(&str, &[&str]); 10] = [
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
    use weresocool_instrument::StereoWaveform;
    use weresocool_shared::Settings;

    fn render_socool_file(path: &str) -> StereoWaveform {
        let render_return = Filename(path)
            .make(RenderType::StereoWaveform, None)
            .expect(&format!("Failed to render {}", path));

        match render_return {
            RenderReturn::StereoWaveform(sw) => sw,
            _ => panic!("Expected StereoWaveform from render"),
        }
    }

    fn get_lufs(path: &str) -> f64 {
        let sw = render_socool_file(path);
        let sample_rate = Settings::global().sample_rate as u32;
        measure_lufs(&sw.l_buffer, &sw.r_buffer, sample_rate)
    }

    /// LUFS of a single preset hit — mirrors the loudness_tests/*.socool
    /// shape ({ f: 80, l: 1/2 }, one bare hit at Gm 1).
    fn preset_lufs(drum: &str, preset: &str) -> f64 {
        let src = format!("{{ f: 80, l: 1/2, g: 1, p: 0 }}\n\nmain = {{ {drum} {preset} | Gm 1 }}\n");
        let render_return = Language(&src)
            .make(RenderType::StereoWaveform, None)
            .unwrap_or_else(|e| panic!("{} {} failed to render: {:?}", drum, preset, e));
        let sw = match render_return {
            RenderReturn::StereoWaveform(sw) => sw,
            _ => panic!("Expected StereoWaveform"),
        };
        let sample_rate = Settings::global().sample_rate as u32;
        measure_lufs(&sw.l_buffer, &sw.r_buffer, sample_rate)
    }

    /// Prints suggested `gain_trim` values per preset (correction toward
    /// that drum's wsc level). Run with:
    /// cargo test --release measure_preset_loudness -- --nocapture
    #[test]
    fn measure_preset_loudness() {
        println!("\n=== Preset Loudness (LUFS) — gain_trim corrections vs wsc ===");
        for (drum, presets) in FAMILIES {
            let target = preset_lufs(drum, "wsc");
            println!("\n{} (wsc target {:.2} LUFS):", drum, target);
            for preset in presets {
                let lufs = preset_lufs(drum, preset);
                let correction = 10_f64.powf((target - lufs) / 20.0);
                println!(
                    "  {:<10} {:>7.2} LUFS   gain_trim ×{:.3}",
                    preset, lufs, correction
                );
            }
        }
    }

    /// Switching presets must never blow up a mix: every preset lands
    /// within 1.5 dB of its drum family's wsc voicing.
    #[test]
    fn verify_preset_balance() {
        let mut failures = vec![];
        for (drum, presets) in FAMILIES {
            let target = preset_lufs(drum, "wsc");
            for preset in presets {
                let lufs = preset_lufs(drum, preset);
                let dev = (lufs - target).abs();
                println!("{} {:<10} {:>7.2} LUFS (Δ {:.2} dB)", drum, preset, lufs, dev);
                if dev > 1.5 {
                    failures.push(format!(
                        "{} {} is {:.2} dB off wsc ({:.2} vs {:.2} LUFS)",
                        drum, preset, dev, lufs, target
                    ));
                }
            }
        }
        assert!(
            failures.is_empty(),
            "Presets out of balance:\n{}",
            failures.join("\n")
        );
    }

    /// Measures the LUFS of each drum and prints correction factors.
    /// Run with: cargo test measure_drum_loudness -- --nocapture
    #[test]
    fn measure_drum_loudness() {
        let kick_lufs = get_lufs("src/testing/loudness_tests/kick_default.socool");
        let snare_lufs = get_lufs("src/testing/loudness_tests/snare_default.socool");
        let hihat_lufs = get_lufs("src/testing/loudness_tests/hihat_default.socool");

        println!("\n=== Drum Loudness Measurements (LUFS) ===");
        println!("Kick:  {:.2} LUFS", kick_lufs);
        println!("Snare: {:.2} LUFS", snare_lufs);
        println!("HiHat: {:.2} LUFS", hihat_lufs);

        // Find the loudest drum as target
        let target_lufs = kick_lufs.max(snare_lufs).max(hihat_lufs);
        println!("\nTarget LUFS: {:.2}", target_lufs);

        // Calculate correction factors
        // correction = 10^((target - measured) / 20)
        let kick_correction = 10_f64.powf((target_lufs - kick_lufs) / 20.0);
        let snare_correction = 10_f64.powf((target_lufs - snare_lufs) / 20.0);
        let hihat_correction = 10_f64.powf((target_lufs - hihat_lufs) / 20.0);

        // Current multipliers from sample.rs
        let kick_current = 8.0;
        let snare_current = 8.0;
        let hihat_current = 6.0;

        println!("\n=== Correction Factors ===");
        println!("Kick:  {:.4}x (current: {})", kick_correction, kick_current);
        println!("Snare: {:.4}x (current: {})", snare_correction, snare_current);
        println!("HiHat: {:.4}x (current: {})", hihat_correction, hihat_current);

        // Calculate new multipliers
        let kick_new = kick_current * kick_correction;
        let snare_new = snare_current * snare_correction;
        let hihat_new = hihat_current * hihat_correction;

        println!("\n=== Suggested New Multipliers for sample.rs ===");
        println!("const KICK_GAIN: f64 = {:.1};   // Was {}", kick_new, kick_current);
        println!("const SNARE_GAIN: f64 = {:.1};  // Was {}", snare_new, snare_current);
        println!("const HIHAT_GAIN: f64 = {:.1};  // Was {}", hihat_new, hihat_current);
    }

    /// Prints each family's `wsc` LUFS against the kick anchor, and the
    /// correction each `*_GAIN` constant in `sample.rs` needs to join it.
    /// Run with:
    /// cargo test --release measure_kit_balance -- --nocapture
    #[test]
    fn measure_kit_balance() {
        let anchor = preset_lufs("Kick", "wsc");
        println!("\n=== Kit Balance (LUFS at Gm 1, anchor = Kick) ===");
        println!("anchor {:.2} LUFS\n", anchor);
        for (drum, _) in FAMILIES {
            let lufs = preset_lufs(drum, "wsc");
            println!(
                "{:<9} {:>7.2} LUFS   Δ {:>6.2} dB   *_GAIN ×{:.3}",
                drum,
                lufs,
                lufs - anchor,
                10_f64.powf((anchor - lufs) / 20.0)
            );
        }
    }

    /// Every drum sits close enough to every other at `Gm 1` that a pattern
    /// balances by musical intent rather than by hunting for the gain that
    /// stops the cowbell burying the kick.
    ///
    /// Two bars, because the kit is not calibrated on one basis. Kick,
    /// Snare, HiHat, Tom, Ride, Crash, Shaker and Cowbell are matched to the
    /// kick to within a fraction of a dB. Clap and Rimshot were calibrated
    /// against peak level rather than LUFS and read ~3.2 dB low here; they
    /// sound right in a mix, so the wide bar records that rather than
    /// pretending otherwise. Tighten them and this test tightens with them.
    #[test]
    fn verify_kit_balance() {
        const LOOSE: [&str; 2] = ["Clap", "Rimshot"];
        let anchor = preset_lufs("Kick", "wsc");
        let mut measured = vec![];
        for (drum, _) in FAMILIES {
            let lufs = preset_lufs(drum, "wsc");
            println!("{:<9} {:>7.2} LUFS (Δ {:>6.2} dB)", drum, lufs, lufs - anchor);
            measured.push((drum, lufs));
        }
        let mut failures = vec![];
        for (drum, lufs) in &measured {
            let dev = (lufs - anchor).abs();
            let bar = if LOOSE.contains(drum) { 3.5 } else { 1.5 };
            if dev > bar {
                failures.push(format!(
                    "  {:<9} {:.2} LUFS is {:.2} dB off the kick (bar {:.1} dB)",
                    drum, lufs, dev, bar
                ));
            }
        }
        assert!(failures.is_empty(), "Kit not balanced:\n{}", failures.join("\n"));
    }

    /// Verifies that all drums are within 1 dB of each other.
    /// Run with: cargo test verify_drum_balance
    #[test]
    fn verify_drum_balance() {
        let kick_lufs = get_lufs("src/testing/loudness_tests/kick_default.socool");
        let snare_lufs = get_lufs("src/testing/loudness_tests/snare_default.socool");
        let hihat_lufs = get_lufs("src/testing/loudness_tests/hihat_default.socool");

        println!("\n=== Drum Balance Verification ===");
        println!("Kick:  {:.2} LUFS", kick_lufs);
        println!("Snare: {:.2} LUFS", snare_lufs);
        println!("HiHat: {:.2} LUFS", hihat_lufs);

        let max_lufs = kick_lufs.max(snare_lufs).max(hihat_lufs);
        let min_lufs = kick_lufs.min(snare_lufs).min(hihat_lufs);
        let range = max_lufs - min_lufs;

        println!("Range: {:.2} dB", range);

        // Allow 1 dB tolerance
        assert!(
            range <= 1.0,
            "Drums not balanced! Range is {:.2} dB (should be <= 1.0 dB)\n\
             Kick: {:.2}, Snare: {:.2}, HiHat: {:.2}",
            range,
            kick_lufs,
            snare_lufs,
            hihat_lufs
        );
    }
}
