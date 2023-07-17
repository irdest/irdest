use libratman::types::Address;
use serde::{Deserialize, Serialize};

/// A local address on the network.
///
/// This structure is only used for local storage.
#[derive(Clone, Ord, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageAddress {
    pub(crate) id: Address,
    pub(crate) key: EncryptedKey,
}

impl StorageAddress {
    pub fn new(id: Address, bare_key: &[u8]) -> Self {
        Self {
            id,
            key: EncryptedKey::new(bare_key),
        }
    }
}

/// Represents an encrypted address key
#[derive(Clone, Ord, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedKey {
    inner: Vec<u8>,
}

impl EncryptedKey {
    fn new(encrypted_data: &[u8]) -> Self {
        Self {
            inner: encrypted_data.into(),
        }
    }

    /// Decrypt the key with some user secret
    pub fn decrypt(&self, _user_secret: &[u8]) -> Vec<u8> {
        todo!()
    }
}
