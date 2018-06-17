use std;
use std::ops::{Index};

#[derive(Debug, Clone)]
pub struct RingBuffer<T: Copy + Clone + Sized> {
    buffer: Vec<T>,
    head: usize,
    tail: usize,
    capacity: usize,
}

impl<T: Sized + Copy + Clone + std::default::Default> RingBuffer<T> {
    pub fn new(capacity: usize) -> RingBuffer<T> {
        RingBuffer {
//            buffer: vec![T::default(); capacity],
            buffer: vec![],
            head: 0,
            tail: 0,
            capacity,
        }
    }
    pub fn push_vec(&mut self, values: Vec<T>)
    where
        T: Clone + Copy,
    {
        for v in values.iter() {
            self.push(*v)
        }
    }

    pub fn push(&mut self, value: T)
        where
            T: Clone + Copy,
    {
        if self.buffer.len() < self.capacity {
            self.buffer.push(value);
            self.tail += 1;
        } else {
            self.buffer[self.head] = value;
            self.head = (self.head + 1) % self.capacity;
            self.tail = (self.tail + 1) % self.capacity;
        }
    }
    pub fn to_vec(&self) -> Vec<T> {
        if self.buffer.len() < self.capacity {
            return self.buffer.clone();
        };
        let mut new_vec = vec![T::default(); self.capacity];
        for index in 0..self.buffer.len() {
            let ring_buffer_index = (index + self.head) % self.capacity;
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
        let expected = vec![1, 2, 3, 4, 5];
        assert_eq!(rb.to_vec(), expected);
        rb.push(256);
        let expected = vec![1, 2, 3, 4, 5, 256];
        assert_eq!(rb.to_vec(), expected);
    }

    #[test]
    fn ring_buffer_start() {
        let capacity = 3;
        let mut rb = RingBuffer::<f32>::new(capacity);
        rb.push(440.0);
        let expected = vec![440.0];
        assert_eq!(rb.to_vec(), expected);
        rb.push(441.0);
        let expected = vec![440.0, 441.0];
        assert_eq!(rb.to_vec(), expected);
        rb.push(442.0);
        let expected = vec![440.0, 441.0, 442.0];
        assert_eq!(rb.to_vec(), expected);
        rb.push(443.0);
        let expected = vec![441.0, 442.0, 443.0];
        assert_eq!(rb.to_vec(), expected);
    }
    #[test]
    fn ring_buffer_empty() {
        let capacity = 3;
        let rb = RingBuffer::<usize>::new(capacity);
        assert_eq!(rb.to_vec(), vec![]);
    }
}
