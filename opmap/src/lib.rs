use drain_while::DrainWhileable;
use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
pub struct OpMap<T>
where
    T: Clone + std::default::Default,
{
    pub map: HashMap<String, Vec<T>>,
}

impl<T> OpMap<T>
where
    T: Clone + std::default::Default,
{
    pub fn join(&mut self, mut other: OpMap<T>, sort_fn: fn(&T, &T) -> std::cmp::Ordering) {
        for (k, v) in other.map.iter_mut() {
            v.sort_by(sort_fn);
            if self.map.contains_key(k) {
                self.map
                    .entry(k.to_owned())
                    .or_default()
                    .extend(v.to_owned());
            } else {
                self.map.insert(k.to_owned(), v.to_owned());
            }
        }
    }

    pub fn insert(&mut self, k: &str, v: T) {
        self.map.entry(k.to_owned()).or_default().push(v);
    }

    pub fn set(&mut self, k: &str, v: Vec<T>) {
        self.map.insert(k.to_string(), v);
    }

    pub fn get(&mut self, k: &str) -> Option<&Vec<T>> {
        self.map.get(k)
    }

    pub fn drain<P>(&mut self, k: &str, predicate: P) -> Vec<T>
    where
        P: Fn(&T) -> bool,
    {
        if self.map.contains_key(k) {
            let v = self.map.get_mut(k).unwrap();
            v.drain_while(predicate).collect()
        } else {
            vec![]
        }
    }

    pub fn from_vec<P>(v: Vec<T>, key_from_t: P) -> OpMap<T>
    where
        P: Fn(&T) -> &str,
    {
        let mut result: OpMap<T> = OpMap::default();
        v.iter().for_each(|t| {
            let k = key_from_t(t);
            result.insert(k, t.to_owned());
        });

        result
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use hamcrest2::prelude::*;

    #[derive(Default, Debug, Clone, PartialEq)]
    struct TestStruct {
        t: f32,
    }

    #[test]
    fn test_insert() {
        let mut ops = OpMap::default();
        let a = TestStruct { t: 0.0 };
        let b = TestStruct { t: 1.0 };
        let c = TestStruct { t: 2.0 };
        let d = TestStruct { t: 3.0 };

        ops.insert("x", a.clone());
        ops.insert("y", b.clone());
        ops.insert("x", c.clone());
        ops.insert("z", d.clone());

        assert_that!(ops.map.get("x").unwrap(), contains(vec![a, c]).exactly());
        assert_that!(ops.map.get("y").unwrap(), contains(vec![b]).exactly());
        assert_that!(ops.map.get("z").unwrap(), contains(vec![d]).exactly());
    }

    #[test]
    fn test_join() {
        let mut ops1 = OpMap::default();
        let mut ops2 = OpMap::default();
        let a = TestStruct { t: 0.0 };
        let b = TestStruct { t: 1.0 };
        let c = TestStruct { t: 2.0 };
        let d = TestStruct { t: 3.0 };
        let e = TestStruct { t: 4.0 };
        let f = TestStruct { t: 5.0 };

        ops1.insert("x", a.clone());
        ops1.insert("y", b.clone());

        ops2.insert("x", d.clone());
        ops2.insert("x", c.clone());
        ops2.insert("y", e.clone());
        ops2.insert("z", f.clone());

        ops1.join(ops2, |a, b| a.t.partial_cmp(&b.t).unwrap());

        assert_that!(
            ops1.map.get("x").unwrap(),
            contains(vec![a, c, d]).in_order()
        );
        assert_that!(ops1.map.get("y").unwrap(), contains(vec![b, e]).in_order());
        assert_that!(ops1.map.get("z").unwrap(), contains(vec![f]).in_order());
    }
}
