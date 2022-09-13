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
//use cchamt::RwContiguousTrie;
//use std::sync::Arc;
//use std::thread;
//use std::time::{SystemTime, Duration};
//
//const NTHREAD: usize = 16;
//
//#[bench]
//fn bench_1000_insert_trie(b: &mut Bencher) {
//    // If i use b.iter(), this may face like infinite panics. Need fix.
//    // Here is a naive alternative.
//    // TODO: fix this
//
//    let mut time = Duration::new(0, 0);
//    let iter = 10;
//
//    for _ in 0..iter {
//        let trie = Arc::new(RwContiguousTrie::<usize>::new(32, 8));
//        let iter = 1000;
//
//        let mut thread_handle: Vec<thread::JoinHandle<_>> = vec![];
//        let step: usize = iter / NTHREAD;
//
//        let start = SystemTime::now();
//
//        for t_id in 0..NTHREAD {
//            let thread_trie = trie.clone();
//            let begin = t_id * step;
//            let end = (t_id + 1) * step;
//            thread_handle.push(thread::spawn(move || {
//                for i in begin..end {
//                    let str = binary_format!(i);
//                    let arr = str.to_owned().into_bytes();
//                    thread_trie.insert(i, &arr[2..]);
//                }
//            }));
//        }
//
//        for thread in thread_handle {
//            thread.join();
//        }
//
//        let end = SystemTime::now();
//        let since = end.duration_since(start).expect("Time went backwards");
//        time += since;
//    }
//
//    time /= iter;
//
//    println!("cctrie_1000_insert{:?}", time);
//    assert!(false);
//}
//
//#[bench]
//fn bench_1000_get_trie(b: &mut Bencher) {
//    let mut time = Duration::new(0, 0);
//    let iter = 10;
//
//    for _ in 0..iter {
//        let trie = Arc::new(RwContiguousTrie::<usize>::new(32, 8));
//        let iter = 1000;
//        for i in 0..iter {
//            let str = binary_format!(i);
//            let arr = str.to_owned().into_bytes();
//            trie.insert(i, &arr[2..]);
//        }
//
//        let start = SystemTime::now();
//        let mut thread_handle: Vec<thread::JoinHandle<_>> = vec![];
//        let step: usize = iter / NTHREAD;
//
//        for t_id in 0..NTHREAD {
//            let thread_trie = trie.clone();
//            let begin = t_id * step;
//            let end = (t_id + 1) * step;
//            thread_handle.push(thread::spawn(move || {
//                for i in begin..end {
//                    let str = binary_format!(i);
//                    let arr = str.to_owned().into_bytes();
//                    assert_eq!(thread_trie.get(&arr[2..]).unwrap(), i);
//                }
//            }));
//        }
//
//        for thread in thread_handle {
//            thread.join();
//        }
//        let end = SystemTime::now();
//        let since = end.duration_since(start).expect("Time went backwards");
//        time += since;
//    }
//
//    time /= iter;
//
//    println!("cctrie_1000_get{:?}", time);
//    assert!(false);
//}
//
//
//#[bench]
//fn bench_10000_insert_trie(b: &mut Bencher) {
//
//    let mut time = Duration::new(0, 0);
//    let iter = 10;
//
//    for _ in 0..iter {
//        let trie = Arc::new(RwContiguousTrie::<usize>::new(32, 8));
//        let iter = 10000;
//
//        let mut thread_handle: Vec<thread::JoinHandle<_>> = vec![];
//        let step: usize = iter / NTHREAD;
//
//        let start = SystemTime::now();
//
//        for t_id in 0..NTHREAD {
//            let thread_trie = trie.clone();
//            let begin = t_id * step;
//            let end = (t_id + 1) * step;
//            thread_handle.push(thread::spawn(move || {
//                for i in begin..end {
//                    let str = binary_format!(i);
//                    let arr = str.to_owned().into_bytes();
//                    thread_trie.insert(i, &arr[2..]);
//                }
//            }));
//        }
//
//        for thread in thread_handle {
//            thread.join();
//        }
//
//        let end = SystemTime::now();
//        let since = end.duration_since(start).expect("Time went backwards");
//        time += since;
//    }
//
//    time /= iter;
//
//    println!("cctrie_10000_insert{:?}", time);
//    assert!(false);
//}
//
//#[bench]
//fn bench_10000_get_trie(b: &mut Bencher) {
//    let mut time = Duration::new(0, 0);
//    let iter = 10;
//
//    for _ in 0..iter {
//        let trie = Arc::new(RwContiguousTrie::<usize>::new(32, 8));
//        let iter = 10000;
//        for i in 0..iter {
//            let str = binary_format!(i);
//            let arr = str.to_owned().into_bytes();
//            trie.insert(i, &arr[2..]);
//        }
//
//        let start = SystemTime::now();
//        let mut thread_handle: Vec<thread::JoinHandle<_>> = vec![];
//        let step: usize = iter / NTHREAD;
//
//        for t_id in 0..NTHREAD {
//            let thread_trie = trie.clone();
//            let begin = t_id * step;
//            let end = (t_id + 1) * step;
//            thread_handle.push(thread::spawn(move || {
//                for i in begin..end {
//                    let str = binary_format!(i);
//                    let arr = str.to_owned().into_bytes();
//                    assert_eq!(thread_trie.get(&arr[2..]).unwrap(), i);
//                }
//            }));
//        }
//
//        for thread in thread_handle {
//            thread.join();
//        }
//        let end = SystemTime::now();
//        let since = end.duration_since(start).expect("Time went backwards");
//        time += since;
//    }
//
//    time /= iter;
//
//    println!("cctrie_10000_get{:?}", time);
//    assert!(false);
//}
//
//
//#[bench]
//fn bench_100000_insert_trie(b: &mut Bencher) {
//    let mut time = Duration::new(0, 0);
//    let iter = 10;
//
//    for _ in 0..iter {
//        let trie = Arc::new(RwContiguousTrie::<usize>::new(32, 8));
//        let iter = 100000;
//
//        let mut thread_handle: Vec<thread::JoinHandle<_>> = vec![];
//        let step: usize = iter / NTHREAD;
//
//        let start = SystemTime::now();
//
//        for t_id in 0..NTHREAD {
//            let thread_trie = trie.clone();
//            let begin = t_id * step;
//            let end = (t_id + 1) * step;
//            thread_handle.push(thread::spawn(move || {
//                for i in begin..end {
//                    let str = binary_format!(i);
//                    let arr = str.to_owned().into_bytes();
//                    thread_trie.insert(i, &arr[2..]);
//                }
//            }));
//        }
//
//        for thread in thread_handle {
//            thread.join();
//        }
//
//        let end = SystemTime::now();
//        let since = end.duration_since(start).expect("Time went backwards");
//        time += since;
//    }
//
//    time /= iter;
//
//    println!("cctrie_100000_insert{:?}", time);
//    assert!(false);
//}
//
//#[bench]
//fn bench_100000_get_trie(b: &mut Bencher) {
//    let mut time = Duration::new(0, 0);
//    let iter = 10;
//
//    for _ in 0..iter {
//        let trie = Arc::new(RwContiguousTrie::<usize>::new(32, 8));
//        let iter = 100000;
//        for i in 0..iter {
//            let str = binary_format!(i);
//            let arr = str.to_owned().into_bytes();
//            trie.insert(i, &arr[2..]);
//        }
//
//        let start = SystemTime::now();
//        let mut thread_handle: Vec<thread::JoinHandle<_>> = vec![];
//        let step: usize = iter / NTHREAD;
//
//        for t_id in 0..NTHREAD {
//            let thread_trie = trie.clone();
//            let begin = t_id * step;
//            let end = (t_id + 1) * step;
//            thread_handle.push(thread::spawn(move || {
//                for i in begin..end {
//                    let str = binary_format!(i);
//                    let arr = str.to_owned().into_bytes();
//                    assert_eq!(thread_trie.get(&arr[2..]).unwrap(), i);
//                }
//            }));
//        }
//
//        for thread in thread_handle {
//            thread.join();
//        }
//        let end = SystemTime::now();
//        let since = end.duration_since(start).expect("Time went backwards");
//        time += since;
//    }
//
//    time /= iter;
//
//    println!("cctrie_100000_get{:?}", time);
//    assert!(false);
//}
//
//#[bench]
//fn bench_1000000_insert_trie(b: &mut Bencher) {
//    let mut time = Duration::new(0, 0);
//    let iter = 10;
//
//    for _ in 0..iter {
//        let trie = Arc::new(RwContiguousTrie::<usize>::new(32, 8));
//        let iter = 1000000;
//
//        let mut thread_handle: Vec<thread::JoinHandle<_>> = vec![];
//        let step: usize = iter / NTHREAD;
//
//        let start = SystemTime::now();
//
//        for t_id in 0..NTHREAD {
//            let thread_trie = trie.clone();
//            let begin = t_id * step;
//            let end = (t_id + 1) * step;
//            thread_handle.push(thread::spawn(move || {
//                for i in begin..end {
//                    let str = binary_format!(i);
//                    let arr = str.to_owned().into_bytes();
//                    thread_trie.insert(i, &arr[2..]);
//                }
//            }));
//        }
//
//        for thread in thread_handle {
//            thread.join();
//        }
//
//        let end = SystemTime::now();
//        let since = end.duration_since(start).expect("Time went backwards");
//        time += since;
//    }
//
//    time /= iter;
//
//    println!("cctrie_1000000_insert{:?}", time);
//    assert!(false);
//}
//
//#[bench]
//fn bench_1000000_get_trie(b: &mut Bencher) {
//    let mut time = Duration::new(0, 0);
//    let iter = 10;
//
//    for _ in 0..iter {
//        let trie = Arc::new(RwContiguousTrie::<usize>::new(32, 8));
//        let iter = 1000000;
//        for i in 0..iter {
//            let str = binary_format!(i);
//            let arr = str.to_owned().into_bytes();
//            trie.insert(i, &arr[2..]);
//        }
//
//        let start = SystemTime::now();
//        let mut thread_handle: Vec<thread::JoinHandle<_>> = vec![];
//        let step: usize = iter / NTHREAD;
//
//        for t_id in 0..NTHREAD {
//            let thread_trie = trie.clone();
//            let begin = t_id * step;
//            let end = (t_id + 1) * step;
//            thread_handle.push(thread::spawn(move || {
//                for i in begin..end {
//                    let str = binary_format!(i);
//                    let arr = str.to_owned().into_bytes();
//                    assert_eq!(thread_trie.get(&arr[2..]).unwrap(), i);
//                }
//            }));
//        }
//
//        for thread in thread_handle {
//            thread.join();
//        }
//        let end = SystemTime::now();
//        let since = end.duration_since(start).expect("Time went backwards");
//        time += since;
//    }
//
//    time /= iter;
//
//    println!("cctrie_1000000_get{:?}", time);
//    assert!(false);
//}
//
//#[bench]
//fn bench_10000000_insert_trie(b: &mut Bencher) {
//
//    let mut time = Duration::new(0, 0);
//    let iter = 10;
//
//    for _ in 0..iter {
//        let trie = Arc::new(RwContiguousTrie::<usize>::new(32, 8));
//        let iter = 10000000;
//
//        let mut thread_handle: Vec<thread::JoinHandle<_>> = vec![];
//        let step: usize = iter / NTHREAD;
//
//        let start = SystemTime::now();
//
//        for t_id in 0..NTHREAD {
//            let thread_trie = trie.clone();
//            let begin = t_id * step;
//            let end = (t_id + 1) * step;
//            thread_handle.push(thread::spawn(move || {
//                for i in begin..end {
//                    let str = binary_format!(i);
//                    let arr = str.to_owned().into_bytes();
//                    thread_trie.insert(i, &arr[2..]);
//                }
//            }));
//        }
//
//        for thread in thread_handle {
//            thread.join();
//        }
//
//        let end = SystemTime::now();
//        let since = end.duration_since(start).expect("Time went backwards");
//        time += since;
//    }
//
//    time /= iter;
//
//    println!("cctrie_10000000_insert{:?}", time);
//    assert!(false);
//}
//
//#[bench]
//fn bench_10000000_get_trie(b: &mut Bencher) {
//    let mut time = Duration::new(0, 0);
//    let iter = 10;
//
//    for _ in 0..iter {
//        let trie = Arc::new(RwContiguousTrie::<usize>::new(32, 8));
//        let iter = 10000000;
//        for i in 0..iter {
//            let str = binary_format!(i);
//            let arr = str.to_owned().into_bytes();
//            trie.insert(i, &arr[2..]);
//        }
//
//        let start = SystemTime::now();
//        let mut thread_handle: Vec<thread::JoinHandle<_>> = vec![];
//        let step: usize = iter / NTHREAD;
//
//        for t_id in 0..NTHREAD {
//            let thread_trie = trie.clone();
//            let begin = t_id * step;
//            let end = (t_id + 1) * step;
//            thread_handle.push(thread::spawn(move || {
//                for i in begin..end {
//                    let str = binary_format!(i);
//                    let arr = str.to_owned().into_bytes();
//                    assert_eq!(thread_trie.get(&arr[2..]).unwrap(), i);
//                }
//            }));
//        }
//
//        for thread in thread_handle {
//            thread.join();
//        }
//        let end = SystemTime::now();
//        let since = end.duration_since(start).expect("Time went backwards");
//        time += since;
//    }
//
//    time /= iter;
//
//    println!("cctrie_10000000_get{:?}", time);
//    assert!(false);
//}
//
//#[bench]
//fn bench_1000_get_hashmap(b: &mut Bencher) {
//    let mut time = Duration::new(0, 0);
//    let iter = 10;
//
//    for _ in 0..iter {
//        let mut hash = HashMap::new();
//        let iter = 1000;
//        for i in 0..iter {
//            hash.insert(i as usize, i as usize);
//        }
//
//        let start = SystemTime::now();
//        let mut thread_handle: Vec<thread::JoinHandle<_>> = vec![];
//        let step: usize = iter / NTHREAD;
//
//        for t_id in 0..NTHREAD {
//            let thread_hash = hash.clone();
//            let begin = t_id * step;
//            let end = (t_id + 1) * step;
//            thread_handle.push(thread::spawn(move || {
//                for i in begin..end {
//                    let _g = thread_hash.get(&i);
//                }
//            }));
//        }
//
//        for thread in thread_handle {
//            thread.join();
//        }
//        let end = SystemTime::now();
//        let since = end.duration_since(start).expect("Time went backwards");
//        time += since;
//    }
//
//    time /= iter;
//
//    println!("hashmap_1000_get{:?}", time);
//    assert!(false);
//}
//
//#[bench]
//fn bench_10000_get_hashmap(b: &mut Bencher) {
//    let mut time = Duration::new(0, 0);
//    let iter = 10;
//
//    for _ in 0..iter {
//        let mut hash = HashMap::new();
//        let iter = 10000;
//        for i in 0..iter {
//            hash.insert(i as usize, i as usize);
//        }
//
//        let start = SystemTime::now();
//        let mut thread_handle: Vec<thread::JoinHandle<_>> = vec![];
//        let step: usize = iter / NTHREAD;
//
//        for t_id in 0..NTHREAD {
//            let thread_hash = hash.clone();
//            let begin = t_id * step;
//            let end = (t_id + 1) * step;
//            thread_handle.push(thread::spawn(move || {
//                for i in begin..end {
//                    let _g = thread_hash.get(&i);
//                }
//            }));
//        }
//
//        for thread in thread_handle {
//            thread.join();
//        }
//        let end = SystemTime::now();
//        let since = end.duration_since(start).expect("Time went backwards");
//        time += since;
//    }
//
//    time /= iter;
//
//    println!("hashmap_10000_get{:?}", time);
//    assert!(false);
//}
//
//#[bench]
//fn bench_100000_get_hashmap(b: &mut Bencher) {
//    let mut time = Duration::new(0, 0);
//    let iter = 10;
//
//    for _ in 0..iter {
//        let mut hash = HashMap::new();
//        let iter = 100000;
//        for i in 0..iter {
//            hash.insert(i as usize, i as usize);
//        }
//
//        let start = SystemTime::now();
//        let mut thread_handle: Vec<thread::JoinHandle<_>> = vec![];
//        let step: usize = iter / NTHREAD;
//
//        for t_id in 0..NTHREAD {
//            let thread_hash = hash.clone();
//            let begin = t_id * step;
//            let end = (t_id + 1) * step;
//            thread_handle.push(thread::spawn(move || {
//                for i in begin..end {
//                    let _g = thread_hash.get(&i);
//                }
//            }));
//        }
//
//        for thread in thread_handle {
//            thread.join();
//        }
//        let end = SystemTime::now();
//        let since = end.duration_since(start).expect("Time went backwards");
//        time += since;
//    }
//
//    time /= iter;
//
//    println!("hashmap_100000_get{:?}", time);
//    assert!(false);
//}
//
//#[bench]
//fn bench_1000000_get_hashmap(b: &mut Bencher) {
//    let mut time = Duration::new(0, 0);
//    let iter = 10;
//
//    for _ in 0..iter {
//        let mut hash = HashMap::new();
//        let iter = 1000000;
//        for i in 0..iter {
//            hash.insert(i as usize, i as usize);
//        }
//
//        let start = SystemTime::now();
//        let mut thread_handle: Vec<thread::JoinHandle<_>> = vec![];
//        let step: usize = iter / NTHREAD;
//
//        for t_id in 0..NTHREAD {
//            let thread_hash = hash.clone();
//            let begin = t_id * step;
//            let end = (t_id + 1) * step;
//            thread_handle.push(thread::spawn(move || {
//                for i in begin..end {
//                    let _g = thread_hash.get(&i);
//                }
//            }));
//        }
//
//        for thread in thread_handle {
//            thread.join();
//        }
//        let end = SystemTime::now();
//        let since = end.duration_since(start).expect("Time went backwards");
//        time += since;
//    }
//
//    time /= iter;
//
//    println!("hashmap_1000000_get{:?}", time);
//    assert!(false);
//}
//
//#[bench]
//fn bench_10000000_get_hashmap(b: &mut Bencher) {
//    let mut time = Duration::new(0, 0);
//    let iter = 10;
//
//    for _ in 0..iter {
//        let mut hash = HashMap::new();
//        let iter = 10000000;
//        for i in 0..iter {
//            hash.insert(i as usize, i as usize);
//        }
//
//        let start = SystemTime::now();
//        let mut thread_handle: Vec<thread::JoinHandle<_>> = vec![];
//        let step: usize = iter / NTHREAD;
//
//        for t_id in 0..NTHREAD {
//            let thread_hash = hash.clone();
//            let begin = t_id * step;
//            let end = (t_id + 1) * step;
//            thread_handle.push(thread::spawn(move || {
//                for i in begin..end {
//                    let _g = thread_hash.get(&i);
//                }
//            }));
//        }
//
//        for thread in thread_handle {
//            thread.join();
//        }
//        let end = SystemTime::now();
//        let since = end.duration_since(start).expect("Time went backwards");
//        time += since;
//    }
//
//    time /= iter;
//
//    println!("hashmap_10000000_get{:?}", time);
//    assert!(false);
//}
