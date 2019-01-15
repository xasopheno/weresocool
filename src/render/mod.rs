use generation::parsed_to_render::r_to_f64;
use instrument::{
    oscillator::{OscType, Oscillator},
    stereo_waveform::StereoWaveform,
};
use operations::{NormalForm, PointOp};

pub trait Render<T> {
    fn render(&mut self, oscillator: &mut Oscillator) -> StereoWaveform;
}

impl Render<PointOp> for PointOp {
    fn render(&mut self, oscillator: &mut Oscillator) -> StereoWaveform {
        oscillator.update(self);
        let n_samples_to_generate = r_to_f64(self.l) * oscillator.basis.l * 44_100.0;
        oscillator.generate(n_samples_to_generate)
    }
}

impl Render<Vec<PointOp>> for Vec<PointOp> {
    fn render(&mut self, oscillator: &mut Oscillator) -> StereoWaveform {
        let mut result: StereoWaveform = StereoWaveform::new(0);
        let mut p_ops = self.clone();
        p_ops.push(PointOp::init_silent());
        for mut p_op in p_ops {
            let stereo_waveform = p_op.render(oscillator);
            result.append(stereo_waveform);
        }

        result
    }
}

#[cfg(test)]
mod test;
