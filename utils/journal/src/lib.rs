//use cchamt::LockfreeTrie;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

pub trait JournalData: Clone + Copy + Eq + PartialEq {}

impl<T> JournalData for T where T: Clone + Copy + Eq + PartialEq {}

pub trait JournalKey: Clone + Copy + Eq + PartialEq + Hash {}

impl<T> JournalKey for T where T: Clone + Copy + Eq + PartialEq + Hash {}

#[derive(Serialize, Deserialize)]
enum JournalInner<K: JournalKey, V: JournalData> {
    Hash { inner: HashMap<K, V> },
    // Trie { inner: LockfreeTrie<K, V> },
}

/// Generic serializable key value store
#[derive(Serialize, Deserialize)]
pub struct Journal<K: JournalKey, V: JournalData> {
    inner_impl: JournalInner<K, V>,
}

impl<K: JournalKey, V: JournalData> Journal<K, V> {
    pub fn new_with_hashmap() -> Self {
        Self {
            inner_impl: JournalInner::Hash {
                inner: HashMap::default(),
            },
        }
    }

    /*pub fn new_with_trie() -> Self {
        Self {
            inner_impl: JournalInner::Trie {
                inner: LockfreeTrie::new(),
            },
        }
    }*/

    pub fn insert(&mut self, key: K, value: V) {
        match self.inner_impl {
            JournalInner::Hash { ref mut inner } => {
                _ = inner.insert(key, value);
            } /*JournalInner::Trie { ref mut inner } => {
                  _ = inner.insert(key, value);
              }*/
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        match self.inner_impl {
            JournalInner::Hash { ref inner } => inner.get(key),
            /*JournalInner::Trie { ref inner } => inner.lookup(key),*/
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        match self.inner_impl {
            JournalInner::Hash { ref mut inner } => inner.remove(key),
            /*JournalInner::Trie { inner: _ } => {
                unimplemented!("trie can't remove nodes")
            }*/
        }
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {
        //let result = add(2, 2);
        //assert_eq!(result, 4);
    }
}
