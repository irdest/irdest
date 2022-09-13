#![allow(warnings)]

extern crate chashmap;
extern crate core;
extern crate rand;
extern crate rayon;

mod cchamt;
mod hamt;
//mod bench;
mod allocator;
mod lockfree_cchamt;
//mod mutex_cchamt;
//mod rwlock_cchamt;

pub use allocator::Allocator;
pub use cchamt::ContiguousTrie;
pub use hamt::{IndexStatus, Trie, TrieData};
pub use lockfree_cchamt::LockfreeTrie;
//pub use mutex_cchamt::MutexContiguousTrie;
//pub use rwlock_cchamt::RwContiguousTrie;
