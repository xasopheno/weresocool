use serde::{Deserialize, Serialize};
use std::cmp;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// Left and Right audio channels
pub struct StereoWaveform {
    pub l_buffer: Vec<f64>,
    pub r_buffer: Vec<f64>,
}

pub trait Normalize {
    fn normalize(&mut self);
}

pub fn make_fade_vec(buffer_size: usize) -> Vec<f64> {
    (0..buffer_size)
        .rev()
        .map(|s| s as f64 / buffer_size as f64)
        .collect()
}

impl StereoWaveform {
    pub const fn new_empty() -> Self {
        Self {
            l_buffer: vec![],
            r_buffer: vec![],
        }
    }

    pub fn new(buffer_size: usize) -> Self {
        Self {
            l_buffer: vec![0.0; buffer_size],
            r_buffer: vec![0.0; buffer_size],
        }
    }

    pub fn new_with_buffer(buffer: Vec<f64>) -> Self {
        Self {
            l_buffer: buffer.clone(),
            r_buffer: buffer,
        }
    }

    pub fn fade_out(&mut self) {
        let fade_vec = make_fade_vec(self.max_len());
        for (i, value) in fade_vec.iter().enumerate() {
            self.l_buffer[i] *= value;
            self.r_buffer[i] *= value;
        }
    }

    pub fn pad(&mut self, buffersize: usize) {
        self.l_buffer.resize(buffersize, 0.0);
        self.r_buffer.resize(buffersize, 0.0);
    }

    pub fn max_len(&self) -> usize {
        cmp::max(self.l_buffer.len(), self.r_buffer.len())
    }

    pub fn total_len(&self) -> usize {
        self.l_buffer.len() + self.r_buffer.len()
    }

    pub fn append(&mut self, mut stereo_waveform: Self) {
        self.l_buffer.append(&mut stereo_waveform.l_buffer);
        self.r_buffer.append(&mut stereo_waveform.r_buffer);
    }

    /// This assumes that all buffers are the same size
    pub fn get_buffer(&mut self, index: usize, buffer_size: usize) -> Option<Self> {
        if (index + 1) * buffer_size < self.l_buffer.len() {
            let l_buffer = &self.l_buffer[index * buffer_size..(index + 1) * buffer_size];
            let r_buffer = &self.r_buffer[index * buffer_size..(index + 1) * buffer_size];
            Some(Self {
                l_buffer: l_buffer.to_vec(),
                r_buffer: r_buffer.to_vec(),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod normalize_tests {
    use super::*;

    #[test]
    fn normalize_scales_negative_dominant_peak() {
        // Dominant peak is negative (-1.4) and exceeds full scale; positive max
        // is only 0.7. The old code stored the signed sample and missed the
        // negative peak, leaving -1.4 to clip on playback. Correct behavior:
        // scale so the peak magnitude lands at 1.0.
        let mut sw = StereoWaveform {
            l_buffer: vec![0.7, -1.4, 0.3],
            r_buffer: vec![0.0, -0.5, 0.2],
        };
        sw.normalize();
        let peak = sw
            .l_buffer
            .iter()
            .chain(sw.r_buffer.iter())
            .fold(0.0_f64, |m, s| m.max(s.abs()));
        assert!((peak - 1.0).abs() < 1e-9, "peak should be 1.0, got {peak}");
        // polarity preserved (no sign inversion from a negative ratio)
        assert!(sw.l_buffer[1] < 0.0);
    }

    #[test]
    fn normalize_leaves_subunity_untouched() {
        let mut sw = StereoWaveform {
            l_buffer: vec![0.5, -0.3],
            r_buffer: vec![0.2, -0.1],
        };
        sw.normalize();
        assert_eq!(sw.l_buffer, vec![0.5, -0.3]);
    }
}

impl Normalize for StereoWaveform {
    fn normalize(&mut self) {
        // Track peak *magnitude*. Storing the signed sample here (the previous
        // bug) ignored negative-dominant peaks: a waveform peaking at -1.4 with
        // a +0.7 positive max would compute ratio = 1/0.7 > 1, clamp to 1.0, and
        // leave the -1.4 sample to clip. Asymmetric peaks are common once a
        // reconstruction is phase-coherent, so this surfaced as distortion.
        let mut max = 0.0_f64;
        for sample in self.l_buffer.iter() {
            if (*sample).abs() > max {
                max = (*sample).abs();
            }
        }

        for sample in self.r_buffer.iter() {
            if (*sample).abs() > max {
                max = (*sample).abs();
            }
        }

        let mut normalization_ratio = 1.0 / max;
        if normalization_ratio > 1.0 {
            normalization_ratio = 1.0
        }

        if normalization_ratio < 1.0 {
            for sample in self.l_buffer.iter_mut() {
                *sample *= normalization_ratio
            }

            for sample in self.r_buffer.iter_mut() {
                *sample *= normalization_ratio
            }
        }
    }
}
