use fader::Fader;
use ring_buffer::RingBuffer;
use sine::generate_waveform;

pub struct Oscillator {
    pub f_buffer: RingBuffer<f32>,
    pub ratios: Vec<R>,
    pub phases: Vec<f32>,
    pub generator:
        fn(freq: f32, ratios: &Vec<R>, phases: &Vec<f32>, buffer_size: usize, sample_rate: f32)
            -> (Vec<f32>, Vec<f32>),
    pub fader: Fader,
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

impl Oscillator {
    pub fn new(f_buffer_size: usize, ratios: Vec<R>, fader: Fader) -> Oscillator {
        println!("{}", "Generated Ratios");
        for r in ratios.iter() {
            println!("   - {}", r.ratio);
        }
        Oscillator {
            f_buffer: RingBuffer::<f32>::new_full(f_buffer_size as usize),
            phases: vec![0.0; ratios.len()],
            ratios,
            generator: generate_waveform,
            fader,
        }
    }

    pub fn generate(&mut self, buffer_size: usize, sample_rate: f32) -> Vec<f32> {
        let mut frequency = self.f_buffer.current();
        if self.f_buffer.previous() as f32 != 0.0 && self.f_buffer.current() == 0.0 {
            frequency = self.f_buffer.previous();
        }

        let (mut waveform, new_phases) = (self.generator)(
            frequency as f32,
            &self.ratios,
            &self.phases,
            buffer_size as usize,
            sample_rate,
        );
        if self.f_buffer.previous() as f32 == 0.0 && self.f_buffer.current() != 0.0 {
            for (i, sample) in self.fader.fade_in.iter().enumerate() {
                waveform[i] = waveform[i] * sample;
            }
        }

        if self.f_buffer.previous() as f32 != 0.0 && self.f_buffer.current() == 0.0 {
            for (i, sample) in self.fader.fade_out.iter().enumerate() {
                waveform[i] = waveform[i] * sample;
            }
        }
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
