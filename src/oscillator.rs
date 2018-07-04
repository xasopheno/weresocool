use ring_buffer::RingBuffer;
use sine::Generator;

pub struct Oscillator {
    pub f_buffer: RingBuffer<f32>,
    pub l_ratios: Vec<R>,
    pub l_phases: Vec<f32>,
    pub r_ratios: Vec<R>,
    pub r_phases: Vec<f32>,
    pub generator: Generator,
    pub gain: Gain,
}

#[derive(Debug)]
pub struct R {
    pub decimal: f32,
    pub offset: f32,
    pub ratio: String,
    pub gain: f32,
}

impl R {
    pub fn atio(n: usize, d: usize, offset: f32, gain: f32) -> R {
        R {
            decimal: n as f32 / d as f32,
            offset,
            ratio: [n.to_string(), d.to_string()].join("/"),
            gain,
        }
    }
}

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
    pub fn new(f_buffer_size: usize, l_ratios: Vec<R>, r_ratios: Vec<R>) -> Oscillator {
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
        }
    }

    pub fn update(&mut self, frequency: f32, gain: f32, probability: f32) {
        let mut new_freq = if frequency < 2500.0 { frequency } else { 0.0 };
        let mut new_gain = if new_freq != 0.0 { gain } else { 0.0 };
        let current_frequency = self.f_buffer.current();

        if probability < 0.2 && frequency != 0.0 {
            new_freq = current_frequency;
        };

//        if (frequency - current_frequency).abs() > frequency * 0.8
//            && frequency != 0.0
//            && current_frequency != 0.0 {
//                new_freq = current_frequency;
//        }

//                println!("{}, {}", frequency, probability);

        self.f_buffer.push(new_freq);
        self.gain.update(new_gain);
//                self.f_buffer.push(220.0);
//                self.gain.update(1.0);
    }

    pub fn generate(&mut self, buffer_size: usize, sample_rate: f32) -> (Vec<f32>, Vec<f32>) {
//                println!("{:?}", self.f_buffer);
        let mut frequency = self.f_buffer.current();
        if self.f_buffer.previous() as f32 != 0.0 && self.f_buffer.current() == 0.0 {
            frequency = self.f_buffer.previous();
        }

        let (mut l_waveform, l_new_phases, normalization) = (self.generator.generate)(
            frequency as f32,
            &self.gain,
            &self.l_ratios,
            &self.l_phases,
            buffer_size as usize,
            sample_rate,
        );

        let (mut r_waveform, r_new_phases, normalization) = (self.generator.generate)(
            frequency as f32,
            &self.gain,
            &self.r_ratios,
            &self.r_phases,
            buffer_size as usize,
            sample_rate,
        );

        self.gain.past *= normalization;
        self.l_phases = l_new_phases;
        self.r_phases = r_new_phases;
        (l_waveform, r_waveform)
    }
}

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
