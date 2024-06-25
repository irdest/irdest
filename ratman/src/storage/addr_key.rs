use libratman::types::Address;
use serde::{Deserialize, Serialize};
use std::{
    ffi::CString,
    hash::{Hash, Hasher},
};
use twox_hash::XxHash64;

#[derive(Clone, Serialize, Deserialize)]
pub enum AddressData {
    /// A local address keypair
    ///
    /// The private key is encrypted with the associated ClientAuth token which
    /// is generated when the address is generated.  The public key is used as
    /// an index.
    Local(EncryptedKey, Option<CString>),
    /// A local namespace keypair
    ///
    /// The private key is encrypted with the router identity key.  This will
    /// change in future, probably.
    Space(EncryptedKey, Option<CString>),
    ///
    /// A remote address marker, where the db index is the public key/ address
    Remote,
}

/// Represents an encrypted address key
#[derive(Clone, Ord, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedKey {
    pub encrypted: Vec<u8>,
    pub nonce: [u8; 12],
}

#[allow(unused)] // requires a bit more refactoring
#[derive(Clone, Ord, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanAddress {
    inner: Address,
}

impl From<Address> for HumanAddress {
    fn from(inner: Address) -> Self {
        Self { inner }
    }
}

impl HumanAddress {
    #[allow(unused)] // see parent
    pub fn to_string(&self) -> String {
        let first = self.inner.as_bytes().iter().take(4).collect::<Vec<&u8>>();
        let id_str = self.inner.to_string();
        let first_str = id_str.split("-").next().unwrap();

        let mut hasher = XxHash64::default();
        first.as_slice().hash(&mut hasher);
        let hash_buf = hasher.finish().to_be_bytes();

        format!("{}::[{}]", first_str, libratman::hex::encode(&hash_buf))
    }
}
