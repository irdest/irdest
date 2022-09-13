use allocator::Allocator;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::option::Option;
use std::ptr::null_mut;
use std::sync::{
    atomic::{AtomicPtr, AtomicU32, Ordering},
    Arc,
};
use std::thread;

pub trait TrieData: Clone + Copy + Eq + PartialEq {}

impl<T> TrieData for T where T: Clone + Copy + Eq + PartialEq {}

pub trait TrieKey: Clone + Copy + Eq + PartialEq + Hash {}

impl<T> TrieKey for T where T: Clone + Copy + Eq + PartialEq + Hash {}

type ANode<K, V> = Vec<AtomicPtr<Node<K, V>>>;

enum Node<K, V> {
    SNode {
        hash: u64,
        key: K,
        val: V,
        txn: AtomicPtr<Node<K, V>>,
    },
    ANode(ANode<K, V>),
    NoTxn,
    FSNode,
    FVNode,
    FNode {
        frozen: AtomicPtr<Node<K, V>>,
    },
    ENode {
        parent: AtomicPtr<Node<K, V>>,
        parentpos: u8,
        narrow: AtomicPtr<Node<K, V>>,
        hash: u64,
        level: u8,
        wide: AtomicPtr<Node<K, V>>,
    },
}

fn hash<T>(obj: T) -> u64
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}

const MAX_MISSES: u32 = 2048; // play with this

struct CacheLevel<K: TrieKey, V: TrieData> {
    parent: AtomicPtr<CacheLevel<K, V>>,
    pub nodes: Vec<AtomicPtr<Node<K, V>>>,
    pub misses: Vec<AtomicU32>,
}

impl<K: TrieKey, V: TrieData> CacheLevel<K, V> {
    pub fn new(level: u8, tfact: f64, ncpu: u8) -> Self {
        let len = 1 << level;
        let mut nodes = Vec::with_capacity(len);
        for i in 0..len {
            nodes[i] = AtomicPtr::new(null_mut());
        }
        let len = (tfact * ncpu as f64) as usize;
        let mut misses = Vec::with_capacity(len);
        for i in 0..len {
            misses[i] = AtomicU32::new(0);
        }
        CacheLevel {
            parent: AtomicPtr::new(null_mut()),
            nodes,
            misses,
        }
    }

    pub fn parent(&self) -> Option<&mut CacheLevel<K, V>> {
        let p = self.parent.load(Ordering::Relaxed);
        if p.is_null() {
            None
        } else {
            Some(unsafe { &mut *p })
        }
    }
}

struct Cache<K: TrieKey, V: TrieData> {
    level: AtomicPtr<CacheLevel<K, V>>,
}

impl<K: TrieKey, V: TrieData> Cache<K, V> {
    pub fn new() -> Self {
        Cache {
            level: AtomicPtr::new(null_mut()), // CacheLevel::new(0, 0.3, 8),
        }
    }
}

pub struct LockfreeTrie<K: TrieKey, V: TrieData> {
    root: AtomicPtr<Node<K, V>>,
    mem: Allocator<Node<K, V>>,
    cache: AtomicPtr<CacheLevel<K, V>>,
}

fn makeanode<K, V>(len: usize) -> ANode<K, V> {
    let mut a: ANode<K, V> = Vec::with_capacity(len);

    for i in 0..len {
        a.push(AtomicPtr::new(null_mut()));
    }
    a
}

/**
 * TODO: fix memory leaks and use atomic_ref or crossbeam crates
 */

impl<K: TrieKey, V: TrieData> LockfreeTrie<K, V> {
    pub fn new() -> Self {
        let mem = Allocator::new(1000000000);
        LockfreeTrie {
            root: AtomicPtr::new(mem.alloc(Node::ANode(makeanode(16)))),
            mem,
            cache: AtomicPtr::new(null_mut()),
        }
    }

