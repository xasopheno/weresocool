use std;

#[derive(Debug, Clone)]
pub struct RingBuffer<T: Copy + Clone + Sized> {
    buffer: Vec<T>,
    top: usize,
}

impl<T: Sized + Copy + Clone + std::default::Default> RingBuffer<T> {
    pub fn new(capacity: usize) -> RingBuffer<T> {
        RingBuffer {
            buffer: vec![T::default(); capacity],
            top: 0
        }
    }
    pub fn append(&mut self, values: Vec<T>) 
        where T: Clone + Copy {
        for v in values.iter() {
            self.buffer[self.top] = *v;
            self.top += 1;
            self.top %= self.buffer.len(); 
        }
    }
    pub fn to_vec(&self) -> Vec<T> {
        let mut new_vec = vec![T::default(); self.buffer.len()];
        for index in 0..self.buffer.len() {
            let ring_buffer_index = (index + self.top)% self.buffer.len();
            new_vec[index] = self.buffer[ring_buffer_index]; 
        }
        new_vec
    }   
} 


pub mod tests {
    use super::*;
    #[test]
    fn ring_buffer() {
        let mut rb = RingBuffer::<usize>::new(10);
        let input = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let expected = vec![4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
        rb.append(input);
        rb.append(vec![11, 12, 13]);
        assert_eq!(rb.to_vec(), expected);
    }
}