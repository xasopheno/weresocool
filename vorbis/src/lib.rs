use itertools::Itertools;
use weresocool_error::Error;
use weresocool_shared::Settings;

pub fn encode_lr_channels_to_ogg_vorbis(l: Vec<f64>, r: Vec<f64>) -> Result<Vec<u8>, Error> {
    if l.len() != r.len() {
        return Err(Error::with_msg(format!(
            "Channel length mismatch: left={}, right={}",
            l.len(),
            r.len()
        )));
    }

    let interleaved: Vec<f64> = interleave_channels(l, r);
    let veci16 = pcm_f64_to_i16(interleaved);

    let mut encoder = vorbis_encoder::Encoder::new(2, Settings::global().sample_rate as u64, 1.0)
        .map_err(|e| Error::with_msg(format!("Failed to create OGG encoder: {}", e)))?;
    let mut encoded = encoder
        .encode(&veci16)
        .map_err(|e| Error::with_msg(format!("Failed to encode audio to OGG: {}", e)))?;
    encoded.append(
        &mut encoder
            .flush()
            .map_err(|e| Error::with_msg(format!("Failed to flush OGG encoder: {}", e)))?,
    );
    Ok(encoded)
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
