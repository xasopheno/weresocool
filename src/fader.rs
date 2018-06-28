pub struct Fader {
    pub fade_in: Vec<f32>,
    pub fade_out: Vec<f32>,
}

impl Fader {
    pub fn new(fade_in_length: usize, fade_out_length: usize, buffer_size: usize) -> Fader {
        let mut fade_out: Vec<f32> = generate_fade_out(fade_out_length);
        for _i in 0..(buffer_size - fade_out_length + 1) {
            fade_out.push(0.0);
        }
        Fader {
            fade_in: generate_fade_in(fade_in_length),
            fade_out,
        }
    }
}

pub fn generate_fade_out(length: usize) -> Vec<f32> {
    let mut fade_vec: Vec<usize> = (1..length).collect();
    fade_vec
        .iter_mut()
        .map(|sample| (1.0 - *sample as f32 * 1.0 / length as f32))
        .collect()
}

pub fn generate_fade_in(length: usize) -> Vec<f32> {
    let mut fade_vec: Vec<f32> = generate_fade_out(length);
    fade_vec.reverse();
    fade_vec
}

// ******************** EXAMPLE ****************************
//        if self.f_buffer.previous() as f32 == 0.0 && self.f_buffer.current() != 0.0 {
//            println!("{}", "FADE_IN");
//            for (i, sample) in self.fader.fade_in.iter().enumerate() {
//                waveform[i] = waveform[i] * sample;
//            }
//        }
//
//        if self.f_buffer.previous() as f32 != 0.0 && self.f_buffer.current() == 0.0 {
//            println!("{}", "FADE_OUT");
//            for (i, sample) in self.fader.fade_out.iter().enumerate() {
//                waveform[i] = waveform[i] * sample;
//            }
//        }

pub mod tests {
    use super::*;

    #[test]
    fn new_fader() {
        let fader = Fader::new(5, 10, 12);
        let expected_fade_in = [0.19999999, 0.39999998, 0.6, 0.8];
        let expected_fade_out = vec![
            0.9,
            0.8,
            0.7,
            0.6,
            0.5,
            0.39999998,
            0.3,
            0.19999999,
            0.100000024,
            0.0,
            0.0,
            0.0,
        ];
        assert_eq!(fader.fade_in, expected_fade_in);
        assert_eq!(fader.fade_out, expected_fade_out);
    }
}
