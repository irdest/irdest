//#![feature(test)]
//
//#[macro_use]
//extern crate cchamt;
//
//extern crate test;
//
//use test::Bencher;
//use std::usize;
//use cchamt::ContiguousTrie;
//
//#[bench]
//fn bench_10_get_trie(b: &mut Bencher) {
//    let mut trie = ContiguousTrie::<usize>::new(32, 8);
//    let mut v: Vec<Vec<u8>> = Vec::new();
//    let range = 10;
//
//    for i in 0..range {
//        let str = binary_format!(i);
//        let arr = str.to_owned().into_bytes();
//        v.push(arr.clone());
//        trie.insert(i, &arr[2..]);
//    }
//
//    b.iter(|| {
//        for i in 0..range {
//            let _g = trie.get(&v[i][2..]);
//        }
//    });
//}
//
//
//#[bench]
//fn bench_100_get_trie(b: &mut Bencher) {
//    let mut trie = ContiguousTrie::<usize>::new(32, 8);
//    let mut v: Vec<Vec<u8>> = Vec::new();
//    let range = 100;
//
//    for i in 0..range {
//        let str = binary_format!(i);
//        let arr = str.to_owned().into_bytes();
//        v.push(arr.clone());
//        trie.insert(i, &arr[2..]);
//    }
//
//    b.iter(|| {
//        for i in 0..range {
//            let _g = trie.get(&v[i][2..]);
//        }
//    });
//}
//
//
//#[bench]
//fn bench_1k_get_trie(b: &mut Bencher) {
//    let mut trie = ContiguousTrie::<usize>::new(32, 8);
//    let mut v: Vec<Vec<u8>> = Vec::new();
//    let range = 1000;
//
//    for i in 0..range {
//        let str = binary_format!(i);
//        let arr = str.to_owned().into_bytes();
//        v.push(arr.clone());
//        trie.insert(i, &arr[2..]);
//    }
//
//    b.iter(|| {
//        for i in 0..range {
//            let _g = trie.get(&v[i][2..]);
//        }
//    });
//}
//
//#[bench]
//fn bench_10k_get_trie(b: &mut Bencher) {
//    let mut trie = ContiguousTrie::<usize>::new(32, 8);
//    let mut v: Vec<Vec<u8>> = Vec::new();
//    let range = 10000;
//
//    for i in 0..range {
//        let str = binary_format!(i);
//        let arr = str.to_owned().into_bytes();
//        v.push(arr.clone());
//        trie.insert(i, &arr[2..]);
//    }
//
//    b.iter(|| {
//        for i in 0..range {
//            let _g = trie.get(&v[i][2..]);
//        }
//    });
//}
//
//
//#[bench]
//fn bench_100k_get_trie(b: &mut Bencher) {
//    let mut trie = ContiguousTrie::<usize>::new(32, 8);
//    let mut v: Vec<Vec<u8>> = Vec::new();
//    let range = 100000;
//
//    for i in 0..range {
//        let str = binary_format!(i);
//        let arr = str.to_owned().into_bytes();
//        v.push(arr.clone());
//        trie.insert(i, &arr[2..]);
//    }
//
//    b.iter(|| {
//        for i in 0..range {
//            let _g = trie.get(&v[i][2..]);
//        }
//    });
//}
//
//#[bench]
//fn bench_million_get_trie(b: &mut Bencher) {
//    let mut trie = ContiguousTrie::<usize>::new(32, 8);
//    let mut v: Vec<Vec<u8>> = Vec::new();
//    let range = 1000000;
//
//    for i in 0..range {
//        let str = binary_format!(i);
//        let arr = str.to_owned().into_bytes();
//        v.push(arr.clone());
//        trie.insert(i, &arr[2..]);
//    }
//
//    b.iter(|| {
//        for i in 0..range {
//            let _g = trie.get(&v[i][2..]);
//        }
//    });
//}
//
//
//#[bench]
//fn bench_10_million_get_trie(b: &mut Bencher) {
//    let mut trie = ContiguousTrie::<usize>::new(32, 8);
//    let mut v: Vec<Vec<u8>> = Vec::new();
//    let range = 10000000;
//
//    for i in 0..range {
//        let str = binary_format!(i);
//        let arr = str.to_owned().into_bytes();
//        v.push(arr.clone());
//        trie.insert(i, &arr[2..]);
//    }
//
//    b.iter(|| {
//        for i in 0..range {
//            let _g = trie.get(&v[i][2..]);
//        }
//    });
//}
//
//
//#[bench]
//fn bench_100_million_get_trie(b: &mut Bencher) {
//    let mut trie = ContiguousTrie::<usize>::new(32, 8);
//    let mut v: Vec<Vec<u8>> = Vec::new();
//    let range = 10000000;
//
//    for i in 0..range {
//        let str = binary_format!(i);
//        let arr = str.to_owned().into_bytes();
//        v.push(arr.clone());
//        trie.insert(i, &arr[2..]);
//    }
//
//    b.iter(|| {
//        for i in 0..range {
//            let _g = trie.get(&v[i][2..]);
//        }
//    });
//}
//
//
//#[bench]
//fn bench_1000_million_get_trie(b: &mut Bencher) {
//    let mut trie = ContiguousTrie::<usize>::new(32, 8);
//    let mut v: Vec<Vec<u8>> = Vec::new();
//    let range = 100000000;
//
//    for i in 0..range {
//        let str = binary_format!(i);
//        let arr = str.to_owned().into_bytes();
//        v.push(arr.clone());
//        trie.insert(i, &arr[2..]);
//    }
//
//    b.iter(|| {
//        for i in 0..range {
//            let _g = trie.get(&v[i][2..]);
//        }
//    });
//}
