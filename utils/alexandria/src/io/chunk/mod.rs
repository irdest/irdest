//! Chunk based I/O

mod error;
pub(crate) use error::{ChunkError, Result};

mod reader;
pub use reader::*;
mod writer;
pub use writer::*;

use crate::{
    crypto::{
        pkcry::{PubKey, SecKey},
        CipherText, Nonce,
    },
    io::Encode,
};

/// Default length for NEW chunks
const DEFAULT_CHUNK_LEN: u64 = 256;

/// A fixed size chunk that can be loaded and decrypted
///
/// Alexandria record files are made up of chunks, each with a unique
/// nonce and parse utilities.
///
/// The layout of an alexandria data file is as follows
///
/// ```text
/// Chunk 1: [[Nonce] [chunk length] [chunk head] [chunk data]]
/// Chunk 2: [[Nonce] [chunk length] [chunk head] [chunk data]]
/// Chunk 3: [[Nonce] [chunk length] [chunk head] [chunk data]]
/// Chunk 4: [[Nonce] [chunk length] [chunk head] [chunk data]]
/// ....
/// ```
///
/// From a disk I/O perspective the chunks look as follows however:
///
/// ```text
/// Chunk 1: [[Nonce] [Gibberish]]
/// Chunk 2: [[Nonce] [Gibberish]]
/// Chunk 3: [[Nonce] [Gibberish]]
/// Chunk 4: [[Nonce] [Gibberish]]
/// ```
///
/// A chunk is decrypted via the [`pkcry`](crate::crypto::pkcry)
/// primitives, and then handed to data-format specific parsers.
///
/// If a data section exceeds the size of a chunk, the parsing must
/// throw a soft-error which is then caught by the
/// [`ChunkIO`](crate::io::chunk::ChunkIO) parser.
pub(crate) struct DataChunk {
    /// The nonce used for this chunk
    nonce: Vec<u8>,
    /// The full length of the chunk
    len: u64,
    /// The write-head of the chunk for appending
    head: u64,
    /// The chunk data
    data: Vec<u8>,
}

impl DataChunk {
    /// Create a new set of chunks from the given data
    ///
    /// If the data encoding is too big for a single chunk this
    /// constructor will automatically create multiple chunks that are
    /// zero-padded.
    ///
    /// **Note: This function should only be used when creating a new
    /// record.  It is extremely wasteful to not re-use existing
    /// chunks.  For all other cases use
    /// [`DataChunk::append`](DataChunk::append) instead!
    pub fn new<E: Encode>(e: E, pk: &PubKey, sk: &SecKey) -> Result<Vec<Self>> {
        let data = e.encode()?;

        // If the data is larger than the chunk length we are allowed
        // to create we must split it
        let chunks = if data.len() > DEFAULT_CHUNK_LEN as usize {
            data.chunks(DEFAULT_CHUNK_LEN as usize)
                .map(|chunk| chunk.to_vec())
                .collect()
        } else {
            vec![data]
        };

        Ok(chunks
            .into_iter()
            .map(|chunk| {
                let CipherText { nonce, data } = pk.seal(&chunk, &sk);
                Self {
                    nonce,
                    len: DEFAULT_CHUNK_LEN,
                    head: chunk.len() as u64,
                    data,
                }
            })
            .collect())
    }

    /// Append data to an existing data chunk
    ///
    /// If the chunk doesn't have enough free space for the encoded
    /// data it creates new zero-padded chunks.
    pub fn append<E: Encode>(mut self, e: E, pk: &PubKey, sk: &SecKey) -> Vec<Self> {
        todo!()
    }
}
