use itertools::Itertools;

pub fn encode_lr_channels_to_ogg_vorbis(l: Vec<f32>, r: Vec<f32>) -> Vec<u8> {
    if &l.len() != &r.len() {
        panic!("Error encoding audio to ogg vorbis. Channels are not the same length");
    }

    let interleaved: Vec<f32> = interleave_channels(l, r);
    let veci16 = pcm_f32_to_i16(interleaved);

    let mut encoder =
        vorbis::Encoder::new(2, 44100, vorbis::VorbisQuality::VeryHighQuality).unwrap();
    let mut encoded = encoder.encode(&veci16).unwrap();
    encoded.append(&mut encoder.flush().unwrap());
    encoded
}

pub fn interleave_channels(l: Vec<f32>, r: Vec<f32>) -> Vec<f32> {
    l.iter().interleave(&r).map(|v| *v).collect()
}

fn pcm_f32_to_i16(vecf32: Vec<f32>) -> Vec<i16> {
    vecf32
        .iter()
        .map(|v| {
            let mut f = v * 32768.0;
            if f > 32767.0 {
                f = 32767.0
            };
            if f < -32768.0 {
                f = -32768.0
            };
            let i = f as i16;
            i
        })
        .collect()
}
