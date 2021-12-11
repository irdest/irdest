//! Capn Proto wrapper module

use crate::io::{proto::chunk as proto, wire::EncryptedChunk};
use crate::crypto::CryEngineHandle;

/// 
pub struct ChunkHeader {
    inner: proto::Header,
}

impl ChunkHeader {
    pub fn new(e: EncryptedChunk, _: ()) -> Self {
        todo!()
    }
}
