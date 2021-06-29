mod test;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::iter::Rev;
use std::slice::Iter;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug, PartialEq, Deserialize, Serialize)]
/// Scope Error
pub enum ScopError {
    #[error("`{0}`")]
    Error(String),
}

/// This is an inner scope. A mapping of id to T.
pub type Def<T> = IndexMap<String, T>;

/// Defs contain:
///  a mapping of scope_name to a mapping of an id to T,
///  an array of scopes in order of scope creation.
///  a set of stem names
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
    /// This new is specific to WereSoCool. It creates an IndexMap<String, IndexMap> to keep track
    /// of the map of scopes. An IndexMap is an ordered HashMap.
    /// It also creates an array of scopes with a global scope prepopulated.
    /// Finally, it creates HashSet for stem names.
    pub fn new() -> Self {
        let mut defs = IndexMap::new();
        defs.insert("global".into(), IndexMap::new());
        Self {
            defs,
            scopes: vec!["global".to_string()],
            stems: HashSet::new(),
        }
    }

    /// Insert a stem into stems set.
    pub fn push_stem<S: Into<String> + Clone>(&mut self, stem_name: S) {
        self.stems.insert(stem_name.into());
    }

    /// This isn't used or useful, yet. :) Perhaps I'll come back to this when it's needed.
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

    /// Iterate through the scope stack from inner to outer scopes.
    pub fn iter_scopes(&self) -> Rev<Iter<String>> {
        self.scopes.iter().rev()
    }

    /// Returns an iterator over each scope.
    pub fn iter(
        &self,
    ) -> indexmap::map::Iter<'_, std::string::String, IndexMap<std::string::String, T>> {
        self.defs.iter()
    }

    /// Returns a mutable iterator over each scope.
    pub fn iter_mut(
        &mut self,
    ) -> indexmap::map::IterMut<'_, std::string::String, IndexMap<std::string::String, T>> {
        self.defs.iter_mut()
    }

    /// Create a new scope with a uuid as the scope name. Returns the scope name that was created.
    pub fn create_uuid_scope(&mut self) -> String {
        let new_scope = Uuid::new_v4().to_string();
        self.defs.insert(new_scope.to_string(), Def::new());
        self.scopes.push(new_scope.to_string());
        new_scope
    }

    #[allow(dead_code)]
    /// Creates a named scope with the name passed in.
    pub fn create_named_scope<S: Into<String> + Clone>(&mut self, new_scope: S) {
        self.defs.insert(new_scope.clone().into(), Def::new());
        self.scopes.push(new_scope.into());
    }

    /// Insert a into a given scope under the name provided.
    pub fn insert<S: Into<String>>(&mut self, scope: &str, name: S, value: T) {
        let current_scope = self
            .defs
            .entry(scope.to_string())
            .or_insert_with(IndexMap::new);
        current_scope.insert(name.into(), value);
        if !self.scopes.contains(&scope.into()) {
            self.scopes.push(scope.into());
        }
    }

    /// Searches through inner -> outer scopes looking for the given id.
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
