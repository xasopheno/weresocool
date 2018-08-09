use std::f32::consts::PI as PI;
use std::f32::consts::E as E;
use num_complex::Complex;

pub trait Fourier {
    fn separate(&mut self);
    fn fft(&mut self);
}

impl Fourier for Vec<f32> {
    fn separate(&mut self) {
        let n = self.len();
        let mut temp_array = vec![0.0; n / 2];
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
        if length < 2 {}
        else {
            self.separate();
            let mut even = self[0..length / 2].to_vec();
            even.fft();
            let mut odd = self[length / 2..length].to_vec();
            odd.fft();

            for k in 0..length/2 {
                let e = even[k];
                let o = odd[k];
                let complex = Complex::new(0.0, -2.0 * PI * k as f32 / length);
                let w = E.powf(complex);
                self[k] = e + w * o;
                self[length/2 + k] = e - w * o;
            }
        }
    }
}

//def fft(x):
    //N = len(x)
        //if N <= 1: return x
            //even = fft(x[0::2])
            //odd =  fft(x[1::2])
            //T= [exp(-2j*pi*k/N)*odd[k] for k in range(N//2)]
            //return [even[k] + T[k] for k in range(N//2)] + \
            //[even[k] - T[k] for k in range(N//2)]

#[cfg(test)]
pub mod tests {
    use super::*;
    use settings::get_test_settings;
    use instrument::oscillator::Oscillator;
    use ratios::{R, Pan};
    #[test]
    fn fourier_separate_test() {
        let array = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let mut result = array.clone();
        result.separate();
        let expected = vec![0.0, 2.0, 4.0, 6.0, 8.0, 1.0, 3.0, 5.0, 7.0, 9.0];

        assert_eq!(result, expected);
    }

    #[test]
    fn fourier_fft() {
//        let n_seconds = 1.0;
//        let sample_rate = n_samples as f32 / n_seconds;
//        let freq_resolution = sample_rate / n_samples as f32;
        let input = vec![1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0];

        let mut result = input.clone();
        result.fft();

        let expected = vec![4.000, 2.613, 0.000, 1.082, 0.000, 1.082, 0.000, 2.613];

        assert_eq!(expected, result);
    }
}
