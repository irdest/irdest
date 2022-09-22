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
    pub fn new(id: Identity, pw: &str, bare_key: &[u8]) -> Self {
        Self {
            id,
            key: EncryptedKey::new(pw, bare_key),
        }
    }
}

/// Represents an address key that's encrypted with a user's
/// passphrase
#[derive(Ord, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedKey {
    inner: Vec<u8>,
}

impl EncryptedKey {
    fn new(pw: &str, _bare: &[u8]) -> Self {
        Self { inner: vec![] }
    }

    pub fn decrypt(&self, pw: &str) -> Vec<u8> {
        todo!()
    }
}
