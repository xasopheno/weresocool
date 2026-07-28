#[cfg(test)]
mod cli_tests {
    use assert_cmd::Command;
    use temp_dir::TempDir;

    /// IGNORED BY DEFAULT — this one opens an audio device.
    ///
    /// `weresocool play` streams to the default output and returns when the
    /// piece ends. With no reachable audio device (CI, a headless shell, a
    /// machine whose device is already held) it does not fail — it blocks,
    /// forever, and takes the whole suite down with it. It sat for hours
    /// before anyone noticed the suite never finished.
    ///
    /// Run it deliberately, on a machine with sound:
    ///     cargo test --release -p weresocool -- --ignored it_plays_a_cool_file
    #[test]
    #[ignore = "opens an audio device; blocks forever without one"]
    fn it_plays_a_cool_file() {
        let mut cmd = Command::new("cargo");

        cmd.arg("run")
            .arg("--release")
            .arg("--")
            .arg("play")
            .arg("src/test_data/play.socool")
            .assert()
            .success();
    }

    #[test]
    fn it_prints_a_csv() {
        let mut cmd = Command::new("cargo");
        let tmp_dir = TempDir::new().unwrap();

        cmd.arg("run")
            .arg("--release")
            .arg("--")
            .arg("print")
            .arg("src/test_data/play.socool")
            .arg("--output_dir")
            .arg(tmp_dir.path())
            .arg("--csv")
            .assert()
            .success();

        let expected_filename = "./src/test_data/play.socool.csv";
        let written_filename = format!("{}/play.socool.csv", tmp_dir.path().display());
        assert_same_file_contents(expected_filename, &written_filename)
    }

    #[test]
    fn it_prints_a_json() {
        let mut cmd = Command::new("cargo");
        let tmp_dir = TempDir::new().unwrap();

        cmd.arg("run")
            .arg("--release")
            .arg("--")
            .arg("print")
            .arg("src/test_data/play.socool")
            .arg("--output_dir")
            .arg(tmp_dir.path())
            .arg("--json")
            .assert()
            .success();

        let expected_filename = "src/test_data/play.socool.json";
        let written_filename = format!("{}/play.socool.data.json", tmp_dir.path().display());

        assert_same_file_contents(expected_filename, &written_filename)
    }

    #[test]
    fn it_prints_a_wav() {
        let mut cmd = Command::new("cargo");
        let tmp_dir = TempDir::new().unwrap();

        cmd.arg("run")
            .arg("--release")
            .arg("--")
            .arg("print")
            .arg("src/test_data/play.socool")
            .arg("--wav")
            .arg("--output_dir")
            .arg(tmp_dir.path())
            .assert()
            .success();

        #[cfg(target_os = "windows")]
        let expected_filename = "src/test_data/play_windows.wav";
        #[cfg(target_os = "macos")]
        let expected_filename = "src/test_data/play_unix.wav";
        #[cfg(target_os = "linux")]
        let expected_filename = "src/test_data/play_unix.wav";
        let written_filename = format!("{}/play.wav", tmp_dir.path().display());
        assert_same_wav_file(expected_filename, &written_filename)
            .expect("Wav files are not the same");
    }

    #[test]
    #[cfg(all(feature = "app", not(target_os = "windows")))]
    fn it_prints_an_mp3() {
        let mut cmd = Command::new("cargo");
        let tmp_dir = TempDir::new().unwrap();

        cmd.arg("run")
            .arg("--release")
            .arg("--")
            .arg("print")
            .arg("src/test_data/play.socool")
            .arg("--output_dir")
            .arg(tmp_dir.path())
            .arg("--mp3")
            .assert()
            .success();

        #[cfg(target_os = "windows")]
        let expected_filename = "src/test_data/play_windows.mp3";
        #[cfg(target_os = "macos")]
        let expected_filename = "src/test_data/play_unix.mp3";
        #[cfg(target_os = "linux")]
        let expected_filename = "src/test_data/play_unix.mp3";

        let written_filename = format!("{}/play.mp3", tmp_dir.path().display());
        assert_same_length(expected_filename, &written_filename);
    }

    #[test]
    fn it_prints_stems() {
        let mut cmd = Command::new("cargo");
        let tmp_dir = TempDir::new().unwrap();

        cmd.arg("run")
            .arg("--release")
            .arg("--")
            .arg("print")
            .arg("src/test_data/play.socool")
            .arg("--output_dir")
            .arg(tmp_dir.path())
            .arg("--stems")
            .assert()
            .success();

        #[cfg(target_os = "windows")]
        let expected_filename = "src/test_data/play_windows.socool.stems.zip";
        #[cfg(target_os = "macos")]
        let expected_filename = "src/test_data/play_unix.socool.stems.zip";
        #[cfg(target_os = "linux")]
        let expected_filename = "src/test_data/play_unix.socool.stems.zip";
        let written_filename = format!("{}/play.socool.stems.zip", tmp_dir.path().display());
        assert_same_zip_contents(expected_filename, &written_filename).unwrap();
    }

