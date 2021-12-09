//! Capn Proto wrapper module

use protobuf::Message;

use crate::io::{error::Result, proto::encrypted as proto};
use std::io::{Read, Write};

/// A single encrypted piece of data
pub enum Encrypted<'r> {
    Owned(proto::Encrypted),
    Ref(&'r proto::Encrypted),
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
    /// Create a new wrapper from a reader
    pub fn from_reader<T: Read>(reader: &mut T) -> Result<Self> {
        let inner = proto::Encrypted::parse_from_reader(reader)?;
        Ok(Self::Owned(inner))
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

/// Basic encrypted chunk structure
///
/// Both the encrypted header and body can be accessed via this type
/// and should be turned into a [Chunk](crate::io::Chunk) instance by
/// consuming this type
pub struct EncryptedChunk {
    inner: proto::Chunk,
}

impl EncryptedChunk {
    /// Create a new encrypted chunk from a set of encrypted fields
    pub fn new(header: Encrypted, body: Encrypted) -> Result<Self> {
        let mut inner = proto::Chunk::new();
        inner.set_version(crate::io::cfg::VERSION);
        inner.set_header(header.peel());
        inner.set_body(body.peel());
        Ok(Self { inner })
    }
    /// Create a new wrapper from a reader
    pub fn from_reader<T: Read>(reader: &mut T) -> Result<Self> {
        let inner = proto::Chunk::parse_from_reader(reader)?;
        Ok(Self { inner })
    }
    /// Write length-prepended encoding to writer stream
    pub fn to_writer<T: Write>(&self, writer: &mut T) -> Result<()> {
        self.inner.write_length_delimited_to_writer(writer)?;
        Ok(())
    }
    /// Get access to the encrypted header
    pub fn header(&self) -> Encrypted {
        Encrypted::wrap(self.inner.get_header())
    }
    /// Get access to the encrypted body
    pub fn body(&self) -> Encrypted {
        Encrypted::wrap(self.inner.get_body())
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
    pub fn new(header: Encrypted, chunks: Encrypted) -> Self {
        let mut inner = proto::RecordIndex::new();
        inner.set_version(crate::io::cfg::VERSION);
        inner.set_header(header.peel());
        inner.set_chunks(chunks.peel());
        Self { inner }
    }
    /// Create a new wrapper from a reader
    pub fn from_reader<T: Read>(reader: &mut T) -> Result<Self> {
        let inner = proto::RecordIndex::parse_from_reader(reader)?;
        Ok(Self { inner })
    }
    /// Write length-prepended encoding to writer stream
    pub fn to_writer<T: Write>(&self, writer: &mut T) -> Result<()> {
        self.inner.write_length_delimited_to_writer(writer)?;
        Ok(())
    }
    /// Get access to the encrypted header
    pub fn header(&self) -> Encrypted {
        Encrypted::wrap(self.inner.get_header())
    }
    /// Get access to the chunk list
    pub fn chunks(&self) -> Encrypted {
        Encrypted::wrap(self.inner.get_chunks())
    }
}
