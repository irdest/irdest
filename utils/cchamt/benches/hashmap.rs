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
//
//#[bench]
//fn bench_10_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 10;
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
//
//#[bench]
//fn bench_100_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 100;
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
//
//#[bench]
//fn bench_1k_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 1000;
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
//
//#[bench]
//fn bench_10k_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 10000;
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
//fn bench_100k_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 100000;
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
//
//#[bench]
//fn bench_million_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 1000000;
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
//
//#[bench]
//fn bench_1000_million_get_hashmap(b: &mut Bencher) {
//    let mut hash = HashMap::new();
//    let range = 1000000000;
//    for i in 0..range {
//        hash.insert(i as usize, i as usize);
//    }
//    b.iter(|| {
//        for i in 0..range {
//            let _g = hash.get(&i);
//        }
//    });
//}
