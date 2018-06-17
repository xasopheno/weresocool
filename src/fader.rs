pub struct Fader {
    pub length: usize,
    pub fade_in: Vec<f32>,
    pub fade_out: Vec<f32>,
}

impl Fader {
    pub fn new(length: usize) -> Fader {
        Fader {
            length,
            fade_in: generate_fade_in(length),
            fade_out: generate_fade_out(length)
        }
    }
}

pub fn generate_fade_out(length: usize) -> Vec<f32> {
    let mut fade_vec: Vec<usize> = (1..length).collect();
    fade_vec
        .iter_mut()
        .map(|sample|(1.0 - *sample as f32 * 1.0/length as f32))
        .collect()
}

pub fn generate_fade_in(length: usize) -> Vec<f32>{
    let mut fade_vec: Vec<f32> = generate_fade_out(length);
    fade_vec.reverse();
    fade_vec
}

pub mod tests {
    use super::*;

    #[test]
    fn new_fader() {
        let fader = Fader::new(10);
        let expected_fade_in = vec![0.1, 0.11111111, 0.125, 0.14285715, 0.16666667, 0.2, 0.25, 0.33333334, 0.5, 1.0];
        let expected_fade_out = vec![1.0, 0.5, 0.33333334, 0.25, 0.2, 0.16666667, 0.14285715, 0.125, 0.11111111, 0.1];
        assert_eq!(fader.fade_in, expected_fade_in);
        assert_eq!(fader.fade_out, expected_fade_out);
    }
}
