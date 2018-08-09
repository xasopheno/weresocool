pub trait Fourier {
    fn separate(&mut self, n: usize);

}

impl Fourier for Vec<f32> {
    fn separate(&mut self, n: usize) {
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
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn fourier_separate_test() {
        let array: Vec<f32> = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let mut result = array.clone();
        result.separate(10);
        let expected = vec![0.0, 2.0, 4.0, 6.0, 8.0, 1.0, 3.0, 5.0, 7.0, 9.0];

        assert_eq!(result, expected);
    }
}
