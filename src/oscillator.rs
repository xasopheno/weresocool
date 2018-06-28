use ring_buffer::RingBuffer;
use sine::Generator;

pub struct Oscillator {
    pub f_buffer: RingBuffer<f32>,
    pub ratios: Vec<R>,
    pub phases: Vec<f32>,
    pub generator: Generator,
    pub gain: Gain,
}

#[derive(Debug)]
pub struct R {
    pub decimal: f32,
    pub ratio: String,
}

impl R {
    pub fn atio(n: usize, d: usize) -> R {
        R {
            decimal: n as f32 / d as f32,
            ratio: [n.to_string(), d.to_string()].join("/"),
        }
    }
}

pub struct Gain {
    pub past: f32,
    pub current: f32,
}

impl Gain {
    pub fn new(past: f32, current: f32) -> Gain {
        Gain {
            past,
            current,
        }
    }

    pub fn update(&mut self, new_gain: f32) -> () {
        self.past = self.current;
        self.current = new_gain;
    }
}

impl Oscillator {
    pub fn new(f_buffer_size: usize, ratios: Vec<R>) -> Oscillator {
        println!("{}", "Generated Ratios");
        for r in ratios.iter() {
            println!("   - {}", r.ratio);
        }

        Oscillator {
            f_buffer: RingBuffer::<f32>::new_full(f_buffer_size as usize),
            phases: vec![0.0; ratios.len()],
            ratios,
            generator: Generator::new(),
            gain: Gain::new(1.0, 1.0),
        }
    }

    pub fn update(&mut self, frequency: f32, gain: f32, probability: f32) {
        let mut new_freq = if frequency < 2500.0 { frequency } else { 0.0 };
        let mut new_gain = if new_freq != 0.0 { gain } else { 0.0 };

        if probability < 0.2 {
            new_freq = self.f_buffer.current();
        };

        println!("{}, {}", frequency, new_gain);

        self.f_buffer.push(new_freq);
        self.gain.update(new_gain);
    }

    pub fn generate(&mut self, buffer_size: usize, sample_rate: f32) -> Vec<f32> {
//        println!("{:?}", self.f_buffer);
        let mut frequency = self.f_buffer.current();
        if self.f_buffer.previous() as f32 != 0.0 && self.f_buffer.current() == 0.0 {
            frequency = self.f_buffer.previous();
        }

        let (mut waveform, new_phases) = (self.generator.generate)(
            frequency as f32,
            &self.gain,
            &self.ratios,
            &self.phases,
            buffer_size as usize,
            sample_rate,
        );

        self.phases = new_phases;
        waveform
    }
}

pub mod tests {
    use super::*;
    #[test]
    fn test_ratio() {
        let r: R = R::atio(3, 2);
        let result = r.ratio;
        let expected = "3/2";
        assert_eq!(result, expected);
        let result = r.decimal;
        let expected = 1.5;
        assert_eq!(result, expected);
    }
}
