extern crate cchamt;

use cchamt::{IndexStatus, Trie};

#[test]
fn test_new_trie() {
    let _trie = Trie::<()>::new();
}

#[test]
fn test_insert() {
    let mut base = Trie::new();

    base.insert((), &"0000000011111111".to_owned().into_bytes());
    base.insert((), &"0000000111111111".to_owned().into_bytes());
    base.insert((), &"0000001011111111".to_owned().into_bytes());
    base.insert((), &"0000001111111111".to_owned().into_bytes());
    base.insert((), &"0000010011111111".to_owned().into_bytes());
    base.insert((), &"0000010111111111".to_owned().into_bytes());
}

#[test]
fn test_index_base() {
    let mut base = Trie::new();

    base.insert((), &"1111111111111111".to_owned().into_bytes());
    base.insert((), &"0110111111111111".to_owned().into_bytes());

    assert_eq!(
        base.index_base(&"1111111111111111".to_owned().into_bytes()),
        IndexStatus::FullMatch
    );
    assert_eq!(
        base.index_base(&"1110111111111110".to_owned().into_bytes()),
        IndexStatus::NoMatch
    );
}

#[test]
fn test_get() {
    let mut base = Trie::new();

    base.insert("abc", &"1111111111111111".to_owned().into_bytes());
    base.insert("cde", &"0110111111111111".to_owned().into_bytes());

    let a = base.get(&"1111111111111111".to_owned().into_bytes());
    let ab = base.get(&"0110111111111111".to_owned().into_bytes());

    match a {
        Some(d) => assert_eq!(d, "abc"),
        _ => println!("find none"),
    }

    match ab {
        Some(d) => assert_ne!(d, "add"),
        _ => println!("find none"),
    }
}

#[test]
fn test_contain() {
    let mut base = Trie::new();
    let key1 = &"1111111111111111".to_owned().into_bytes();
    let key2 = &"0110111111111111".to_owned().into_bytes();

    base.insert("abc", key1);
    base.insert("cde", key2);

    assert!(base.contain(key1));
    assert!(base.contain(key2));
}

#[test]
fn test_update() {
    let mut base = Trie::new();
    let key1 = &"1111111111111111".to_owned().into_bytes();
    base.insert(1, key1);
    base.insert(2, key1);
    base.insert(1, key1);
    base.insert(1, key1);
    base.insert(2, key1);
    let result = base.get(key1);

    assert!(base.contain(key1));

    match result {
        Some(d) => assert_eq!(d, 2),
        _ => assert!(false),
    }
}

#[test]
fn test_delete() {
    let mut base = Trie::new();
    let key1 = &"1111111111111111".to_owned().into_bytes();
    base.insert(1, key1);
    base.delete_key(key1);

    assert!(!base.contain(key1));
}
