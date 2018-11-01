#[derive(Clone, Debug, PartialEq)]
pub struct StereoWaveform {
    pub l_buffer: Vec<f32>,
    pub r_buffer: Vec<f32>,
}

pub trait Normalize {
    fn normalize(&mut self);
}

impl StereoWaveform {
    pub fn new(buffer_size: usize) -> StereoWaveform {
        StereoWaveform {
            l_buffer: vec![0.0; buffer_size],
            r_buffer: vec![0.0; buffer_size],
        }
    }

    pub fn append(&mut self, mut stereo_waveform: StereoWaveform) {
        self.l_buffer.append(&mut stereo_waveform.l_buffer);
        self.r_buffer.append(&mut stereo_waveform.r_buffer);
    }

    pub fn get_buffer(&mut self, index: usize, buffer_size: usize) -> StereoWaveform {
        if (index + 1) * buffer_size < self.l_buffer.len() {
            let l_buffer = &self.l_buffer[index * buffer_size..(index + 1) * buffer_size];
            let r_buffer = &self.r_buffer[index * buffer_size..(index + 1) * buffer_size];
            StereoWaveform {
                l_buffer: l_buffer.to_vec(),
                r_buffer: r_buffer.to_vec(),
            }
        } else {
            StereoWaveform::new(2048)
        }
    }
}

impl Normalize for StereoWaveform {
    fn normalize(&mut self) {
        let mut max = 0.0;
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

        let normalization_ratio = 1.0 / max * 0.85;

        for sample in self.l_buffer.iter_mut() {
            *sample *= normalization_ratio
        }

        for sample in self.r_buffer.iter_mut() {
            *sample *= normalization_ratio
        }
//        println!("Normalized by {}", normalization_ratio);
    }
}
