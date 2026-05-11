#[cfg(test)]
mod loudness_balance_tests {
    use crate::generation::{RenderReturn, RenderType};
    use crate::interpretable::{InputType::Filename, Interpretable};
    use weresocool_analyze::measure_lufs;
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
