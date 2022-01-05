//! Protobuf wrapper module

use crate::crypto::{CipherText, CryEngineHandle, CryReqPayload, CryRespPayload, ResponsePayload};
use crate::io::{
    error::Result,
    proto::chunk as proto,
    wire::traits::{FromEncrypted, FromReader, ToEncrypted, ToWriter},
};
use id::Identity;
use protobuf::Message;
use std::io::{Read, Write};
use tracing::callsite::Identifier;

/// Encrypted chunk header at the start of a chunk file
#[derive(Debug, PartialEq)]
pub struct ChunkHeader {
    inner: proto::Header,
}

impl ChunkHeader {
    pub(crate) fn new(max_len: u64) -> Self {
        let mut inner = proto::Header::new();
        inner.set_version(crate::io::cfg::VERSION);
        inner.set_maxLen(max_len);
        Self { inner }
    }

    /// Get the current usage (in bytes) of this chunk
    pub(crate) fn usage(&self) -> u64 {
        self.inner.get_usage()
    }

    /// Add a value to the current usage
    pub(crate) fn add_usage(&mut self, u: u64) {
        self.inner.set_usage(self.usage() + u);
    }
}

impl ToWriter for ChunkHeader {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.inner.write_to_bytes()?)
    }
}

impl FromReader for ChunkHeader {
    fn new_from_bytes(buf: &Vec<u8>) -> Result<Self> {
        Ok(proto::Header::parse_from_bytes(buf).map(|inner| Self { inner })?)
    }
}

impl ToEncrypted for ChunkHeader {}
impl FromEncrypted for ChunkHeader {}

/// Encrypted chunk list
pub struct ChunkList {
    inner: proto::ChunkList,
}

impl ChunkList {
    pub(crate) fn new() -> Self {
        let mut inner = proto::ChunkList::new();
        Self { inner }
    }

    /// Get the list of known chunks
    pub(crate) fn chunks(&self) -> Vec<Identity> {
        self.inner
            .chunks
            .iter()
            .map(|s| Identity::from_string(&s))
            .collect()
    }

    /// Set the inner list to a new list
    pub(crate) fn set_chunks(&mut self, list: &Vec<Identity>) {
        self.inner
            .set_chunks(list.iter().map(|id| id.to_string()).collect());
    }
}

impl ToWriter for ChunkList {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.inner.write_to_bytes()?)
    }
}

impl FromReader for ChunkList {
    fn new_from_bytes(buf: &Vec<u8>) -> Result<Self> {
        Ok(proto::ChunkList::parse_from_bytes(buf).map(|inner| Self { inner })?)
    }
}

impl ToEncrypted for ChunkList {}
impl FromEncrypted for ChunkList {}
