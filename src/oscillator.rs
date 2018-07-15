use ratios::{StereoRatios, R};
use ring_buffer::RingBuffer;
use settings::Settings;
use sine::Generator;

pub struct Oscillator {
    pub f_buffer: RingBuffer<f32>,
    pub stereo_ratios: StereoRatios,
    pub stereo_phases: StereoPhases,
    pub stereo_spectral_history: StereoSpectralHistory,
    pub generator: Generator,
    pub gain: Gain,
    pub settings: Settings,
}

#[derive(Debug, Clone)]
pub struct StereoSpectralHistory {
    pub l_frequencies: SpectralHistory,
    pub r_frequencies: SpectralHistory,
}

impl StereoSpectralHistory {
    pub fn new(num_l_frequencies: usize, num_r_frequencies: usize) -> StereoSpectralHistory {
        StereoSpectralHistory {
            l_frequencies: SpectralHistory::new(num_l_frequencies),
            r_frequencies: SpectralHistory::new(num_r_frequencies),
        }
    }
    pub fn update(&mut self, l_frequencies: SpectralHistory, r_frequencies: SpectralHistory) {
        self.l_frequencies = l_frequencies;
        self.r_frequencies = r_frequencies;
    }
}

#[derive(Debug, Clone)]
pub struct SpectralHistory {
    pub past_frequencies: Vec<f32>,
    pub current_frequencies: Vec<f32>,
}

impl SpectralHistory {
    pub fn new(len: usize) -> SpectralHistory {
        SpectralHistory {
            past_frequencies: vec![0.0; len],
            current_frequencies: vec![0.0; len],
        }
    }
}

pub struct StereoPhases {
    pub l_phases: Vec<f32>,
    pub r_phases: Vec<f32>,
}

impl StereoPhases {
    pub fn new(l_ratios_len: usize, r_ratios_len: usize) -> StereoPhases {
        StereoPhases {
            l_phases: vec![0.0; l_ratios_len],
            r_phases: vec![0.0; r_ratios_len],
        }
    }
    pub fn update(&mut self, l_phases: Vec<f32>, r_phases: Vec<f32>) {
        self.l_phases = l_phases;
        self.r_phases = r_phases;
    }
}

pub struct StereoWaveform {
    pub l_waveform: Vec<f32>,
    pub r_waveform: Vec<f32>,
}

#[derive(Debug)]
pub struct Gain {
    pub past: f32,
    pub current: f32,
}

impl Gain {
    pub fn new(past: f32, current: f32) -> Gain {
        Gain { past, current }
    }

    pub fn update(&mut self, mut new_gain: f32) -> () {
        self.past = self.current;
        if (self.current - new_gain).abs() > 0.5 {
            new_gain = new_gain * 0.51;
        }

        self.current = new_gain;
    }
}

impl Oscillator {
    pub fn new(
        f_buffer_size: usize,
        stereo_ratios: StereoRatios,
        settings: Settings,
    ) -> Oscillator {
        println!("{}", "Left Generated Ratios");
        for r in stereo_ratios.l_ratios.iter() {
            println!("   - {} offset: {}", r.ratio, r.offset);
        }

        println!("{}", "Right Generated Ratios");
        for r in stereo_ratios.r_ratios.iter() {
            println!("   - {} offset: {}", r.ratio, r.offset);
        }

        Oscillator {
            f_buffer: RingBuffer::<f32>::new_full(f_buffer_size as usize),
            stereo_phases: StereoPhases::new(
                stereo_ratios.l_ratios.len(),
                stereo_ratios.r_ratios.len(),
            ),
            stereo_spectral_history: StereoSpectralHistory::new(
                stereo_ratios.l_ratios.len(),
                stereo_ratios.r_ratios.len(),
            ),
            stereo_ratios,
            generator: Generator::new(),
            gain: Gain::new(0.0, 0.0),
            settings,
        }
    }

