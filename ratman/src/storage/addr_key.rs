use serde::{Deserialize, Serialize};
use std::ffi::CString;

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
