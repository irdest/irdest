use async_std::sync::Arc;
use std::collections::{BTreeMap, BTreeSet};

/// A simple structure to look-up existing categories, namespaces, and
/// topics
#[derive(Default)]
pub struct Lookup {
    /// A set of known namespaces on a given category
    categories: BTreeMap<String, BTreeSet<String>>,
    /// A set of known topics on a given namespace
    namespaces: BTreeMap<String, BTreeSet<String>>,
}

impl Lookup {
    /// Take a list of Strings in the format of
    /// `/category/namespace/topic` and parse them into dedicated
    /// lookup tables for each type.
    // TODO: hook this up to Ratman/ the database
    pub fn populate(input: Vec<&str>) -> Self {
        input
            .into_iter()
            .filter_map(|s| {
                let mut split = s.split("/").map(|s| s.to_string());
                let first = split.next();

                match first {
                    None => None,
                    Some(s) if s.as_str() != "" => None,
                    _ => Some((split.next(), split.next(), split.next())),
                }
            })
            .filter_map(|(c, n, t)| match (c, n, t) {
                (Some(c), Some(n), Some(t)) => Some((c, n, t)),
                _ => None,
            })
            .fold(Self::default(), |mut _self, (c, n, t)| {
                _self
                    .categories
                    .entry(c.clone())
                    .or_default()
                    .insert(n.clone());
                _self
                    .namespaces
                    .entry(n.clone())
                    .or_default()
                    .insert(t.clone());
                _self
            })
    }

    pub fn categories(&self) -> Vec<String> {
        self.categories.iter().map(|(k, _)| k.clone()).collect()
    }

    pub fn namespaces(&self) -> Vec<String> {
        self.categories
            .iter()
            .map(|(_, n_set)| n_set.iter().map(|ns| ns.clone()))
            .flatten()
            .collect()
    }

    /// Return all known topics
    // TODO: this should be replaced with a database lookup -- there's
    // really no reason to get the information out of this structure
    // again except during testing
    pub fn all(self: &Arc<Self>) -> Vec<String> {
        let mut buf = vec![];
        for (c, n_set) in self.categories.iter() {
            for n in n_set.iter() {
                for t in self.namespaces.get(n).as_ref().unwrap().iter() {
                    buf.push(format!("/{}/{}/{}", c, n, t));
                }
            }
        }
        buf.sort();
        buf
    }
}