    pub fn update(&mut self, frequency: f32, gain: f32, _probability: f32) {
        let new_freq = if frequency < self.settings.max_freq && frequency > self.settings.min_freq {
            frequency
        } else {
            0.0
        };

        let mut new_gain = if new_freq != 0.0 { gain } else { 0.0 };

        if new_gain < self.settings.gain_threshold_min {
            new_gain = 0.0
        };

        self.f_buffer.push(new_freq);
        self.gain.update(new_gain);
    }

    fn f_buffer_to_ratios(&mut self) {
        let base_frequency = self.f_buffer.current();

        for freq in self.f_buffer.to_vec() {
            let mut value = freq / base_frequency;
            if value.is_infinite() || value.is_nan() || value == 0.0 {
                value = 1.0;
            }

            println!("{}", value);
        }
    }

    pub fn generate(&mut self) -> StereoWaveform {
        println!("{:?}", self.f_buffer.to_vec());
        //        self.f_buffer_to_ratios();
        let current_frequency = self.f_buffer.current();
        let previous_frequency = self.f_buffer.previous();

        let mut frequency = current_frequency;

        if current_frequency == 0.0 && previous_frequency != 0.0 {
            frequency = previous_frequency
        }

        let (l_waveform, l_new_phases, _loudness) = (self.generator.generate)(
            frequency,
            &self.gain,
            &self.stereo_ratios.l_ratios,
            &self.stereo_phases.l_phases,
            &self.settings,
        );

        let (r_waveform, r_new_phases, loudness) = (self.generator.generate)(
            frequency,
            &self.gain,
            &self.stereo_ratios.r_ratios,
            &self.stereo_phases.r_phases,
            &self.settings,
        );

        self.gain.current *= loudness;

        self.stereo_phases.update(l_new_phases, r_new_phases);

        StereoWaveform {
            l_waveform,
            r_waveform,
        }
    }
    pub fn update_spectral_history(&mut self, current_base_frequency: f32) {
        let mut calculated_l_frequencies: Vec<f32> = self
            .stereo_ratios
            .l_ratios
            .iter()
            .map(|ratio| ratio.decimal * current_base_frequency)
            .collect();
        let mut calculated_r_frequencies: Vec<f32> = self
            .stereo_ratios
            .l_ratios
            .iter()
            .map(|ratio| ratio.decimal * current_base_frequency)
            .collect();

        for (current, past) in calculated_l_frequencies.iter_mut().zip(
            self.stereo_spectral_history.clone()
                .l_frequencies
                .current_frequencies
                .iter_mut(),
        ) {
            if (*current - *past).abs() > 40.0 {
                *current *= 3.0/2.0;
            }
        }

        for (current, past) in calculated_r_frequencies.iter_mut().zip(
            self.stereo_spectral_history.clone()
                .r_frequencies
                .current_frequencies
                .iter_mut(),
        ) {
            if (*current - *past).abs() > 40.0 {
                *current *= 3.0/2.0;
            }
        }
        let l_frequencies = SpectralHistory {
          past_frequencies: self.stereo_spectral_history.l_frequencies.current_frequencies.clone(),
          current_frequencies: calculated_l_frequencies
        };

        let r_frequencies = SpectralHistory {
            past_frequencies: self.stereo_spectral_history.r_frequencies.current_frequencies.clone(),
            current_frequencies: calculated_r_frequencies
        };

        self.stereo_spectral_history.update(l_frequencies, r_frequencies);
    }
}

fn silence(buffer_size: usize) -> (Vec<f32>, Vec<f32>) {
    (vec![0.0; buffer_size], vec![0.0; buffer_size])
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn test_ratio() {
        let r: R = R::atio(3, 2, 0.0, 1.0);
        let result = r.ratio;
        let expected = "3/2";
        assert_eq!(result, expected);
        let result = r.decimal;
        let expected = 1.5;
        assert_eq!(result, expected);
    }
}