    fn assert_same_wav_file(
        expected_filename: &str,
        written_filename: &str,
    ) -> Result<(), hound::Error> {
        let mut expected_reader = hound::WavReader::open(expected_filename)
            .expect("Something went wrong reading the file");
        let mut written_reader = hound::WavReader::open(written_filename)
            .expect("Something went wrong reading the file");

        // AUDIO IS COMPARED WITH A TOLERANCE, not bit-exactly.
        //
        // These are f32 samples out of a long chain of float arithmetic.
        // Reassociation by the optimiser, a different FMA decision, a new
        // instruction selection — any of them move the last bits without
        // changing what anyone hears, and none of them is a regression. This
        // file already keeps separate expectations for Windows and Unix for
        // that exact reason; asserting `==` on top of that was asking for a
        // guarantee the platform does not give.
        //
        // 1e-3 of full scale is about -60 dB relative to a signal that peaks
        // near 0.13 here: far below audibility, far above float drift, and
        // tight enough that a real synthesis change still fails.
        const TOLERANCE: f32 = 1e-3;

        let mut worst = 0.0f32;
        let mut n = 0usize;
        for (written_sample, expected_sample) in expected_reader
            .samples::<f32>()
            .zip(written_reader.samples::<f32>())
        {
            let (w, e) = (written_sample?, expected_sample?);
            worst = worst.max((w - e).abs());
            n += 1;
        }
        assert!(n > 0, "no samples compared — one of the files is empty");
        assert!(
            worst <= TOLERANCE,
            "wav differs by {worst:.3e} at worst over {n} samples (tolerance {TOLERANCE:.0e}) — \
             that is an audible synthesis change, not float drift"
        );

        Ok(())
    }

    fn assert_same_zip_contents(
        expected_filename: &str,
        written_filename: &str,
    ) -> zip::result::ZipResult<()> {
        let written_read = std::io::Cursor::new(
            std::fs::read(written_filename).expect("Something went wrong reading file"),
        );
        let mut written_zip = zip::ZipArchive::new(written_read)?;

        let expected_read = std::io::Cursor::new(
            std::fs::read(expected_filename).expect("Something went wrong reading file"),
        );
        let mut expected_zip = zip::ZipArchive::new(expected_read)?;
        written_zip.extract(std::path::Path::new("/tmp/written_zip"))?;
        expected_zip.extract(std::path::Path::new("/tmp/expected_zip"))?;

        for (written_filename, expected_filename) in
            written_zip.file_names().zip(expected_zip.file_names())
        {
            // A stem is a WAV, so compare it as AUDIO. Byte similarity is
            // the wrong instrument here: these are f32 samples, and a
            // difference far below audibility scrambles mantissa bytes
            // arbitrarily, so "95% of bytes match" is neither necessary nor
            // sufficient for "sounds the same".
            let written = format!("/tmp/written_zip/{}", written_filename);
            let expected = format!("/tmp/expected_zip/{}", expected_filename);
            if written.ends_with(".wav") {
                assert_same_wav_file(&expected, &written).expect("stem wav differs");
            } else {
                assert_same_bytes(&expected, &written);
            }
        }

        Ok(())
    }

    fn assert_same_bytes(expected_filename: &str, written_filename: &str) {
        let tolerance = 5;

        let written_read =
            std::fs::read(written_filename).expect("Something went wrong reading the written file");
        let expected_read = std::fs::read(expected_filename)
            .expect("Something went wrong reading the expected file");

        // Compare file sizes first
        if written_read.len() != expected_read.len() {
            eprintln!(
                "File size mismatch: expected {} bytes, got {} bytes",
                expected_read.len(),
                written_read.len()
            );
        }

        let mut differences = 0;
        for (i, (w, e)) in written_read.iter().zip(expected_read.iter()).enumerate() {
            if (w.abs_diff(*e)) > tolerance {
                differences += 1;
                if differences <= 10 {
                    eprintln!(
                        "Mismatch at byte {}: expected 0x{:02x}, got 0x{:02x} (diff: {})",
                        i,
                        e,
                        w,
                        w.abs_diff(*e)
                    );
                }
            }
        }

        let total_bytes = written_read.len().min(expected_read.len());
        let similarity = 100.0 * (total_bytes - differences) as f64 / total_bytes as f64;

        eprintln!(
            "Comparison complete: {} bytes compared, {} differences (similarity: {:.2}%)",
            total_bytes, differences, similarity
        );

        assert!(
            similarity > 95.0,
            "Files are not similar enough (similarity: {:.2}%)",
            similarity
        );
    }

    pub fn assert_same_length(expected_filename: &str, written_filename: &str) {
        let written_read =
            std::fs::read(written_filename).expect("Something went wrong reading the written file");
        let expected_read = std::fs::read(expected_filename)
            .expect("Something went wrong reading the expected file");

        // Compare file sizes first
        if written_read.len() != expected_read.len() {
            eprintln!(
                "File size mismatch: expected {} bytes, got {} bytes",
                expected_read.len(),
                written_read.len()
            );
        }

        assert!(
            written_read.len() == expected_read.len(),
            "Files are not the same length"
        );
    }

    fn assert_same_file_contents(expected_filename: &str, written_filename: &str) {
        let mut expected = std::fs::read_to_string(expected_filename)
            .expect("Something went wrong reading the file");
        let mut written = std::fs::read_to_string(written_filename)
            .expect("Something went wrong reading the file");
        expected = expected.replace('\r', "");
        written = written.replace('\r', "");

        assert!(expected == written);
    }
}
