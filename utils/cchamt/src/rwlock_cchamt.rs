/// Concurrent Cache Conscious Hash Trie using Read-Write Lock

use std::sync::{Arc, RwLock};
use std::thread;
use std::time::SystemTime;

pub trait TrieData: Clone + Copy + Eq + PartialEq {}

impl<T> TrieData for T where T: Clone + Copy + Eq + PartialEq {}

/// Private Functions for this module
/// compute the depth in the trie using the array index of trie.memory
// TODO bug here
#[inline(always)]
fn get_depth(key_group: usize, index: usize) -> usize {
    let mut depth = 0;
    let key_length = u32::pow(2, key_group as u32);
    let mut multitude = key_length;
    let mut compare = multitude;

    while index >= compare as usize {
        depth += 1;
        multitude *= key_length;
        compare += multitude;
    }
    depth
}

/// Core Data structure
#[derive(Debug)]
pub struct RwContiguousTrie<T: TrieData> {
    memory: RwLock<Vec<Option<SubTrie<T>>>>,
    key_length: usize,
    key_segment_size: usize,
}


#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SubTrie<T: TrieData> {
    pub data: Option<T>,
    depth: usize,
    children_offset: Option<usize>,    // the start position in allocator that place the array in hash trie
}

// Contiguous store all the nodes contiguous with the sequential order of key
impl<T: TrieData> RwContiguousTrie<T> {
    pub fn new(key_length: usize, key_segment_size: usize) -> Self {
        assert_eq!(key_length % key_segment_size, 0);

        let memory: RwLock<Vec<Option<SubTrie<T>>>>;
        // init with all nodes that is not leaf
        // length = summation of KEY_LEN^1 to KEY_LEN^(KEY_LEN/KEY_GROUP-1)
        {
            let mut nodes_length = 0;
            let array_length = usize::pow(2, key_segment_size as u32);
            let mut multitude = array_length;
            for _ in 0..(key_length / key_segment_size - 1) {
                nodes_length += multitude;
                multitude *= array_length;
            }
//            println!("nl {}", nodes_length);
            memory = RwLock::new(Vec::with_capacity(nodes_length));

            let mut this = memory.write().unwrap();
            for i in 0..nodes_length {
                (*this).push(Some(SubTrie {
                    data: None,
//                    depth: get_depth(key_segment_size as usize, i),
                    depth: 0,
                    children_offset: Some((i + 1) * array_length as usize),
                }));
//                println!("co {} {}", i, (i + 1) * array_length as usize);
            }
        }

        RwContiguousTrie {
            memory,
            key_length,
            key_segment_size,
        }
    }

    // return the index in the first <= 4 bits
// for instances: 0000 0000 -> 0
    #[inline(always)]
    fn compute_index(&self, key: &[u8]) -> usize {
        let mut id = 0;
        let length = if key.len() > self.key_segment_size { self.key_segment_size } else { key.len() };
        for i in 0..length {
            let temp = key[i] as usize - '0' as usize;
            id += temp << (length - i - 1);
        }
        return id as usize;
    }

    // key should be 1-1 mapping to self memory array
    #[inline(always)]
    fn key2index(&self, key: &[u8]) -> usize {
        let mut current_index = self.compute_index(key);
        let mut key_start = 0;
        let this = self.memory.read().unwrap();
        while (*this).len() > current_index && (*this)[current_index].is_some() {
//            println!("comp_index {} ci {} {}", self.compute_index(&key[key_start..]), current_index, self.memory.len());
            match &(*this)[current_index] {
                Some(a) => {
                    match a.children_offset {
                        Some(b) => {
                            key_start += self.key_segment_size;
                            current_index = b + self.compute_index(&key[key_start..]);
                        }
                        None => break,
                    }
                }
                None => break,
            }
        }
        current_index
    }

    pub fn insert(&self, value: T, key: &[u8]) {
        let current_index = self.key2index(key);
        let mut length = 0;
        {
            let this = self.memory.read().unwrap();
            length = (*this).len();
        }
//        println!("debug {} {}", current_index, self.memory.len());
        if current_index >= length {
            let push_amount = current_index - length + 1;
            {
                let mut this = self.memory.write().unwrap();
                for _ in 0..push_amount {
                    (*this).push(None);
                }
            }
        }

        {
            let this = self.memory.read().unwrap();
            if (*this)[current_index].is_some() {
                assert!(false);
            }
        }

        let mut this = self.memory.write().unwrap();
        (*this)[current_index] = Some(SubTrie {
            data: Some(value),
//            depth: get_depth(self.key_length, current_index),
            depth: 0,
            children_offset: None,
        });
    }

    #[inline(always)]
    pub fn contain(&self, key: &[u8]) -> bool {
        let current_index = self.key2index(key);
        let mut this = self.memory.read().unwrap();
        if (*this).len() <= current_index {
            return false;
        }
        match &(*this)[current_index] {
            Some(_) => {
                true
            }
            None => false,
        }
    }

    #[inline(always)]
    pub fn get(&self, key: &[u8]) -> Option<T> {
        let current_index = self.key2index(key);
        let mut this = self.memory.read().unwrap();
        if (*this).len() <= current_index {
            return None;
        }
        match &(*this)[current_index] {
            Some(a) => {
                a.data
            }
            None => None,
        }
    }
}

// TODO should change this to key_length+2, which is {:0key_length+2b}
#[macro_export]
macro_rules! binary_format {
    ($x:expr) => {
        format!("{:#034b}", $x)
    };
}

const NTHREAD: usize = 4;

fn main() {

    let trie = Arc::new(RwContiguousTrie::<usize>::new(32, 8));
    let iter = 100000;

    let mut thread_handle: Vec<thread::JoinHandle<_>> = vec![];
    let step: usize = iter / NTHREAD;

    let start = SystemTime::now();

    for t_id in 0..NTHREAD {
        let thread_trie = trie.clone();
        let begin = t_id * step;
        let end = (t_id + 1) * step;
        thread_handle.push(thread::spawn(move || {
            for i in begin..end {
                let str = binary_format!(i);
                let arr = str.to_owned().into_bytes();
                thread_trie.insert(i, &arr[2..]);
            }
        }));
    }

    for thread in thread_handle {
        thread.join();
    }

    let end = SystemTime::now();
    let since = end.duration_since(start).expect("Time went backwards");
    println!("cccctrie_insert{:?}", since);

    let mut thread_handle: Vec<thread::JoinHandle<_>> = vec![];

    let start = SystemTime::now();

    for tid in 0..NTHREAD {
        let thread_trie = trie.clone();
        let begin = tid * step;
        let end = (tid + 1) * step;

        thread_handle.push(thread::spawn(move || {
//            println!("{:?}", trie_arc.get(allocator_arc.clone().as_ref(), &"0000001111111111".to_owned().into_bytes()));
            for i in begin..end {
                let str = binary_format!(i);
                let arr = str.to_owned().into_bytes();
                assert_eq!(thread_trie.get(&arr[2..]).unwrap(), i);
            }
        }));
    }

    for thread in thread_handle {
        thread.join();
    }

    let end = SystemTime::now();
    let since = end.duration_since(start).expect("Time went backwards");
    println!("cccctrie_get{:?}", since);
}
