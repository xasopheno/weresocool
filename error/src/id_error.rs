use crate::{Error, ErrorInner};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
pub struct IdError {
    pub id: String,
    /// Defined names close enough to the missing one to be worth offering.
    /// A point-free language is a language of NAMES — the overwhelming
    /// majority of "why doesn't this work" is a name that doesn't quite
    /// match, and the compiler is holding the list of names it does know.
    #[serde(default)]
    pub did_you_mean: Vec<String>,
}

impl IdError {
    pub fn into_error(self) -> Error {
        Error {
            inner: Box::new(ErrorInner::IdError(self)),
        }
    }
}

impl fmt::Display for IdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not find id: {}", self.id)?;
        match self.did_you_mean.len() {
            0 => Ok(()),
            1 => write!(f, " — did you mean `{}`?", self.did_you_mean[0]),
            _ => write!(
                f,
                " — did you mean {}?",
                self.did_you_mean
                    .iter()
                    .map(|s| format!("`{s}`"))
                    .collect::<Vec<_>>()
                    .join(" or ")
            ),
        }
    }
}

/// Names within striking distance of `id`, best first, at most three.
///
/// The threshold scales with length: a three-letter name may differ by one,
/// a long one by up to a third of itself. Anything looser starts offering
/// names that merely happen to be short.
pub fn nearest_names<'a>(id: &str, known: impl Iterator<Item = &'a String>) -> Vec<String> {
    let budget = (id.chars().count() / 3).max(1);
    let mut scored: Vec<(usize, &String)> = known
        .filter(|k| !k.starts_with("__"))
        .map(|k| (edit_distance(id, k), k))
        .filter(|(d, _)| *d <= budget)
        .collect();
    scored.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(b.1)));
    scored.dedup_by(|a, b| a.1 == b.1);
    scored.into_iter().take(3).map(|(_, k)| k.clone()).collect()
}

/// Levenshtein distance, two rows instead of a full matrix.
fn edit_distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    if a.is_empty() {
        return b.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            cur[j + 1] = (prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn offers_the_near_miss() {
        let known = names(&["tower", "river", "birds"]);
        assert_eq!(nearest_names("towerz", known.iter()), vec!["tower".to_string()]);
    }

    #[test]
    fn offers_nothing_for_an_unrelated_name() {
        let known = names(&["tower", "river"]);
        assert!(nearest_names("kaleidoscope", known.iter()).is_empty());
    }

    #[test]
    fn a_short_name_gets_one_edit_of_slack() {
        let known = names(&["bd", "hh", "sn"]);
        assert_eq!(nearest_names("bs", known.iter()), vec!["bd".to_string()]);
    }

    #[test]
    fn hidden_synthetic_defs_are_never_offered() {
        let known = names(&["__ref_0", "tower"]);
        assert_eq!(nearest_names("__ref_1", known.iter()), Vec::<String>::new());
    }

    #[test]
    fn message_reads_as_a_sentence() {
        let e = IdError { id: "towerz".into(), did_you_mean: names(&["tower"]) };
        assert_eq!(format!("{e}"), "Could not find id: towerz — did you mean `tower`?");
    }
}
