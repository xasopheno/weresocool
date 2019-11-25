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
        let frequency = self.calculate_gain(silent_next, silence_now, op.index + index, op.total_samples);
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

pub fn calculate_current_phase(&mut self, info: &SampleInfo, rand: f64) {
    let frequency = if self.sound_to_silence() {
        self.past.frequency
    } else if self.portamento_index < info.portamento_length
        && !self.silence_to_sound()
        && !self.sound_to_silence()
    {
        self.past.frequency + (info.index as f64 * info.p_delta)
    } else {
        self.current.frequency
    };

    let gain = info.gain;
    let current_phase = if gain == 0.0 {
        0.0
    } else {
        ((info.factor * frequency) + self.phase + rand) % tau()
    };

    self.phase = current_phase;
}

self.calculate_current_phase(&info, 0.0);

self.phase.sin() * info.gain
