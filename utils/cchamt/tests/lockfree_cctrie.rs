#[macro_use]
extern crate cchamt;

extern crate rand;

use cchamt::LockfreeTrie;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::usize;

#[test]
fn test_lockfree_new_lockfree_trie() {
    let trie = LockfreeTrie::<u64, usize>::new();
}

#[test]
fn test_lockfree_2_power_16_insert() {
    let mut trie = LockfreeTrie::<u64, u64>::new();

    for i in 0..65536 {
        trie.insert(i, i + 1);
    }

    for i in 0..65536 {
        if let Some(j) = trie.lookup(&i) {
            assert_eq!(*j, (i + 1) as u64);
        } else {
            assert!(false, "<{}> not found", i);
        }
    }
}

#[test]
fn test_lockfree_million_consecutive_insert() {
    let mut trie = LockfreeTrie::<u64, u64>::new();

    for i in 0..1000000 {
        trie.insert(i, i + 1);
    }

    for i in 0..1000000 {
        if let Some(j) = trie.lookup(&i) {
            assert_eq!(*j, (i + 1) as u64);
        } else {
            assert!(false, "<{}> not found", i);
        }
    }
}
