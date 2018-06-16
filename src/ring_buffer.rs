use std;
use std::ops::{Index};

#[derive(Debug, Clone)]
pub struct RingBuffer<T: Copy + Clone + Sized> {
    buffer: Vec<T>,
    top: usize,
    capacity: usize,
    fill: usize,
}

impl<T: Sized + Copy + Clone + std::default::Default> RingBuffer<T> {
    pub fn new(capacity: usize) -> RingBuffer<T> {
        RingBuffer {
            buffer: vec![T::default(); capacity],
            top: 0,
            fill: 0,
            capacity,
        }
    }
    pub fn push_vec(&mut self, values: Vec<T>)
    where
        T: Clone + Copy,
    {
        for v in values.iter() {
            self.buffer[self.top] = *v;
            self.top += 1;
            self.fill += 1;
            self.top %= self.capacity;
        }
    }

    pub fn push(&mut self, value: T)
        where
            T: Clone + Copy,
    {
        self.buffer[self.top] = value;
        self.top += 1;
        self.top %= self.capacity;
    }
    pub fn to_vec(&self) -> Vec<T> {
        let mut new_vec = vec![T::default(); self.capacity];
        let top = if self.fill < self.capacity { 0 } else { self.top };
        for index in 0..self.buffer.len() {
            let ring_buffer_index = (index + top) % self.capacity;
            new_vec[index] = self.buffer[ring_buffer_index];
        }
        new_vec
    }
}

impl<T> Index<usize> for RingBuffer<T>
    where T: std::marker::Copy
{
    type Output = T;

    fn index(&self, idx: usize) -> &Self::Output {
        self.buffer.index((idx + self.top) % self.capacity)
    }
}

pub mod tests {
    use super::*;
    #[test]
    fn ring_buffer() {
        let mut rb = RingBuffer::<usize>::new(10);
        let input = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        rb.push_vec(input);
        rb.push_vec(vec![11, 12, 13]);
        let expected = vec![4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
        rb.push_vec(vec![50]);
        let expected = vec![5, 6, 7, 8, 9, 10, 11, 12, 13, 50];
        assert_eq!(rb.to_vec(), expected);
    }
    #[test]
    fn ring_buffer_vec_push_start() {
        let mut rb = RingBuffer::<usize>::new(10);
        let input = vec![1, 2, 3, 4, 5];
        rb.push_vec(input);
        let expected = vec![1, 2, 3, 4, 5, 0, 0, 0, 0, 0];
        assert_eq!(rb.to_vec(), expected);
    }
    #[test]
    fn ring_buffer_start() {
        let mut rb = RingBuffer::<f32>::new(5);
        rb.push(440.0);
        let expected = vec![440.0, 0.0, 0.0, 0.0, 0.0];
        assert_eq!(rb.to_vec(), expected);
        rb.push(441.0);
        let expected = vec![441.0, 440.0, 0.0, 0.0, 0.0];
        assert_eq!(rb.to_vec(), expected);
    }
}
