extern crate num_rational;
extern crate socool_ast;
use generation::parsed_to_render::r_to_f64;
use instrument::{
    oscillator::{Origin, Oscillator},
    stereo_waveform::StereoWaveform,
};
use num_rational::Rational64;
use socool_ast::operations::PointOp;

pub trait Render<T> {
    fn render(&mut self, origin: &Origin, oscillator: &mut Oscillator, silence_next: bool) -> StereoWaveform;
}

impl Render<PointOp> for PointOp {
    fn render(&mut self, origin: &Origin, oscillator: &mut Oscillator, silence_next: bool) -> StereoWaveform {
        println!("{:?}", self);
        println!("{:?}", silence_next);
        oscillator.update(origin.clone(), self);
        let n_samples_to_generate = r_to_f64(self.l) * origin.l * 44_100.0;

        oscillator.generate(n_samples_to_generate)
    }
}

impl Render<Vec<PointOp>> for Vec<PointOp> {
    fn render(&mut self, origin: &Origin, oscillator: &mut Oscillator, _silence_next: bool) -> StereoWaveform {
        let mut result: StereoWaveform = StereoWaveform::new(0);
        let mut p_ops = self.clone();
        p_ops.push(PointOp::init_silent());

        let mut iter = p_ops.iter().peekable();

        while let Some(p_op) = iter.next() {
            let pk = iter.peek();
            let silence_next = match pk {
                Some(p) => {
                    if p.g == Rational64::new(0,1) || p.fm == Rational64::new(0,1) {
                        true
                    } else {
                        false
                    }
                },
                None => { true }
            };

            let stereo_waveform = p_op.clone().render(origin, oscillator, silence_next);
            result.append(stereo_waveform);
        }

        result
    }
}

#[cfg(test)]
mod test;
