use itertools::Itertools;

pub fn encode_lr_channels_to_ogg_vorbis(l: Vec<f64>, r: Vec<f64>) -> Vec<u8> {
    if l.len() != r.len() {
        panic!("Error encoding audio to ogg vorbis. Channels are not the same length");
    }

    let interleaved: Vec<f64> = interleave_channels(l, r);
    let veci16 = pcm_f64_to_i16(interleaved);

    let mut encoder = vorbis_encoder::Encoder::new(2, 44100, 1.0).unwrap();
    let mut encoded = encoder.encode(&veci16).unwrap();
    encoded.append(&mut encoder.flush().unwrap());
    encoded
}

pub fn interleave_channels(l: Vec<f64>, r: Vec<f64>) -> Vec<f64> {
    l.iter().interleave(&r).copied().collect()
}

fn pcm_f64_to_i16(vecf64: Vec<f64>) -> Vec<i16> {
    vecf64
        .iter()
        .map(|v| {
            let mut f = v * 32768.0;
            if f > 32767.0 {
                f = 32767.0
            };
            if f < -32768.0 {
                f = -32768.0
            };
            f as i16
        })
        .collect()
}