    fn _freeze(mem: &Allocator<Node<K, V>>, nnode: &mut Node<K, V>) -> () {
        if let Node::ANode(ref cur) = nnode {
            let mut i = 0;
            while i < cur.len() {
                let node = &cur[i];
                let nodeptr = node.load(Ordering::Relaxed);
                let noderef = unsafe { &mut *nodeptr };

                i += 1;
                if nodeptr.is_null() {
                    if node.compare_and_swap(nodeptr, mem.alloc(Node::FVNode), Ordering::Relaxed)
                        != nodeptr
                    {
                        i -= 1;
                    }
                } else if let Node::SNode { ref txn, .. } = noderef {
                    let txnptr = txn.load(Ordering::Relaxed);
                    let txnref = unsafe { &mut *txnptr };
                    if let Node::NoTxn = txnref {
                        if txn.compare_and_swap(txnptr, mem.alloc(Node::FSNode), Ordering::Relaxed)
                            != txnptr
                        {
                            i -= 1;
                        }
                    } else if let Node::FSNode = txnref {
                    } else {
                        node.compare_and_swap(nodeptr, txnptr, Ordering::Relaxed);
                        i -= 1;
                    }
                } else if let Node::ANode(ref an) = noderef {
                    let fnode = mem.alloc(Node::FNode {
                        frozen: AtomicPtr::new(noderef),
                    });
                    node.compare_and_swap(nodeptr, fnode, Ordering::Relaxed);
                    i -= 1;
                } else if let Node::FNode { ref frozen } = noderef {
                    LockfreeTrie::_freeze(mem, unsafe { &mut *frozen.load(Ordering::Relaxed) });
                } else if let Node::ENode { .. } = noderef {
                    LockfreeTrie::_complete_expansion(mem, noderef);
                    i -= 1;
                }
            }
        } else {
            // this has never happened once, but just to be sure...
            panic!("CORRUPTION: nnode is not an ANode")
        }
    }

    fn _copy(mem: &Allocator<Node<K, V>>, an: &ANode<K, V>, wide: &mut Node<K, V>, lev: u64) -> () {
        for node in an {
            match unsafe { &*node.load(Ordering::Relaxed) } {
                Node::FNode { ref frozen } => {
                    let frzref = unsafe { &*frozen.load(Ordering::Relaxed) };
                    if let Node::ANode(ref an2) = frzref {
                        LockfreeTrie::_copy(mem, an2, wide, lev);
                    } else {
                        // this has never happened once, but just to be sure...
                        panic!("CORRUPTION: FNode contains non-ANode")
                    }
                }
                Node::SNode {
                    hash,
                    key,
                    val,
                    txn,
                } => {
                    LockfreeTrie::_insert(mem, *key, *val, *hash, lev as u8, wide, None);
                }
                _ => { /* ignore */ }
            }
        }
    }

    fn _complete_expansion(mem: &Allocator<Node<K, V>>, enode: &mut Node<K, V>) -> () {
        if let Node::ENode {
            ref parent,
            parentpos,
            ref narrow,
            level,
            wide: ref mut _wide,
            ..
        } = enode
        {
            let narrowptr = narrow.load(Ordering::Relaxed);
            LockfreeTrie::_freeze(mem, unsafe { &mut *narrowptr });
            let mut widenode = mem.alloc(Node::ANode(makeanode(16)));
            if let Node::ANode(ref an) = unsafe { &*narrowptr } {
                LockfreeTrie::_copy(mem, an, unsafe { &mut *widenode }, *level as u64);
            } else {
                // this has never happened once, but just to be sure...
                panic!("CORRUPTION: narrow is not an ANode")
            }
            if _wide.compare_and_swap(null_mut(), widenode, Ordering::Relaxed) != null_mut() {
                let _wideptr = _wide.load(Ordering::Relaxed);
                if let Node::ANode(ref an) = unsafe { &mut *_wideptr } {
                    widenode = unsafe { &mut *_wideptr };
                } else {
                    // this has never happened once, but just to be sure...
                    panic!("CORRUPTION: _wide is not an ANode")
                }
            }
            let parentref = unsafe { &*parent.load(Ordering::Relaxed) };
            if let Node::ANode(ref an) = parentref {
                let anptr = &an[*parentpos as usize];
                anptr.compare_and_swap(enode, widenode, Ordering::Relaxed);
            } else {
                // this has never happened once, but just to be sure...
                panic!("CORRUPTION: parent is not an ANode")
            }
        } else {
            // this has never happened once, but just to be sure...
            panic!("CORRUPTION: enode is not an ENode")
        }
    }

    fn _create_anode(
        mem: &Allocator<Node<K, V>>,
        old: Node<K, V>,
        sn: Node<K, V>,
        lev: u8,
    ) -> ANode<K, V> {
        let mut v = makeanode(4);

        if let Node::SNode { hash: h_old, .. } = old {
            let old_pos = (h_old >> lev) as usize & (v.len() - 1);
            if let Node::SNode { hash: h_sn, .. } = sn {
                let sn_pos = (h_sn >> lev) as usize & (v.len() - 1);
                if old_pos == sn_pos {
                    v[old_pos] = AtomicPtr::new(mem.alloc(Node::ANode(
                        LockfreeTrie::_create_anode(mem, old, sn, lev + 4),
                    )));
                } else {
                    v[old_pos] = AtomicPtr::new(mem.alloc(old));
                    v[sn_pos] = AtomicPtr::new(mem.alloc(sn));
                }
            } else {
                // this has never happened once, but just to be sure...
                panic!("CORRUPTION: expected SNode");
            }
        } else {
            // this has never happened once, but just to be sure...
            panic!("CORRUPTION: expected SNode");
        }
        return v;
    }

