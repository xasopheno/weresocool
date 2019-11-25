pub fn update(&mut self, op: &RenderOp) {
    self.portamento_index = 0;
    self.past.frequency = self.current.frequency;
    self.current.frequency = op.f;

    self.past.gain = self.current.gain;

    self.current.gain = match self.index {
        0 => op.g.0 * loudness_normalization(op.f),
        _ => op.g.1 * loudness_normalization(op.f),
    };

    self.osc_type = op.osc_type;

    self.attack = op.attack.trunc() as usize;
    self.decay = op.decay.trunc() as usize;

    self.asr = op.asr;
}


pub fn generate_waveform(
    &mut self,
    op: &RenderOp, 
) -> Vec<f64> {
    let mut buffer: Vec<f64> = vec![0.0; op.samples];

    let factor: f64 = tau() / 44_100.0;
    let p_delta = self.calculate_portamento_delta(op.portamento);
    let silence_now = self.current.gain == 0.0 || self.current.frequency == 0.0;

    let silent_next = match self.index {
        0 => op.next_l_silent,
        _ => op.next_r_silent,
    };

    for (index, sample) in buffer.iter_mut().enumerate() {
        let gain =
            self.calculate_gain(silent_next, silence_now, op.index + index, op.total_samples);
        let info = SampleInfo {
            index: op.index + index,
            p_delta,
            gain,
            portamento_length: op.portamento,
            factor,
        };
        let new_sample = match self.osc_type {
            OscType::Sine => self.generate_sine_sample(info),
            OscType::Square => self.generate_square_sample(info),
            OscType::Noise => self.generate_random_sample(info),
        };
        self.portamento_index += 1;

        *sample += new_sample
    }
    buffer
}
