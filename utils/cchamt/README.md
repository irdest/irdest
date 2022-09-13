# concurrent-cache-conscious-hamt-in-rust

## Instructions
- you need rust nightly for this repository currently
    - [install with rustup](https://github.com/rust-lang-nursery/rustup.rs#working-with-nightly-rust)
- build with `cargo build`
- test with `cargo test`
- bench with `cargo bench`


## Source files
```
src
├── allocator.rs 		// allocator used by lockfree_cchamt for static packing entries
├── cchamt.rs 			// the simplest cache conscious implementation for showing the optimal case while reading sequentially
├── hamt.rs 			// plain hash trie implementation
├── lib.rs
├── lockfree_cchamt.rs 	        // An implementation that follows the concurrent trie paper + static data packing
├── mutex_cchamt.rs 	        // cchamt + mutex per hash trie
└── rwlock_cchamt.rs 	        // cchamt + rwrite lock per hash trie
```

## Raw Data
- in the dump directory

## TODO
- [X] Trie
- [X] Contiguous Trie
- [X] Benchmark
    - [X] Bench with different size (such as 1k, 10k, 1m, 10m...)
    - [X] Bench with different reading sequence (now is consecutively, should try others)
    - [X] Bench with different size on different amount of threads
- [ ] Concurrent by lock
    - [x] Mutex per trie
    - [x] RwLock per trie
    - [ ] Mutex per element
    - [ ] RwLock per element
- [X] Concurrent by lock-free
- [ ] Every kind of optimization
- [ ] Customized Allocator for cache conscious data structure
    - [X] Static data packing (clustering)
    - [ ] Dynamic data packing (http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.76.2169&rep=rep1&type=pdf)
        

## Benchmark
- See [here](https://github.com/chichunchen/concurrent-cache-conscious-hamt-in-rust/blob/bench/Benchmark.ipynb).
    - In the contiguous base, sequential order such as ascending or descending performs very well when we have more than
10^5 elements
    - While other point that worth to talk about is when we read the element from hashmap randomly, the official hashmap
performs pretty bad, and our contiguous cctrie seems just become a little bit worse.
