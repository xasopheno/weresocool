use ring_buffer::RingBuffer;
use settings::Settings;
use sine::Generator;
use ratios::{R};

pub struct Oscillator {
    pub f_buffer: RingBuffer<f32>,
    pub l_ratios: Vec<R>,
    pub l_phases: Vec<f32>,
    pub r_ratios: Vec<R>,
    pub r_phases: Vec<f32>,
    pub generator: Generator,
    pub gain: Gain,
    pub settings: Settings,
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

    pub fn update(&mut self, new_gain: f32) -> () {
        self.past = self.current;
        self.current = new_gain;
    }
}

impl Oscillator {
    pub fn new(
        f_buffer_size: usize,
        l_ratios: Vec<R>,
        r_ratios: Vec<R>,
        settings: Settings,
    ) -> Oscillator {
        println!("{}", "Left Generated Ratios");
        for r in l_ratios.iter() {
            println!("   - {} offset: {}", r.ratio, r.offset);
        }

        println!("{}", "Right Generated Ratios");
        for r in r_ratios.iter() {
            println!("   - {} offset: {}", r.ratio, r.offset);
        }

        Oscillator {
            f_buffer: RingBuffer::<f32>::new_full(f_buffer_size as usize),
            l_phases: vec![0.0; l_ratios.len()],
            l_ratios,
            r_phases: vec![0.0; r_ratios.len()],
            r_ratios,
            generator: Generator::new(),
            gain: Gain::new(0.0, 0.0),
            settings,
        }
    }

    pub fn update(&mut self, frequency: f32, gain: f32, _probability: f32) {
        let new_freq =
            if frequency < self.settings.max_freq && frequency > self.settings.min_freq {
                frequency
            } else {
                0.0
            };
        let mut new_gain = if new_freq != 0.0 { gain } else { 0.0 };

        if new_gain < self.settings.gain_threshold_min {
            new_gain = 0.0
        };

//      println!("{}, {}", new_freq, new_gain);

        self.f_buffer.push(new_freq);
        self.gain.update(new_gain);
//                self.f_buffer.push(220.0);
//                self.gain.update(1.0);
    }


    pub fn generate(&mut self) -> (Vec<f32>, Vec<f32>) {
//           println!("{:?}", self.f_buffer);
        let current_frequency = self.f_buffer.current();
        let previous_frequency = self.f_buffer.previous();
        println!("{:?}, {:?}", previous_frequency, current_frequency, );
        if current_frequency == 0.0 && previous_frequency == 0.0 {
            return silence(self.settings.buffer_size);
        }

        let mut frequency = current_frequency;

        if previous_frequency != 0.0 && current_frequency == 0.0 {
            frequency = previous_frequency;
        }
        println!("{:?}", self.gain);

        let (l_waveform, l_new_phases, _loudness) = (self.generator.generate)(
            frequency,
            &self.gain,
            &self.l_ratios,
            &self.l_phases,
            &self.settings,
        );

        let (r_waveform, r_new_phases, loudness) = (self.generator.generate)(
            frequency,
            &self.gain,
            &self.r_ratios,
            &self.r_phases,
            &self.settings,
        );

        self.gain.current *= loudness;
        self.l_phases = l_new_phases;
        self.r_phases = r_new_phases;
        (l_waveform, r_waveform)
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
