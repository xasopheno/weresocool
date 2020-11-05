extern crate dsp;

use self::dsp::Sample;
use reverb::Reverb;

// pub fn map_in_place<F, M>(a: &mut Vec<f64>, mut map: M)
pub fn map_in_place<F, M>(a: &mut Vec<F>, mut map: M)
where
    M: FnMut(F) -> F,
    F: Copy,
{
    for f in a {
        *f = map(*f);
    }
}

impl Reverb {
    pub fn audio_requested(&mut self, output: &mut Vec<f64>, _sample_hz: f64) {
        fn to_f32_sample<S: Sample>(s: S) -> f64 {
            s.to_float_sample().to_sample::<f64>()
        }

        fn from_f32_sample<S: Sample>(f: f64) -> S {
            f.to_sample::<S::Float>().to_sample::<S>()
        }

        // match F::n_channels() {
        // // Mono.
        map_in_place(output, |sample| {
            let dry = to_f32_sample(sample);
            let (output_1, output_2) = self.calc_frame(dry as f32, 0.6);
            let avg = (output_1 + output_2) / 2.0;
            from_f32_sample(avg as f64)
        })
    }
}
