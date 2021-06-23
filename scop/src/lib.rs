use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::iter::Rev;
use std::slice::Iter;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug, PartialEq, Deserialize, Serialize)]
pub enum ScopError {
    #[error("`{0}`")]
    Error(String),
}

pub type Term = i32;
pub type Def<T> = IndexMap<String, T>;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Defs<T> {
    defs: IndexMap<String, Def<T>>,
    scopes: Vec<String>,
    pub stems: HashSet<String>,
}

impl<T> Default for Defs<T>
where
    T: Clone + Sized,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Defs<T>
where
    T: Clone + Sized,
{
    pub fn new() -> Self {
        let mut defs = IndexMap::new();
        defs.insert("global".into(), IndexMap::new());
        Self {
            defs,
            scopes: vec!["global".to_string()],
            stems: HashSet::new(),
        }
    }

    pub fn push_stem<S: Into<String> + Clone>(&mut self, stem_name: S) {
        self.stems.insert(stem_name.into());
    }

    pub fn flat_map(
        &self,
        f: Box<dyn Fn(&str, &str, &T) -> Result<(), ScopError>>,
    ) -> Result<(), ScopError> {
        for (scope_name, scope) in self.iter() {
            for (name, term) in scope {
                f(scope_name, name, term)?;
            }
        }
        Ok(())
    }

    pub fn iter_scopes(&self) -> Rev<Iter<String>> {
        self.scopes.iter().rev()
    }

    pub fn iter(
        &self,
    ) -> indexmap::map::Iter<'_, std::string::String, IndexMap<std::string::String, T>> {
        self.defs.iter()
    }

    pub fn create_uuid_scope(&mut self) -> String {
        let new_scope = Uuid::new_v4().to_string();
        self.defs.insert(new_scope.to_string(), Def::new());
        self.scopes.push(new_scope.to_string());
        new_scope
    }

    #[allow(dead_code)]
    pub fn create_named_scope<S: Into<String> + Clone>(&mut self, new_scope: S) {
        self.defs.insert(new_scope.clone().into(), Def::new());
        self.scopes.push(new_scope.into());
    }
    pub fn insert<S: Into<String>>(&mut self, scope: &str, name: S, value: T) {
        let current_scope = self
            .defs
            .entry(scope.to_string())
            .or_insert_with(IndexMap::new);
        current_scope.insert(name.into(), value);
    }

    pub fn get(&self, id: &str) -> Option<&T> {
        for scope in self.scopes.iter().rev() {
            let current = self
                .defs
                .get(&scope.to_string())
                .expect("Named scope not found");
            let result = current.get(&id.to_string());
            if result.is_some() {
                return result;
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_can_insert_and_find_in_global_scope() -> Result<(), ScopError> {
        let mut defs: Defs<Term> = Defs::new();

        defs.insert("global", "1", 1);
        defs.insert("global", "2", 2);

        let found = defs.get("1");
        assert_eq!(found, Some(&1));
        let not_found = defs.get("3");
        assert_eq!(not_found, None);
        Ok(())
    }

    #[test]
    fn it_can_insert_and_find_with_uuid_scope_name() -> Result<(), ScopError> {
        let mut defs: Defs<Term> = Defs::new();

        defs.insert("global", "1", 1);
        defs.insert("global", "2", 2);

        let new_scope = defs.create_uuid_scope();
        defs.insert(&new_scope, "3", 3);

        let found = defs.get("3");
        assert_eq!(found, Some(&3));
        Ok(())
    }

    #[test]
    fn it_finds_value_in_innermost_scope() -> Result<(), ScopError> {
        let mut defs: Defs<Term> = Defs::new();

        defs.insert("global", "1", 1);
        defs.insert("global", "2", 2);

        let new_scope = defs.create_uuid_scope();
        defs.insert(&new_scope, "1", 10);

        let found = defs.get("1");
        assert_eq!(found, Some(&10));
        Ok(())
    }
}
