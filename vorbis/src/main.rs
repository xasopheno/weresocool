use std::io::prelude::*;
use weresocool_vorbis::encode_lr_channels_to_ogg_vorbis;

fn main() {
    println!("Hello, Vorbis");

    let (l, r) = wav_filename_to_lr_channels("test.wav");
    let encoded = encode_lr_channels_to_ogg_vorbis(l, r);

    let mut file = std::fs::File::create("result.ogg").unwrap();
    file.write_all(&encoded).unwrap();
    println!("Finished");
}

fn wav_filename_to_lr_channels(filename: &str) -> (Vec<f32>, Vec<f32>) {
    let file = std::fs::File::open(filename).unwrap();
    let mut reader = hound::WavReader::new(file).unwrap();
    let mut count = 0;

    let (l, r): (Vec<f32>, Vec<f32>) = reader
        .samples::<f32>()
        .map(|v| v.unwrap())
        .collect::<Vec<f32>>()
        .iter()
        .partition(|_v| {
            let result = count % 2 == 0;
            count += 1;
            result
        });

    (l, r)
}

#[cfg(test)]
mod test {
    use super::*;
    use hamcrest2::prelude::*;
    use weresocool_vorbis::*;

    #[test]
    fn test_interleaving() {
        let (l, r) = wav_filename_to_lr_channels("test.wav");
        let interleaved: Vec<f32> = interleave_channels(l, r);

        let test_file = std::fs::File::open("test.wav").unwrap();
        let mut test_reader = hound::WavReader::new(test_file).unwrap();
        assert_that!(
            &test_reader
                .samples::<f32>()
                .map(|v| v.unwrap())
                .collect::<Vec<f32>>(),
            contains(interleaved).exactly()
        );
    }
}