    fn _insert(
        mem: &Allocator<Node<K, V>>,
        key: K,
        val: V,
        h: u64,
        lev: u8,
        cur: &mut Node<K, V>,
        prev: Option<&mut Node<K, V>>,
    ) -> bool {
        if let Node::ANode(ref mut cur2) = cur {
            let pos = (h >> lev) as usize & (cur2.len() - 1);
            let old = &cur2[pos];
            let oldptr = old.load(Ordering::Relaxed);
            let oldref = unsafe { &mut *oldptr };

            if oldptr.is_null() {
                let sn = mem.alloc(Node::SNode {
                    hash: h,
                    key,
                    val,
                    txn: AtomicPtr::new(mem.alloc(Node::NoTxn)),
                });
                if old.compare_and_swap(oldptr, sn, Ordering::Relaxed) == oldptr {
                    true
                } else {
                    LockfreeTrie::_insert(mem, key, val, h, lev, cur, prev)
                }
            } else if let Node::ANode(ref mut an) = oldref {
                LockfreeTrie::_insert(mem, key, val, h, lev + 4, oldref, Some(cur))
            } else if let Node::SNode {
                hash: _hash,
                key: _key,
                val: _val,
                ref mut txn,
            } = oldref
            {
                let txnptr = txn.load(Ordering::Relaxed);
                let txnref = unsafe { &*txnptr };

                if let Node::NoTxn = txnref {
                    if *_key == key {
                        let sn = mem.alloc(Node::SNode {
                            hash: h,
                            key,
                            val,
                            txn: AtomicPtr::new(mem.alloc(Node::NoTxn)),
                        });
                        if txn.compare_and_swap(txnptr, sn, Ordering::Relaxed) == txnptr {
                            old.compare_and_swap(oldptr, sn, Ordering::Relaxed);
                            true
                        } else {
                            LockfreeTrie::_insert(mem, key, val, h, lev, cur, prev)
                        }
                    } else if cur2.len() == 4 {
                        if let Some(prevref) = prev {
                            let parent2 = AtomicPtr::new(prevref);
                            if let Node::ANode(ref mut prev2) = prevref {
                                let ppos = (h >> (lev - 4)) as usize & (prev2.len() - 1);
                                let prev2aptr = &prev2[ppos];
                                let en = mem.alloc(Node::ENode {
                                    parent: parent2,
                                    parentpos: ppos as u8,
                                    narrow: AtomicPtr::new(cur),
                                    hash: h,
                                    level: lev,
                                    wide: AtomicPtr::new(null_mut()),
                                });
                                if prev2aptr.compare_and_swap(cur, en, Ordering::Relaxed) == cur {
                                    LockfreeTrie::_complete_expansion(mem, unsafe { &mut *en });
                                    if let Node::ENode { ref wide, .. } = unsafe { &mut *en } {
                                        let wideref = unsafe { &mut *wide.load(Ordering::Relaxed) };
                                        LockfreeTrie::_insert(
                                            mem,
                                            key,
                                            val,
                                            h,
                                            lev,
                                            wideref,
                                            Some(prevref),
                                        )
                                    } else {
                                        // this has never happened once, but just to be sure...
                                        panic!("CORRUPTION: en is not an ENode")
                                    }
                                } else {
                                    LockfreeTrie::_insert(mem, key, val, h, lev, cur, Some(prevref))
                                }
                            } else {
                                // this has never happened once, but just to be sure...
                                panic!("CORRUPTION: prevref is not an ANode")
                            }
                        } else {
                            // this has never happened once, but just to be sure...
                            panic!("ERROR: prev is None")
                        }
                    } else {
                        let an = mem.alloc(Node::ANode(LockfreeTrie::_create_anode(
                            mem,
                            Node::SNode {
                                hash: *_hash,
                                key: *_key,
                                val: *_val,
                                txn: AtomicPtr::new(mem.alloc(Node::NoTxn)),
                            },
                            Node::SNode {
                                hash: h,
                                key,
                                val,
                                txn: AtomicPtr::new(mem.alloc(Node::NoTxn)),
                            },
                            lev + 4,
                        )));
                        if txn.compare_and_swap(txnptr, an, Ordering::Relaxed) == txnptr {
                            old.compare_and_swap(oldptr, an, Ordering::Relaxed);
                            true
                        } else {
                            LockfreeTrie::_insert(mem, key, val, h, lev, cur, prev)
                        }
                    }
                } else if let Node::FSNode = txnref {
                    false
                } else {
                    old.compare_and_swap(oldptr, txnptr, Ordering::Relaxed);
                    LockfreeTrie::_insert(mem, key, val, h, lev, cur, prev)
                }
            } else {
                if let Node::ENode { .. } = oldref {
                    LockfreeTrie::_complete_expansion(mem, oldref);
                }
                false
            }
        } else {
            // this has never happened once, but just to be sure...
            panic!("CORRUPTION: curref is not an ANode")
        }
    }

