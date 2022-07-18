use itertools::Itertools;

fn main() {
    let file = std::fs::File::open("test.wav").unwrap();
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

    let interleaved = l.iter().interleave(r.as_slice());

    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create("out.wav", spec).unwrap();
    for t in interleaved {
        writer.write_sample(*t).unwrap();
    }
    writer.finalize().unwrap();

    // https://docs.rs/vorbis-encoder/0.1.4/src/vorbis_encoder/lib.rs.html#18-20
    println!("Hello, world!");
}
