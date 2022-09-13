/// The very basic hash trie implementation
/// This file is only for learning how to implement hash trie in Rust

pub trait TrieData: Clone + Copy + Eq + PartialEq {}

impl<T> TrieData for T where T: Clone + Copy + Eq + PartialEq {}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Trie<T: TrieData> {
    pub data: Option<T>,
    depth: u32,
    children: Vec<Option<Box<Trie<T>>>>,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum IndexStatus {
    FullMatch,
    StartingMatch,
    NoMatch,
}

const KEY_LEN: usize = 16;
const KEY_GROUP: usize = 4;

// index is the sum of binary in a group
fn compute_index(key: &[u8]) -> usize {
    let mut id = 0;
    let length = if key.len() > KEY_GROUP {
        KEY_GROUP
    } else {
        key.len()
    };
    for i in 0..length {
        let temp = key[i] as usize - '0' as usize;
        id += temp << i;
    }

    return id as usize;
}

impl<T: TrieData> Trie<T> {
    pub fn new() -> Self {
        let mut children = Vec::with_capacity(KEY_LEN);
        for i in 0..KEY_LEN {
            children.push(None);
        }
        Trie {
            data: None,
            depth: 0,
            children,
        }
    }

    pub fn depth(&self) -> u32 {
        self.depth
    }

    // insert a generic value by a key, and the key should be in binary format
    pub fn insert(&mut self, value: T, key: &[u8]) -> u32 {
        if key.len() == 0 {
            self.data = Some(value);
            return 1;
        } else {
            let index = compute_index(key);

            // if the trie has not been created, then create one
            if self.children[index].is_none() {
                // println!("create subtree");
                self.children[index] = Some(Box::new(Trie::new()));
            }
            let value = match key.len() {
                n if n >= KEY_GROUP => self.children[index]
                    .as_mut()
                    .map(|ref mut a| a.insert(value, &key[KEY_GROUP..]))
                    .unwrap_or(0),
                _ => 9999, // TODO value should be Option
            };
            self.depth += value;
            return value;
        }
    }

    // get value from key
    pub fn get(&self, key: &[u8]) -> Option<T> {
        let result = self.get_sub_trie(key);

        match result {
            Some(trie) => match trie.data {
                Some(data) => return Some(data),
                _ => return None,
            },
            _ => return None,
        }
    }

    // return true if the key exists, otherwise, return false
    pub fn contain(&self, key: &[u8]) -> bool {
        let trie_op = self.get_sub_trie(key);
        match trie_op {
            Some(trie) => {
                if trie.data == None {
                    return false;
                } else {
                    return true;
                }
            }
            _ => return false,
        }
    }

    pub fn index_base(&self, key: &[u8]) -> IndexStatus {
        if key.len() == 0 {
            self.data
                .map(|_| IndexStatus::FullMatch)
                .unwrap_or(IndexStatus::StartingMatch)
        } else {
            let index = compute_index(key);
            self.children[index]
                .as_ref()
                .map(|ref a| a.index_base(&key[KEY_GROUP..]))
                .unwrap_or(IndexStatus::NoMatch)
        }
    }

    pub fn get_sub_trie<'a>(&'a self, key: &[u8]) -> Option<&'a Trie<T>> {
        let index = compute_index(key);
        match key.len() {
            n if n >= KEY_GROUP => self.children[index]
                .as_ref()
                .and_then(|ref a| a.get_sub_trie(&key[KEY_GROUP..])),
            _ => Some(&self),
        }
    }

    // TODO delete the data in the trie found by the key
    pub fn delete_key(&mut self, key: &[u8]) {
        if key.len() == 0 {
            self.data = None;
        } else {
            let index = compute_index(key);

            if index >= KEY_GROUP {
                self.children[index]
                    .as_mut()
                    .map(|ref mut a| a.delete_key(&key[KEY_GROUP..]));
            }
        }
    }
}
