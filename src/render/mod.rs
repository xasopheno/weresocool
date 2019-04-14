use crate::generation::parsed_to_render::r_to_f64;
use crate::instrument::{
    oscillator::{Basis, Oscillator},
    stereo_waveform::StereoWaveform,
};
use socool_ast::operations::PointOp;

pub trait Render<T> {
    fn render(&mut self, origin: &Basis, oscillator: &mut Oscillator) -> StereoWaveform;
}

pub trait RenderPointOp<T> {
    fn render(
        &mut self,
        origin: &Basis,
        oscillator: &mut Oscillator,
        next_op: Option<PointOp>,
    ) -> StereoWaveform;
}

impl RenderPointOp<PointOp> for PointOp {
    fn render(
        &mut self,
        origin: &Basis,
        oscillator: &mut Oscillator,
        next_op: Option<PointOp>,
    ) -> StereoWaveform {
        oscillator.update(origin.clone(), self, next_op);

        let n_samples_to_generate = r_to_f64(self.l) * origin.l * 44_100.0;
        let portamento_length = r_to_f64(self.portamento);

        oscillator.generate(n_samples_to_generate, portamento_length)
    }
}

impl Render<Vec<PointOp>> for Vec<PointOp> {
    fn render(&mut self, origin: &Basis, oscillator: &mut Oscillator) -> StereoWaveform {
        let mut result: StereoWaveform = StereoWaveform::new(0);
        let mut p_ops = self.clone();
        p_ops.push(PointOp::init_silent());

        let mut iter = p_ops.iter().peekable();

        while let Some(p_op) = iter.next() {
            let mut next_op = None;
            let peek = iter.peek();
            match peek {
                Some(p) => next_op = Some(p.clone().clone()),
                None => {}
            };

            let stereo_waveform = p_op.clone().render(origin, oscillator, next_op);
            result.append(stereo_waveform);
        }

        result
    }
}

#[cfg(test)]
mod test;
