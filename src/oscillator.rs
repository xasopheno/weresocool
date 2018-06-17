use ring_buffer::RingBuffer;
use fader::Fader;

pub struct Oscillator {
    pub f_buffer: RingBuffer<f32>,
    pub phase: f32,
    pub generator:
    fn(freq: f32, phase: f32, buffer_size: usize, sample_rate: f32) -> (Vec<f32>, f32),
    pub fader: Fader,
    pub faded_in: bool,
}

impl Oscillator {
    pub fn generate(&mut self, buffer_size: usize, sample_rate: f32) -> Vec<f32> {
        let (mut waveform, new_phase) = (self.generator)(self.f_buffer.current() as f32, self.phase, buffer_size as usize, sample_rate);
        println!("{},{}", self.f_buffer.previous() as f32 == 0.0 && self.f_buffer.current() != 0.0, self.f_buffer.previous() as f32);
        let mut faded = false;
        if self.f_buffer.previous() as f32 == 0.0 && self.f_buffer.current() != 0.0 {
            faded = true;
            for (i, sample) in self.fader.fade_in.iter().enumerate() {
                waveform[i] = waveform[i] * sample;
            }
        }
        self.phase = new_phase;
        if faded == true {
            println!("{:?}", waveform);
        }
        waveform
    }
}
