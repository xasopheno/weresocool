pub struct Fader {
    pub length: usize,
    pub fade_in: Vec<f32>,
    pub fade_out: Vec<f32>,
}

impl Fader {
    pub fn new(length: usize, buffer_size: usize) -> Fader {
        let mut fade_out: Vec<f32> = generate_fade_out(length);
        for _i in 0..(buffer_size - length + 1) {
            fade_out.push(-0.0);
        };
        Fader {
            length,
            fade_in: generate_fade_in(length),
            fade_out
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
        let fader = Fader::new(10, 12);
        let expected_fade_in = [0.100000024, 0.19999999, 0.3, 0.39999998, 0.5, 0.6, 0.7, 0.8, 0.9];
        let expected_fade_out = vec![0.9, 0.8, 0.7, 0.6, 0.5, 0.39999998, 0.3, 0.19999999, 0.100000024, 0.0, 0.0, 0.0 ];
        assert_eq!(fader.fade_in, expected_fade_in);
        assert_eq!(fader.fade_out, expected_fade_out);
    }
}
