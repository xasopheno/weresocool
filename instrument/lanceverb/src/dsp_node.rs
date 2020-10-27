extern crate dsp;

use reverb::Reverb;
use self::dsp::Sample;

impl<F> dsp::Node<F> for Reverb
    where F: dsp::Frame,
{
    fn audio_requested(&mut self, output: &mut [F], _sample_hz: f64) {

        fn to_f32_sample<S: Sample>(s: S) -> f32 {
            s.to_float_sample().to_sample::<f32>()
        }

        fn from_f32_sample<S: Sample>(f: f32) -> S {
            f.to_sample::<S::Float>().to_sample::<S>()
        }

        match F::n_channels() {
            // Mono.
            1 => dsp::slice::map_in_place(output, |frame| frame.map(|sample| {
                let dry = to_f32_sample(sample);
                let (output_1, output_2) = self.calc_frame(dry, 0.6);
                let avg = (output_1 + output_2) / 2.0;
                from_f32_sample(avg)
            })),
            // Stereo.
            2 => dsp::slice::map_in_place(output, |frame| {
                let dry = frame.channels().fold(0.0, |sum, s| sum + to_f32_sample(s)) / 2.0;
                let (output_1, output_2) = self.calc_frame(dry, 0.6);
                let (left, right) = (from_f32_sample(output_1), from_f32_sample(output_2));
                dsp::Frame::from_fn(|i| if i == 0 { left } else { right })
            }),
            // No other number of channels is supported.
            n => panic!("The given number of channels ({:?}) is not supported by lanceverb", n),
        }
    }
}


