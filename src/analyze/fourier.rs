use std::f32::consts::PI as PI;
use std::f32::consts::E as E;
use num_complex::Complex;

//pub trait Fourier {
//    fn separate(&mut self);
//    fn fft(&mut self);
//}

//impl Fourier for Vec<f32> {
pub fn separate(array: &mut Vec<f32>) -> Vec<f32> {
    let mut a = array.clone();
    let n = a.len();
    let mut temp_array = vec![0.0; n / 2];
    for i in 0..n / 2 {
        temp_array[i] = a[i * 2 + 1]
    }
    for i in 0..n / 2 {
        a[i] = a[i * 2]
    }
    for i in 0..n / 2 {
        a[i + n / 2] = temp_array[i]
    }

    a
}

pub fn vec_f32_to_complex(array: &mut Vec<f32>) -> Vec<Complex<f32>> {
     array.clone()
        .iter_mut()
        .map(|x| Complex::new(*x as f32, 0f32))
        .collect::<Vec<_>>()
}

//pub fn fft(&mut self) {
//    let length = self.len();
//    if length < 2 {}
//    else {
//        self.separate();
//
//
//
//        let mut even = self[0..length / 2].to_vec();
//        even.fft();
//        let mut odd = self[length / 2..length].to_vec();
//        odd.fft();
//
//        for k in 0..length/2 {
//            let e = even[k];
//            let o = odd[k];
//            let complex = Complex::new(0.0, -2.0 * PI * k as f32 / length as f32);
//            let w = complex.exp();
//            self[k] = e + w * o;
//            self[length/2 + k] = e - w * o;
//        }
//    }
//}
////}

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
        let mut array = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let mut result = separate(&mut array);
        let expected = vec![0.0, 2.0, 4.0, 6.0, 1.0, 3.0, 5.0, 7.0];

        assert_eq!(result, expected);
    }

    #[test]
    fn fourier_array_to_complex_test() {
        let mut array = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let mut result = vec_f32_to_complex(&mut array);
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
//
//    #[test]
//    fn fourier_fft() {
////        let n_seconds = 1.0;
////        let sample_rate = n_samples as f32 / n_seconds;
////        let freq_resolution = sample_rate / n_samples as f32;
//        let input = vec![1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0];
//
//        let mut result = input.clone();
//        result.fft();
//
//        let expected = vec![4.000, 2.613, 0.000, 1.082, 0.000, 1.082, 0.000, 2.613];
//
//        assert_eq!(expected, result);
//    }
}
