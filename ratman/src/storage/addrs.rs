use crate::Identity;
use serde::{Deserialize, Serialize};

/// A local address on the network.
///
/// This structure is only used for local storage.
#[derive(Ord, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalAddress {
    pub(crate) id: Identity,
    pub(crate) key: EncryptedKey,
}

impl LocalAddress {
    pub fn new(id: Identity, bare_key: &[u8]) -> Self {
        Self {
            id,
            key: EncryptedKey::new(bare_key),
        }
    }
}

/// Represents an encrypted address key
#[derive(Ord, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedKey {
    inner: Vec<u8>,
}

impl EncryptedKey {
    fn new(_bare: &[u8]) -> Self {
        Self { inner: vec![] }
    }
}
