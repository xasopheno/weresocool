use std::collections::HashSet;

pub fn set_elements(vec: Vec<i32>) -> Vec<i32> {
    let set_hash: HashSet<i32> = vec.iter().cloned().collect();
    let mut set_vec: Vec<i32> = set_hash.iter().cloned().collect();
    set_vec.sort();
    set_vec
}

pub mod tests {
    use set_elements::set_elements;
    #[test]
    fn test_set_elements() {
        let test_case = vec![1, 1, 2, 3];
        let expected = vec![1, 2, 3];
        
        assert_eq!(set_elements(test_case), 
        expected);
    }
}