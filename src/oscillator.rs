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
        let mut frequency = self.f_buffer.current();
        if self.f_buffer.previous() as f32 != 0.0 && self.f_buffer.current() == 0.0 {
            frequency = self.f_buffer.previous();
        }
//        let mut frequency = self.f_buffer.to_vec().iter().sum::<f32>() as f32/ self.f_buffer.to_vec().len() as f32;
        let (mut waveform, new_phase) = (self.generator)(frequency as f32, self.phase, buffer_size as usize, sample_rate);
        let mut faded = false;
        if self.f_buffer.previous() as f32 == 0.0 && self.f_buffer.current() != 0.0 {
            for (i, sample) in self.fader.fade_in.iter().enumerate() {
                waveform[i] = waveform[i] * sample;
            }
        }

        let mut print = false;
        if self.f_buffer.previous() as f32 != 0.0 && self.f_buffer.current() == 0.0 {
            print = true;
            for (i, sample) in self.fader.fade_out.iter().enumerate() {
                waveform[i] = waveform[i] * sample;
            }
        }
        self.phase = new_phase;
        waveform
    }
}
