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

    /// Sum a vec of StereoWaveform to a single stereo_waveform.
    pub fn sum_many(mut vec_wav: Vec<Self>) -> StereoWaveform {
        // Sort the vectors by length
        StereoWaveform::sort_vecs(&mut vec_wav);

        // Get the length of the longest vector
        let max_len = vec_wav[0].l_buffer.len();

        let mut result = StereoWaveform::new(max_len);

        for wav in vec_wav {
            StereoWaveform::sum_vec(&mut result.l_buffer, &wav.l_buffer[..]);
            StereoWaveform::sum_vec(&mut result.r_buffer, &wav.r_buffer[..])
        }

        result
    }

    /// Sort a vec of StereoWaveform by length. Assumes both channels have the same
    /// buffer length
    fn sort_vecs(vec_wav: &mut [StereoWaveform]) {
        vec_wav.sort_unstable_by(|a, b| b.l_buffer.len().cmp(&a.l_buffer.len()));
    }

    /// Sum two vectors. Assumes vector a is longer than or of the same length
    /// as vector b.
    pub fn sum_vec(a: &mut [f64], b: &[f64]) {
        for (ai, bi) in a.iter_mut().zip(b) {
            *ai += *bi;
        }
    }
}

impl Normalize for StereoWaveform {
    fn normalize(&mut self) {
        let mut max = std::f64::MIN;
        for sample in self.l_buffer.iter() {
            if (*sample).abs() > max {
                max = *sample;
            }
        }

        for sample in self.r_buffer.iter() {
            if (*sample).abs() > max {
                max = *sample;
            }
        }

        let mut normalization_ratio = 1.0 / max;
        if normalization_ratio > 1.0 {
            normalization_ratio = 1.0
        }

        println!("Normalized by {:?}", normalization_ratio);

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
