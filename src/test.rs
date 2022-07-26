#[cfg(test)]
mod test {
    use assert_cmd::Command;

    // #[test]
    // fn it_plays_a_cool_file() {
    // let mut cmd = Command::new("cargo");

    // cmd.arg("run")
    // .arg("--release")
    // .arg("--")
    // .arg("play")
    // .arg("test_data/play.socool")
    // .assert()
    // .success();
    // }

    #[test]
    fn it_prints_a_csv() {
        let mut cmd = Command::new("cargo");

        cmd.arg("run")
            .arg("--release")
            .arg("--")
            .arg("print")
            .arg("src/test_data/play.socool")
            .arg("--output_dir")
            .arg("/tmp")
            .arg("--csv")
            .assert()
            .success();

        let expected_filename = "./src/test_data/play.socool.csv";
        let written_filename = "/tmp/play.socool.csv";
        assert_same_file_contents(expected_filename, written_filename)
    }

    // #[test]
    // fn it_prints_a_json() {
    // let mut cmd = Command::new("cargo");

    // cmd.arg("run")
    // .arg("--release")
    // .arg("--")
    // .arg("print")
    // .arg("test_data/play.socool")
    // .arg("--output_dir")
    // .arg("/tmp")
    // .arg("--json")
    // .assert()
    // .success();

    // let expected_filename = "test_data/play.socool.json";
    // let written_filename = "/tmp/play.socool.json";
    // assert_same_file_contents(expected_filename, written_filename)
    // }

    // #[test]
    // fn it_prints_a_wav() {
    // let mut cmd = Command::new("cargo");

    // cmd.arg("run")
    // .arg("--release")
    // .arg("--")
    // .arg("print")
    // .arg("test_data/play.socool")
    // .arg("--wav")
    // .arg("--output_dir")
    // .arg("/tmp")
    // .assert()
    // .success();

    // let expected_filename = "test_data/play.wav";
    // let written_filename = "/tmp/play.wav";
    // assert_same_wav_file(expected_filename, written_filename)
    // .expect("Wave files are no the same");
    // }

    // #[test]
    // fn it_prints_an_mp3() {
    // let mut cmd = Command::new("cargo");

    // cmd.arg("run")
    // .arg("--release")
    // .arg("--")
    // .arg("print")
    // .arg("test_data/play.socool")
    // .arg("--output_dir")
    // .arg("/tmp")
    // .arg("--mp3")
    // .assert()
    // .success();

    // let expected_filename = "test_data/play.mp3";
    // let written_filename = "/tmp/play.mp3";
    // assert_same_bytes(expected_filename, written_filename);
    // }

    // #[test]
    // fn it_prints_stems() {
    // let mut cmd = Command::new("cargo");

    // cmd.arg("run")
    // .arg("--release")
    // .arg("--")
    // .arg("print")
    // .arg("test_data/play.socool")
    // .arg("--output_dir")
    // .arg("/tmp")
    // .arg("--stems")
    // .assert()
    // .success();

    // let expected_filename = "test_data/play.socool.stems.zip";
    // let written_filename = "/tmp/play.socool.stems.zip";
    // assert_same_zip_contents(expected_filename, written_filename).unwrap();
    // }

    // fn assert_same_wav_file(
    // expected_filename: &str,
    // written_filename: &str,
    // ) -> Result<(), hound::Error> {
    // let mut expected_reader = hound::WavReader::open(expected_filename)
    // .expect("Something went wrong reading the file");
    // let mut written_reader = hound::WavReader::open(written_filename)
    // .expect("Something went wrong reading the file");

    // for (written_sample, expected_sample) in expected_reader
    // .samples::<f32>()
    // .zip(written_reader.samples::<f32>())
    // {
    // assert!(written_sample? == expected_sample?);
    // }

    // Ok(())
    // }

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
            assert_same_bytes(
                format!("/tmp/written_zip/{}", written_filename).as_str(),
                format!("/tmp/expected_zip/{}", expected_filename).as_str(),
            );
        }

        Ok(())
    }

    fn assert_same_bytes(expected_filename: &str, written_filename: &str) {
        let written_read =
            std::fs::read(written_filename).expect("Something went wrong reading file");
        let expected_read =
            std::fs::read(expected_filename).expect("Something went wrong reading file");

        assert!(written_read == expected_read);
    }

    fn assert_same_file_contents(expected_filename: &str, written_filename: &str) {
        let expected = std::fs::read_to_string(expected_filename)
            .expect("Something went wrong reading the file");
        let written = std::fs::read_to_string(written_filename)
            .expect("Something went wrong reading the file");
        dbg!(&expected);
        dbg!(&written);

        assert!(expected == written);
    }
}
