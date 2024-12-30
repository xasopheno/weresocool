use std::fmt::Debug;

#[derive(Clone)]
pub struct Resizeable2DVec<T> {
    inner: Vec<Vec<T>>,
}

impl<T: Clone> Resizeable2DVec<T> {
    /// Creates a new Resizable2DVec with a specified outer size.
    /// Each inner vector will be initialized as an empty vector.
    pub fn new(outer_size: usize) -> Self {
        let mut inner = Vec::with_capacity(outer_size);
        for _ in 0..outer_size {
            inner.push(Vec::new());
        }
        Self { inner }
    }

    /// Pushes a single item to the inner vector at the specified index.
    /// If the index is out of bounds, the vector will grow to accommodate it.
    pub fn push_at(&mut self, index: usize, item: T) {
        self.ensure_size(index + 1);
        self.inner[index].push(item);
    }

    /// Extends the inner vector at the specified index with multiple items.
    /// If the index is out of bounds, the vector will grow to accommodate it.
    pub fn extend_at(&mut self, index: usize, items: Vec<T>) {
        self.ensure_size(index + 1);
        self.inner[index].extend(items);
    }

    /// Returns a reference to the vector at the given index.
    pub fn get(&self, index: usize) -> Option<&Vec<T>> {
        self.inner.get(index)
    }

    /// Flattens the 2D vector into a single 1D `Vec<T>`.
    /// The order of elements is row-by-row, preserving the order of items in each row.
    /// This does not consume self.
    pub fn to_vec_flat(&self) -> Vec<T> {
        self.inner.iter().flat_map(|v| v.clone()).collect()
    }

    /// Clones the entire 2D vector into a `Vec<Vec<T>>`.
    /// This method makes a deep copy of the 2D structure without consuming self.
    pub fn to_vec(&self) -> Vec<Vec<T>> {
        self.inner.clone()
    }

    /// Ensures the 2D vector has at least `new_size` outer vectors.
    /// If the size is smaller, it extends the outer vector with empty inner vectors.
    fn ensure_size(&mut self, new_size: usize) {
        if self.inner.len() < new_size {
            self.inner
                .extend((self.inner.len()..new_size).map(|_| Vec::new()));
        }
    }
}

impl<T: Debug + Clone> Debug for Resizeable2DVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resizable2DVec")
            .field("inner", &self.inner)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extend_at() {
        let mut vec2d: Resizeable2DVec<i32> = Resizeable2DVec::new(3);

        vec2d.extend_at(0, vec![1, 2, 3]);
        vec2d.extend_at(2, vec![4, 5, 6]);

        assert_eq!(vec2d.get(0), Some(&vec![1, 2, 3]));
        assert_eq!(vec2d.get(1), Some(&Vec::new()));
        assert_eq!(vec2d.get(2), Some(&vec![4, 5, 6]));
    }

    #[test]
    fn test_push_at() {
        let mut vec2d: Resizeable2DVec<i32> = Resizeable2DVec::new(3);

        vec2d.push_at(0, 1);
        vec2d.push_at(0, 2);
        vec2d.push_at(2, 3);
        vec2d.push_at(2, 4);

        assert_eq!(vec2d.get(0), Some(&vec![1, 2]));
        assert_eq!(vec2d.get(1), Some(&Vec::new()));
        assert_eq!(vec2d.get(2), Some(&vec![3, 4]));
    }

    #[test]
    fn test_ensure_size() {
        let mut vec2d: Resizeable2DVec<i32> = Resizeable2DVec::new(2);

        vec2d.push_at(3, 42);
        assert_eq!(vec2d.get(2), Some(&Vec::new()));
        assert_eq!(vec2d.get(3), Some(&vec![42]));
    }

    #[test]
    fn test_to_vec_flat() {
        let mut vec2d: Resizeable2DVec<i32> = Resizeable2DVec::new(3);
        vec2d.push_at(0, 10);
        vec2d.push_at(0, 20);
        vec2d.push_at(2, 30);
        vec2d.push_at(2, 40);

        let flat = vec2d.to_vec_flat();
        assert_eq!(flat, vec![10, 20, 30, 40]);
    }

    #[test]
    fn test_to_vec() {
        let mut vec2d: Resizeable2DVec<i32> = Resizeable2DVec::new(3);
        vec2d.push_at(0, 10);
        vec2d.push_at(0, 20);
        vec2d.push_at(2, 30);
        vec2d.push_at(2, 40);

        let cloned = vec2d.to_vec();
        assert_eq!(cloned, vec![vec![10, 20], vec![], vec![30, 40]]);
    }

    #[test]
    fn test_debug() {
        let mut vec2d: Resizeable2DVec<i32> = Resizeable2DVec::new(2);
        vec2d.push_at(0, 42);
        let debug_output = format!("{:?}", vec2d);
        assert!(debug_output.contains("Resizable2DVec"));
        assert!(debug_output.contains("42"));
    }
}
