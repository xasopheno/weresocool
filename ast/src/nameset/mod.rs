use std::collections::BTreeMap;

/// Set of Names associated with a PointOp
#[derive(Debug, Clone, Hash, Eq, Ord, PartialOrd, Default, PartialEq)]
pub struct NameSet {
    index: usize,
    map: BTreeMap<String, usize>,
}

impl NameSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<S>(&mut self, value: S)
    where
        S: Into<String>,
    {
        self.map.insert(value.into(), self.index);
        self.index += 1;
    }

    pub fn to_vec(&self) -> Vec<String> {
        let count_b: BTreeMap<&usize, &String> = self.map.iter().map(|(k, v)| (v, k)).collect();
        count_b
            .values()
            .into_iter()
            .map(|v| v.to_string())
            .collect()
    }

    pub fn to_vec_str(&self) -> Vec<&str> {
        let count_b: BTreeMap<&usize, &str> =
            self.map.iter().map(|(k, v)| (v, k.as_str())).collect();
        count_b.values().into_iter().copied().collect()
    }

    pub fn last(&self) -> Option<String> {
        let vec = self.to_vec();
        if !vec.is_empty() {
            Some(vec.last().unwrap().to_string())
        } else {
            None
        }
    }

    pub fn contains(&self, value: &str) -> bool {
        self.map.contains_key(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_sorts_values_by_occurrence() {
        let mut set = NameSet::new();
        let events = vec!["a", "b", "a", "c", "a", "d", "c", "a", "c"];
        for event in events {
            set.insert(event);
        }

        assert_eq!(set.to_vec(), vec!["b", "d", "a", "c"]);
    }
}