    pub fn insert(&mut self, key: K, val: V) -> bool {
        LockfreeTrie::_insert(
            &mut self.mem,
            key,
            val,
            hash(key),
            0,
            unsafe { &mut *self.root.load(Ordering::Relaxed) },
            None,
        ) || self.insert(key, val)
    }

    fn _inhabit<'a>(
        &'a self,
        cache: Option<&'a CacheLevel<K, V>>,
        nv: *mut Node<K, V>,
        hash: u64,
        lev: u8,
    ) -> () {
        if let Some(level) = cache {
            let length = level.nodes.capacity();
            let cache_level = (length - 1).trailing_zeros();
            if cache_level == lev.into() {
                let pos = hash as usize & (length - 1);
                (&level.nodes[pos]).store(nv, Ordering::Relaxed);
            }
        } else {
            if lev >= 12 {
                let clevel = Box::into_raw(Box::new(CacheLevel::new(lev, 0.3, 8)));
                let levptr = self.cache.load(Ordering::Relaxed);
                let oldptr = self
                    .cache
                    .compare_and_swap(levptr, clevel, Ordering::Relaxed);

                if !oldptr.is_null() {
                    let _b = unsafe { Box::from_raw(oldptr) };
                }

                self._inhabit(Some(unsafe { &*clevel }), nv, hash, lev);
            }
        }
    }

    fn _record_miss(&self) -> () {
        let mut counter_id: u64 = 0;
        let mut count: u32 = 0;
        let levptr = self.cache.load(Ordering::Relaxed);
        if !levptr.is_null() {
            let cn = unsafe { &*levptr };
            {
                counter_id = hash(thread::current().id()) % cn.misses.capacity() as u64;
                count = cn.misses[counter_id as usize].load(Ordering::Relaxed);
            }
            if count > MAX_MISSES {
                (&cn.misses[counter_id as usize]).store(0, Ordering::Relaxed);
                self._sample_and_adjust(Some(cn));
            } else {
                (&cn.misses[counter_id as usize]).store(count + 1, Ordering::Relaxed);
            }
        }
    }

    fn _sample_and_adjust<'a>(&'a self, cache: Option<&'a CacheLevel<K, V>>) -> () {
        if let Some(level) = cache {
            let histogram = self._sample_snodes_levels();
            let mut best = 0;
            for i in 0..histogram.len() {
                if histogram[i] > histogram[best] {
                    best = i;
                }
            }
            let prev = (level.nodes.capacity() as u64 - 1).trailing_zeros() as usize;
            if (histogram[best as usize] as f32) > histogram[prev >> 2] as f32 * 1.5 {
                self._adjust_level(best << 2);
            }
        }
    }

    fn _adjust_level(&self, level: usize) -> () {
        let clevel = Box::into_raw(Box::new(CacheLevel::new(level as u8, 0.3, 8)));
        let levptr = self.cache.load(Ordering::Relaxed);
        let oldptr = self
            .cache
            .compare_and_swap(levptr, clevel, Ordering::Relaxed);

        if !oldptr.is_null() {
            let _b = unsafe { Box::from_raw(oldptr) };
        }
    }

    fn _fill_hist(hist: &mut Vec<i32>, node: &Node<K, V>, level: u8) -> () {
        if let Node::ANode(ref an) = node {
            for v in an {
                let vptr = v.load(Ordering::Relaxed);

                if !vptr.is_null() {
                    let vref = unsafe { &*vptr };

                    if let Node::SNode { .. } = vref {
                        if level as usize >= hist.capacity() {
                            hist.resize_with((level as usize) << 1, Default::default);
                            hist[level as usize] = 0;
                        }
                        hist[level as usize] += 1;
                    } else if let Node::ANode(_) = vref {
                        LockfreeTrie::_fill_hist(hist, vref, level + 1);
                    }
                }
            }
        }
    }

    fn _sample_snodes_levels(&self) -> Vec<i32> {
        let mut hist = Vec::new();

        let root = unsafe { &*self.root.load(Ordering::Relaxed) };
        LockfreeTrie::_fill_hist(&mut hist, root, 0);

        hist
    }

    fn _lookup<'a>(
        &self,
        key: &K,
        h: u64,
        lev: u8,
        cur: &'a mut Node<K, V>,
        cache: Option<&'a CacheLevel<K, V>>,
        cache_lev: Option<u8>,
    ) -> Option<&'a V> {
        if let Node::ANode(ref cur2) = cur {
            let pos = (h >> lev) as usize & (cur2.len() - 1);
            let oldptr = (&cur2[pos]).load(Ordering::Relaxed);
            let oldref = unsafe { &mut *oldptr };

            if Some(lev) == cache_lev {
                self._inhabit(cache, cur, h, lev);
            }
            if oldptr.is_null() {
                None
            } else if let Node::FVNode = oldref {
                None
            } else if let Node::ANode(ref an) = oldref {
                self._lookup(key, h, lev + 4, oldref, cache, cache_lev)
            } else if let Node::SNode { key: _key, val, .. } = oldref {
                if let Some(clev) = cache_lev {
                    if !(lev >= clev || lev <= clev + 4) {
                        self._record_miss();
                    }
                    if lev + 4 == clev {
                        self._inhabit(cache, oldptr, h, lev + 4);
                    }
                }
                if *_key == *key {
                    Some(val)
                } else {
                    None
                }
            } else if let Node::ENode { narrow, .. } = oldref {
                self._lookup(
                    key,
                    h,
                    lev + 4,
                    unsafe { &mut *narrow.load(Ordering::Relaxed) },
                    cache,
                    cache_lev,
                )
            } else if let Node::FNode { frozen } = oldref {
                self._lookup(
                    key,
                    h,
                    lev + 4,
                    unsafe { &mut *frozen.load(Ordering::Relaxed) },
                    cache,
                    cache_lev,
                )
            } else {
                // this has never happened once, but just to be sure...
                panic!("CORRUPTION: oldref is not a valid node")
            }
        } else {
            // this has never happened once, but just to be sure...
            panic!("CORRUPTION: cur is not a pointer to ANode")
        }
    }

    /**
     * implemented as fastLookup()
     */
    pub fn lookup(&self, key: &K) -> Option<&V> {
        let h = hash(key);
        let mut cache_head_ptr = self.cache.load(Ordering::Relaxed);

        if cache_head_ptr.is_null() {
            self._lookup(
                key,
                hash(key),
                0,
                unsafe { &mut *self.root.load(Ordering::Relaxed) },
                None,
                None,
            )
        } else {
            let cache_head = unsafe { &*cache_head_ptr };
            let top_level = (cache_head.nodes.capacity() - 1).trailing_zeros();
            while !cache_head_ptr.is_null() {
                let cache_head = unsafe { &*cache_head_ptr };
                let pos = h & (cache_head.nodes.capacity() - 1) as u64;
                let cachee_ptr = cache_head.nodes[pos as usize].load(Ordering::Relaxed);
                let level = (cache_head.nodes.capacity() - 1).trailing_zeros();
                if !cachee_ptr.is_null() {
                    let cachee = unsafe { &*cachee_ptr };
                    if let Node::SNode {
                        txn,
                        key: _key,
                        val,
                        ..
                    } = cachee
                    {
                        if let Node::NoTxn = unsafe { &*txn.load(Ordering::Relaxed) } {
                            if *_key == *key {
                                return Some(val);
                            } else {
                                return None;
                            }
                        }
                    } else if let Node::ANode(ref an) = cachee {
                        let cpos = (h >> level) & (an.capacity() - 1) as u64;
                        let oldptr = an[cpos as usize].load(Ordering::Relaxed);

                        if !oldptr.is_null() {
                            if let Node::SNode { txn, .. } = unsafe { &*oldptr } {
                                if let Node::FSNode = unsafe { &*txn.load(Ordering::Relaxed) } {
                                    continue;
                                }
                            }
                        }
                        return self._lookup(
                            key,
                            hash(key),
                            0,
                            unsafe { &mut *self.root.load(Ordering::Relaxed) },
                            Some(cache_head),
                            Some(level as u8),
                        );
                    }
                }
                cache_head_ptr = cache_head.parent.load(Ordering::Relaxed);
            }
            self._lookup(
                key,
                hash(key),
                0,
                unsafe { &mut *self.root.load(Ordering::Relaxed) },
                None,
                Some(top_level as u8),
            )
        }
    }
}
