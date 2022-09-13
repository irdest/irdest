/*
#[macro_use]
extern crate cchamt;

extern crate rand;

use cchamt::MutexContiguousTrie;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::usize;

#[test]
fn test_new_contiguous_trie() {
    let trie = MutexContiguousTrie::<usize>::new(32, 8);
}

#[test]
fn test_2_power_16_insert() {
    let mut trie = MutexContiguousTrie::<usize>::new(32, 8);

    for i in 0..65536 {
        let str = binary_format!(i);
        let arr = str.to_owned().into_bytes();
        trie.insert(i, &arr[2..]);
    }

    for i in 0..65536 {
        let str = binary_format!(i);
        let arr = str.to_owned().into_bytes();
        assert_eq!(trie.get(&arr[2..]).unwrap(), i);
    }
}

#[test]
fn test_million_consecutive_insert() {
    let mut trie = MutexContiguousTrie::<usize>::new(32, 8);

    for i in 0..1000000 {
        let str = binary_format!(i);
        let arr = str.to_owned().into_bytes();
        trie.insert(i, &arr[2..]);
    }

    for i in 0..1000000 {
        let str = binary_format!(i);
        let arr = str.to_owned().into_bytes();
        assert_eq!(trie.get(&arr[2..]).unwrap(), i);
    }
}
*/
