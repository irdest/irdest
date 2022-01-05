//! Protobuf wrapper module

use protobuf::Message;

use crate::io::{
    error::Result,
    proto::encrypted as proto,
    wire::traits::{FromReader, ToWriter},
};
use std::io::{Read, Write};

/// A single encrypted piece of data
pub enum Encrypted<'r> {
    Owned(proto::Encrypted),
    Ref(&'r proto::Encrypted),
}

impl FromReader for Encrypted<'_> {
    fn new_from_bytes(buf: &Vec<u8>) -> Result<Self> {
        Ok(proto::Encrypted::parse_from_bytes(&buf).map(|i| Self::Owned(i))?)
    }
}

impl ToWriter for Encrypted<'_> {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(match self {
            Self::Owned(e) => e.write_to_bytes()?,
            Self::Ref(e) => e.write_to_bytes()?,
        })
    }
}

impl<'r> Encrypted<'r> {
    /// Create a new wrapper from data
    pub fn new(nonce: Vec<u8>, data: Vec<u8>) -> Self {
        let mut inner = proto::Encrypted::new();
        inner.set_version(crate::io::cfg::VERSION);
        inner.set_nonce(nonce);
        inner.set_data(data);
        Self::Owned(inner)
    }

    /// Create a new wrapper from an existing parsed inner type
    pub fn wrap(inner: &'r proto::Encrypted) -> Self {
        Self::Ref(inner)
    }

    /// Peel away the wrapper type for encoding
    pub fn peel(self) -> proto::Encrypted {
        match self {
            Self::Owned(inner) => inner,
            _ => unimplemented!(), // not supported
        }
    }

    /// Get access to the encoding version
    pub fn version(&self) -> u32 {
        match self {
            Self::Owned(i) => i.get_version(),
            Self::Ref(i) => i.get_version(),
        }
    }

    /// Get access to the inner nonce
    pub fn nonce(&self) -> &[u8] {
        match self {
            Self::Owned(i) => i.get_nonce(),
            Self::Ref(i) => i.get_nonce(),
        }
    }

    /// Get access to the inner encrypted section
    pub fn data(&self) -> &[u8] {
        match self {
            Self::Owned(i) => i.get_data(),
            Self::Ref(i) => i.get_data(),
        }
    }
}

/// Encrypted record index structure
///
/// This type has an encrypted header, as well as a list of chunks
/// accessible by it.
pub struct RecordIndex {
    inner: proto::RecordIndex,
}

impl RecordIndex {
    /// Create a new encrypted record index from an encrypted header and child list
    pub fn new(list: Encrypted) -> Self {
        let mut inner = proto::RecordIndex::new();
        inner.set_version(crate::io::cfg::VERSION);
        inner.set_list(list.peel());
        Self { inner }
    }

    /// Get access to the encrypted chunk list
    pub fn list(&self) -> Encrypted {
        Encrypted::wrap(self.inner.get_list())
    }
}

impl ToWriter for RecordIndex {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.inner.write_to_bytes()?)
    }
}
impl FromReader for RecordIndex {
    fn new_from_bytes(buf: &Vec<u8>) -> Result<Self> {
        Ok(proto::RecordIndex::parse_from_bytes(buf).map(|inner| Self { inner })?)
    }
}
