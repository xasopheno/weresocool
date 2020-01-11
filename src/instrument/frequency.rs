use crate::instrument::voice::Voice;

impl Voice {
    pub fn calculate_portamento_delta(
        &self,
        portamento_length: usize,
        start: f64,
        target: f64,
    ) -> f64 {
        (target - start) / (portamento_length as f64)
    }

    pub fn calculate_frequency(
        &self,
        index: usize,
        portamento_length: usize,
        p_delta: f64,
        start: f64,
        target: f64,
    ) -> f64 {
        if self.sound_to_silence() {
            start
        } else if index < portamento_length && !self.silence_to_sound() && !self.sound_to_silence()
        {
            (index as f64).mul_add(p_delta, start)
        } else {
            target
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_portamento_delta() {
        let v = Voice::init(0);
        let result = v.calculate_portamento_delta(10, 0.0, 100.0);
        assert_eq!(result, 10.0);
        let result = v.calculate_portamento_delta(10, 100.0, 0.0);
        assert_eq!(result, -10.0);
        let result = v.calculate_portamento_delta(10, 90.0, 100.0);
        assert_eq!(result, 1.0);
    }
    #[test]
    fn test_calculate_frequency() {
        for i in 0..10 {
            let v = Voice::init(0);
            let result = v.calculate_frequency(i, 4, 25.0, 0.0, 100.0);
            let expected = std::cmp::min(100, result as usize);
            assert_eq!(result as usize, expected);
        }
    }
}
