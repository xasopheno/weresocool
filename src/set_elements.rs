use std::collections::HashSet;

pub fn set_elements(vec: Vec<i32>) -> Vec<i32> {
    let set_hash: HashSet<i32> = vec.iter().cloned().collect();
    let mut set_vec: Vec<i32> = set_hash.iter().cloned().collect();
    set_vec.sort();
    set_vec
}

