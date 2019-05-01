use num_complex::Complex;
use std::f64::consts::E;
use std::f64::consts::PI;

pub fn vec_f64_to_complex(array: &mut [f64]) -> Vec<Complex<f64>> {
    array
        .iter_mut()
        .map(|x| Complex::new(*x as f64, 0f64))
        .collect::<Vec<_>>()
}

pub fn magnitude(array: &mut [Complex<f64>]) -> Vec<f64> {
    array
        .iter_mut()
        .map(|x| (x.re * x.re + x.im * x.im).sqrt())
        .collect::<Vec<_>>()
}

pub trait Fourier {
    fn separate(&mut self);
    fn fft(&mut self);
}

impl Fourier for Vec<Complex<f64>> {
    fn separate(&mut self) {
        let n = self.len();
        let mut temp_array = vec![Complex::new(0.0, 0.0); n / 2];
        for i in 0..n / 2 {
            temp_array[i] = self[i * 2 + 1]
        }
        for i in 0..n / 2 {
            self[i] = self[i * 2]
        }
        for i in 0..n / 2 {
            self[i + n / 2] = temp_array[i]
        }
    }

    fn fft(&mut self) {
        let length = self.len();
        if length < 2 {
        } else {
            self.separate();

            let mut even = self[0..length / 2].to_vec();
            even.fft();
            let mut odd = self[length / 2..length].to_vec();
            odd.fft();

            for k in 0..length / 2 {
                let e = even[k];
                let o = odd[k];
                let complex = Complex::new(0.0, -2.0 * PI * k as f64 / length as f64);
                let w = complex.exp();
                self[k] = e + w * o;
                self[length / 2 + k] = e - w * o;
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use instrument::oscillator::Oscillator;
    use ratios::{Pan, R};
    use settings::get_test_settings;

    #[test]
    fn fourier_array_to_complex_test() {
        let mut array = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let mut result = vec_f64_to_complex(&mut array);
        let expected = vec![
            Complex::new(0.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(2.0, 0.0),
            Complex::new(3.0, 0.0),
            Complex::new(4.0, 0.0),
            Complex::new(5.0, 0.0),
            Complex::new(6.0, 0.0),
            Complex::new(7.0, 0.0),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn fourier_separate_test() {
        let mut array = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let mut result = vec_f64_to_complex(&mut array);
        result.separate();
        let expected = vec![
            Complex::new(0.0, 0.0),
            Complex::new(2.0, 0.0),
            Complex::new(4.0, 0.0),
            Complex::new(6.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(3.0, 0.0),
            Complex::new(5.0, 0.0),
            Complex::new(7.0, 0.0),
        ];
        assert_eq!(result, expected);
    }
    //
    #[test]
    fn fourier_fft() {
        let mut input = vec![1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0];
        let input = vec_f64_to_complex(&mut input);
        let mut result = input.clone();
        result.fft();

        let expected = vec![
            Complex { re: 4.0, im: 0.0 },
            Complex {
                re: 0.9999999,
                im: -2.4142134,
            },
            Complex { re: 0.0, im: 0.0 },
            Complex {
                re: 1.0,
                im: -0.41421354,
            },
            Complex { re: 0.0, im: 0.0 },
            Complex {
                re: 1.0,
                im: 0.41421342,
            },
            Complex { re: 0.0, im: 0.0 },
            Complex {
                re: 1.0,
                im: 2.4142137,
            },
        ];
        assert_eq!(result, expected);
    }
}
