use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum AddressData {
    /// A full local address keypair
    ///
    /// The private key is encrypted with the associated ClientAuth token which
    /// is generated when the address is generated.  The public key is used as
    /// an index.
    Local(EncryptedKey),
    /// A remote address marker, where the db index is the public key/ address
    Remote,
}

/// Represents an encrypted address key
#[derive(Clone, Ord, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedKey {
    pub encrypted: Vec<u8>,
    pub nonce: [u8; 12],
}
