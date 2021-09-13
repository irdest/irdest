//! Wire serialiser formats

use bincode::{self, Result};
use serde::{de::DeserializeOwned, Serialize};

/// A generic trait for anything that can be serialised
pub(crate) trait Encode<T> {
    fn encode(&self) -> Result<Vec<u8>>;
}

/// A generic trait for anything that can be deserialised
pub(crate) trait Decode<T> {
    fn decode(data: &Vec<u8>) -> Result<T>;
}

// Blanket impl for anything than can be `Encode<T>`
impl<T> Encode<T> for T
where
    T: Serialize,
{
    fn encode(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
    }
}

impl<T> Decode<T> for T
where
    T: DeserializeOwned,
{
    fn decode(data: &Vec<u8>) -> Result<T> {
        bincode::deserialize(data)
    }
}

#[test]
fn encode_simple() {
    use {crate::utils::Id, serde::Deserialize};

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct TestStruct {
        id: Id,
    }

    let t = TestStruct { id: Id::random() };

    let enc = t.encode().unwrap();
    let dec = TestStruct::decode(&enc).unwrap();

    assert_eq!(dec, t);
}

#[test]
fn encode_skip() {
    use std::cell::Cell;
    use {crate::utils::Id, serde::Deserialize};

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct TestStruct {
        #[serde(skip)]
        _dont: Option<Cell<*const usize>>,
        id: Id,
    }

    let t = TestStruct {
        _dont: Some(Cell::new(0 as *const usize)), // NullPtr
        id: Id::random(),
    };

    let enc = t.encode().unwrap();
    let dec = TestStruct::decode(&enc).unwrap();

    assert_eq!(dec.id, t.id);
}
