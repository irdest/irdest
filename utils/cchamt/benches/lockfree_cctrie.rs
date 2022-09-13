#![feature(test)]

#[macro_use]
extern crate cchamt;

extern crate test;

use cchamt::LockfreeTrie;
use std::collections::HashMap;
use std::usize;
use test::Bencher;

#[bench]
fn bench_1k_get_trie(b: &mut Bencher) {
    let mut trie = LockfreeTrie::<usize, usize>::new();
    let mut v: Vec<Vec<u8>> = Vec::new();
    let range = 1000;

    for i in 0..range {
        trie.insert(i, i + 1);
    }

    b.iter(|| {
        for i in 0..range {
            let _g = trie.lookup(&i);
        }
    });
}

#[bench]
fn bench_1k_get_hashmap(b: &mut Bencher) {
    let mut hash = HashMap::new();
    let range = 1000;
    for i in 0..range {
        hash.insert(i as usize, i as usize);
    }
    b.iter(|| {
        for i in 0..range {
            let _g = hash.get(&i);
        }
    });
}

#[bench]
fn bench_100k_get_trie(b: &mut Bencher) {
    let mut trie = LockfreeTrie::<usize, usize>::new();
    let range = 100000;

    for i in 0..range {
        trie.insert(i, i + 1);
    }

    b.iter(|| {
        for i in 0..range {
            let _g = trie.lookup(&i);
        }
    });
}

#[bench]
fn bench_100k_get_hashmap(b: &mut Bencher) {
    let mut hash = HashMap::new();
    let range = 100000;
    for i in 0..range {
        hash.insert(i as usize, i as usize);
    }
    b.iter(|| {
        for i in 0..range {
            let _g = hash.get(&i);
        }
    });
}

#[bench]
fn bench_million_get_trie(b: &mut Bencher) {
    let mut trie = LockfreeTrie::<usize, usize>::new();
    let mut v: Vec<Vec<u8>> = Vec::new();
    let range = 1000000;

    for i in 0..range {
        trie.insert(i, i + 1);
    }

    b.iter(|| {
        for i in 0..range {
            let _g = trie.lookup(&i);
        }
    });
}

#[bench]
fn bench_million_get_hashmap(b: &mut Bencher) {
    let mut hash = HashMap::new();
    let range = 1000000;
    for i in 0..range {
        hash.insert(i as usize, i as usize);
    }
    b.iter(|| {
        for i in 0..range {
            let _g = hash.get(&i);
        }
    });
}

//#[bench]
//fn bench_10_million_get_trie(b: &mut Bencher) {
//    let mut trie = LockfreeTrie::<usize,usize>::new();
//    let mut v: Vec<Vec<u8>> = Vec::new();
//    let range = 10000000;
//
//    for i in 0..range {
//        trie.insert(i, i+1);
//    }
//
//    b.iter(|| {
//        for i in 0..range {
//            let _g = trie.lookup(&i);
//        }
//    });
//}
//
//
//#[bench]
//fn bench_10_million_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 10000000;
//    for i in 0..range {
//        hash.insert(i as usize, i as usize);
//    }
//    b.iter(|| {
//        for i in 0..range {
//            let _g = hash.get(&i);
//        }
//    });
//}
//
//#[bench]
//fn bench_100_million_get_trie(b: &mut Bencher) {
//    let mut trie = LockfreeTrie::<usize,usize>::new();
//    let range = 10000000;
//
//    for i in 0..range {
//        trie.insert(i, i+1);
//    }
//
//    b.iter(|| {
//        for i in 0..range {
//            let _g = trie.lookup(&i);
//        }
//    });
//}
//
//
//#[bench]
//fn bench_100_million_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 100000000;
//    for i in 0..range {
//        hash.insert(i as usize, i as usize);
//    }
//    b.iter(|| {
//        for i in 0..range {
//            let _g = hash.get(&i);
//        }
//    });
//}
