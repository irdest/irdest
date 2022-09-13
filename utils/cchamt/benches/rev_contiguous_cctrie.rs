//#![feature(test)]
//
//#[macro_use]
//extern crate cchamt;
//
//extern crate test;
//
//use test::Bencher;
//use std::usize;
//use std::collections::HashMap;
//use cchamt::ContiguousTrie;
//
//#[bench]
//fn bench_rev_10_get_trie(b: &mut Bencher) {
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
//            let _g = trie.get(&v[range-i][2..]);
//        }
//    });
//}
//
//
//#[bench]
//fn bench_rev_10_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 10;
//    for i in 0..range {
//        hash.insert(i as usize, i as usize);
//    }
//    b.iter(|| {
//        for i in 0..range {
//			let x = range - i;
//            let _g = hash.get(&x);
//        }
//    });
//}
//
//#[bench]
//fn bench_rev_100_get_trie(b: &mut Bencher) {
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
//            let _g = trie.get(&v[range-i][2..]);
//        }
//    });
//}
//
//
//#[bench]
//fn bench_rev_100_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 100;
//    for i in 0..range {
//        hash.insert(i as usize, i as usize);
//    }
//    b.iter(|| {
//        for i in 0..range {
//			let x = range - i;
//            let _g = hash.get(&x);
//        }
//    });
//}
//
//#[bench]
//fn bench_rev_1000_get_trie(b: &mut Bencher) {
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
//            let _g = trie.get(&v[range-i][2..]);
//        }
//    });
//}
//
//
//#[bench]
//fn bench_rev_1000_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 1000;
//    for i in 0..range {
//        hash.insert(i as usize, i as usize);
//    }
//    b.iter(|| {
//        for i in 0..range {
//			let x = range - i;
//            let _g = hash.get(&x);
//        }
//    });
//}
//
//#[bench]
//fn bench_rev_10000_get_trie(b: &mut Bencher) {
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
//            let _g = trie.get(&v[range-i][2..]);
//        }
//    });
//}
//
//
//#[bench]
//fn bench_rev_10000_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 10000;
//    for i in 0..range {
//        hash.insert(i as usize, i as usize);
//    }
//    b.iter(|| {
//        for i in 0..range {
//			let x = range - i;
//            let _g = hash.get(&x);
//        }
//    });
//}
//
//#[bench]
//fn bench_rev_100000_get_trie(b: &mut Bencher) {
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
//            let _g = trie.get(&v[range-i][2..]);
//        }
//    });
//}
//
//
//#[bench]
//fn bench_rev_100000_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 100000;
//    for i in 0..range {
//        hash.insert(i as usize, i as usize);
//    }
//    b.iter(|| {
//        for i in 0..range {
//			let x = range - i;
//            let _g = hash.get(&x);
//        }
//    });
//}
//
//#[bench]
//fn bench_rev_1000000_get_trie(b: &mut Bencher) {
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
//            let _g = trie.get(&v[range-i][2..]);
//        }
//    });
//}
//
//
//#[bench]
//fn bench_rev_1000000_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 1000000;
//    for i in 0..range {
//        hash.insert(i as usize, i as usize);
//    }
//    b.iter(|| {
//        for i in 0..range {
//			let x = range - i;
//            let _g = hash.get(&x);
//        }
//    });
//}
//
//#[bench]
//fn bench_rev_10000000_get_trie(b: &mut Bencher) {
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
//            let _g = trie.get(&v[range-i][2..]);
//        }
//    });
//}
//
//
//#[bench]
//fn bench_rev_10000000_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 10000000;
//    for i in 0..range {
//        hash.insert(i as usize, i as usize);
//    }
//    b.iter(|| {
//        for i in 0..range {
//			let x = range - i;
//            let _g = hash.get(&x);
//        }
//    });
//}
//
