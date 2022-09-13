//#![feature(test)]
//
//#[macro_use]
//extern crate cchamt;
//
//extern crate test;
//extern crate rand;
//
//use test::Bencher;
//use std::usize;
//use std::collections::HashMap;
//use rand::{Rng, thread_rng};
//use cchamt::ContiguousTrie;
//
//
//#[bench]
//fn bench_10_get_trie(b: &mut Bencher) {
//    let mut trie = ContiguousTrie::<usize>::new(32, 8);
//    let mut v: Vec<Vec<u8>> = vec![];
//    let range = 10;
//    {
//        let slice: &mut [Vec<u8>] = v.as_mut_slice();
//        thread_rng().shuffle(slice);
//    }
//
//    for i in 0..range {
//        let str = binary_format!(i);
//        let arr = str.to_owned().into_bytes();
//        v.push(arr.clone());
//        trie.insert(i, &arr[2..]);
//    }
//
//    b.iter(|| {
//        for arr in &v {
//            let _g = trie.get(&arr[2..]);
//        }
//    });
//}
//
//
//#[bench]
//fn bench_10_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 10;
//    let mut v: Vec<usize> = (0..10).collect();
//    {
//        let slice: &mut [usize] = v.as_mut_slice();
//        thread_rng().shuffle(slice);
//    }
//
//    for i in &v {
//        hash.insert(*i as usize, *i as usize);
//    }
//
//    b.iter(|| {
//        for i in &v {
//            let _g = hash.get(&i);
//        }
//    });
//}
//
//#[bench]
//fn bench_100_get_trie(b: &mut Bencher) {
//    let mut trie = ContiguousTrie::<usize>::new(32, 8);
//    let mut v: Vec<Vec<u8>> = vec![];
//    let range = 100;
//    {
//        let slice: &mut [Vec<u8>] = v.as_mut_slice();
//        thread_rng().shuffle(slice);
//    }
//
//    for i in 0..range {
//        let str = binary_format!(i);
//        let arr = str.to_owned().into_bytes();
//        v.push(arr.clone());
//        trie.insert(i, &arr[2..]);
//    }
//
//    b.iter(|| {
//        for arr in &v {
//            let _g = trie.get(&arr[2..]);
//        }
//    });
//}
//
//
//#[bench]
//fn bench_100_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 100;
//    let mut v: Vec<usize> = (0..100).collect();
//    {
//        let slice: &mut [usize] = v.as_mut_slice();
//        thread_rng().shuffle(slice);
//    }
//
//    for i in &v {
//        hash.insert(*i as usize, *i as usize);
//    }
//
//    b.iter(|| {
//        for i in &v {
//            let _g = hash.get(&i);
//        }
//    });
//}
//
//#[bench]
//fn bench_1000_get_trie(b: &mut Bencher) {
//    let mut trie = ContiguousTrie::<usize>::new(32, 8);
//    let mut v: Vec<Vec<u8>> = vec![];
//    let range = 1000;
//    {
//        let slice: &mut [Vec<u8>] = v.as_mut_slice();
//        thread_rng().shuffle(slice);
//    }
//
//    for i in 0..range {
//        let str = binary_format!(i);
//        let arr = str.to_owned().into_bytes();
//        v.push(arr.clone());
//        trie.insert(i, &arr[2..]);
//    }
//
//    b.iter(|| {
//        for arr in &v {
//            let _g = trie.get(&arr[2..]);
//        }
//    });
//}
//
//
//#[bench]
//fn bench_1000_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 1000;
//    let mut v: Vec<usize> = (0..1000).collect();
//    {
//        let slice: &mut [usize] = v.as_mut_slice();
//        thread_rng().shuffle(slice);
//    }
//
//    for i in &v {
//        hash.insert(*i as usize, *i as usize);
//    }
//
//    b.iter(|| {
//        for i in &v {
//            let _g = hash.get(&i);
//        }
//    });
//}
//
//#[bench]
//fn bench_10000_get_trie(b: &mut Bencher) {
//    let mut trie = ContiguousTrie::<usize>::new(32, 8);
//    let mut v: Vec<Vec<u8>> = vec![];
//    let range = 10000;
//    {
//        let slice: &mut [Vec<u8>] = v.as_mut_slice();
//        thread_rng().shuffle(slice);
//    }
//
//    for i in 0..range {
//        let str = binary_format!(i);
//        let arr = str.to_owned().into_bytes();
//        v.push(arr.clone());
//        trie.insert(i, &arr[2..]);
//    }
//
//    b.iter(|| {
//        for arr in &v {
//            let _g = trie.get(&arr[2..]);
//        }
//    });
//}
//
//
//#[bench]
//fn bench_10000_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 10000;
//    let mut v: Vec<usize> = (0..10000).collect();
//    {
//        let slice: &mut [usize] = v.as_mut_slice();
//        thread_rng().shuffle(slice);
//    }
//
//    for i in &v {
//        hash.insert(*i as usize, *i as usize);
//    }
//
//    b.iter(|| {
//        for i in &v {
//            let _g = hash.get(&i);
//        }
//    });
//}
//
//#[bench]
//fn bench_100000_get_trie(b: &mut Bencher) {
//    let mut trie = ContiguousTrie::<usize>::new(32, 8);
//    let mut v: Vec<Vec<u8>> = vec![];
//    let range = 100000;
//    {
//        let slice: &mut [Vec<u8>] = v.as_mut_slice();
//        thread_rng().shuffle(slice);
//    }
//
//    for i in 0..range {
//        let str = binary_format!(i);
//        let arr = str.to_owned().into_bytes();
//        v.push(arr.clone());
//        trie.insert(i, &arr[2..]);
//    }
//
//    b.iter(|| {
//        for arr in &v {
//            let _g = trie.get(&arr[2..]);
//        }
//    });
//}
//
//
//#[bench]
//fn bench_100000_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 100000;
//    let mut v: Vec<usize> = (0..100000).collect();
//    {
//        let slice: &mut [usize] = v.as_mut_slice();
//        thread_rng().shuffle(slice);
//    }
//
//    for i in &v {
//        hash.insert(*i as usize, *i as usize);
//    }
//
//    b.iter(|| {
//        for i in &v {
//            let _g = hash.get(&i);
//        }
//    });
//}
//
//#[bench]
//fn bench_1000000_get_trie(b: &mut Bencher) {
//    let mut trie = ContiguousTrie::<usize>::new(32, 8);
//    let mut v: Vec<Vec<u8>> = vec![];
//    let range = 1000000;
//    {
//        let slice: &mut [Vec<u8>] = v.as_mut_slice();
//        thread_rng().shuffle(slice);
//    }
//
//    for i in 0..range {
//        let str = binary_format!(i);
//        let arr = str.to_owned().into_bytes();
//        v.push(arr.clone());
//        trie.insert(i, &arr[2..]);
//    }
//
//    b.iter(|| {
//        for arr in &v {
//            let _g = trie.get(&arr[2..]);
//        }
//    });
//}
//
//
//#[bench]
//fn bench_1000000_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 1000000;
//    let mut v: Vec<usize> = (0..1000000).collect();
//    {
//        let slice: &mut [usize] = v.as_mut_slice();
//        thread_rng().shuffle(slice);
//    }
//
//    for i in &v {
//        hash.insert(*i as usize, *i as usize);
//    }
//
//    b.iter(|| {
//        for i in &v {
//            let _g = hash.get(&i);
//        }
//    });
//}
//
//#[bench]
//fn bench_10000000_get_trie(b: &mut Bencher) {
//    let mut trie = ContiguousTrie::<usize>::new(32, 8);
//    let mut v: Vec<Vec<u8>> = vec![];
//    let range = 10000000;
//    {
//        let slice: &mut [Vec<u8>] = v.as_mut_slice();
//        thread_rng().shuffle(slice);
//    }
//
//    for i in 0..range {
//        let str = binary_format!(i);
//        let arr = str.to_owned().into_bytes();
//        v.push(arr.clone());
//        trie.insert(i, &arr[2..]);
//    }
//
//    b.iter(|| {
//        for arr in &v {
//            let _g = trie.get(&arr[2..]);
//        }
//    });
//}
//
//
//#[bench]
//fn bench_10000000_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 10000000;
//    let mut v: Vec<usize> = (0..10000000).collect();
//    {
//        let slice: &mut [usize] = v.as_mut_slice();
//        thread_rng().shuffle(slice);
//    }
//
//    for i in &v {
//        hash.insert(*i as usize, *i as usize);
//    }
//
//    b.iter(|| {
//        for i in &v {
//            let _g = hash.get(&i);
//        }
//    });
//}
//
