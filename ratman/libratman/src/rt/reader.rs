use crate::{chunk::Chunk, Result};
use byteorder::{BigEndian, ByteOrder};
use tokio::io::{AsyncRead, AsyncReadExt};

use super::writer::AsyncWriter;

/// A structure capable of reading a certain length of data
///
/// Because we want to allow to read a chunk section by section the "consume"
/// function takes its own const generic type: S, which is runtime enforced to
/// be a valid position within L.  But it does mean that a user of this API
/// needs to:
///
/// - Stub out a lifetime that can usually be inferred - Provide a const generic
/// size for the inner chunk - Provide the type of reader
///
/// And then when consuming data from it:
///
/// - specify the consumption size
/// - Potentially twice, depending on usage
///
/// Still: it provides a UNIFORM and straight forward API for reading a chunk of
/// data and OWNING the data afterward. References can later be made to sub
/// chunks, but it's important that a chunk of memory can simply be owned in an
/// array.
pub struct AsyncReader<'r, const L: usize, T: AsyncRead + Unpin>(Chunk<L>, &'r mut T);

impl<'r, const L: usize, T: 'r + AsyncRead + Unpin> AsyncReader<'r, L, T> {
    pub fn new(reader: &'r mut T) -> Self {
        Self(Chunk::alloc(), reader)
    }

    /// Incrementally read
    pub async fn read_to_fill(&mut self) -> Result<()> {
        (&mut self.0).fill_from_reader(&mut self.1).await?;
        Ok(())
    }

    pub async fn fill_inplace(mut self) -> Result<AsyncReader<'r, L, T>> {
        (&mut self).read_to_fill().await?;
        Ok(self)
    }

    /// Consume S bytes of data from the chunk, advancing an external
    /// cursor along to keep track of how far into the buffer we've
    /// already read.
    ///
    /// This function will panic if the cursor overruns the length of
    /// the available data or if the cursor becomes negative
    pub fn consume_with_state<const S: usize>(&mut self, cursor: &mut usize) -> [u8; S] {
        assert!(*cursor < L);
        let mut buf: [u8; S] = [0; S];
        buf.copy_from_slice(&mut self.0 .0[*cursor..(*cursor + S)]);
        *cursor += S;
        buf
    }

    /// Consume S bytes of data from the start of the chunk and
    /// discard the rest
    pub fn consume<const S: usize>(mut self) -> [u8; S] {
        self.consume_with_state(&mut 0)
    }

    /// Read some amount of bytes, then return how many
    pub async fn read(&mut self) -> Result<usize> {
        (&mut self.0).read_from_reader(&mut self.1).await?;
        Ok(self.0 .1) // I am regretting these tuple structs...
    }

    /// Take an AsyncRead type, construct a temporary reader from it,
    /// then read until a chunk is filled and return that.
    pub async fn read_to_chunk(reader: &'r mut T) -> Result<Chunk<L>> {
        let mut reader = Self(Chunk::alloc(), reader);
        reader.read_to_fill().await?;
        Ok(reader.0)
    }
}

/// Works much like AsyncReader but with a dynamically allocated heap
/// buffer as a Vector instead of a constant chunk size
pub struct AsyncVecReader<'r, T: AsyncRead + Unpin>(Vec<u8>, &'r mut T);

impl<'r, T: 'r + AsyncRead + Unpin> AsyncVecReader<'r, T> {
    pub fn new(target_size: usize, reader: &'r mut T) -> Self {
        Self(vec![0; target_size], reader)
    }

    /// A much more simplified form of an in-memory buffer
    pub async fn read_to_vec(mut self) -> Result<Vec<u8>> {
        self.1.read_exact(&mut self.0).await?;
        Ok(self.0)
    }
}

/// Asynchronously read a length field
pub struct LengthReader<'r, T: 'r + AsyncRead + Unpin>(AsyncReader<'r, 4, T>);

impl<'r, T: 'r + AsyncRead + Unpin> LengthReader<'r, T> {
    pub fn new(r: &'r mut T) -> Self {
        Self(AsyncReader(Chunk::alloc(), r))
    }

    pub async fn read_u32(self) -> Result<u32> {
        Ok(BigEndian::read_u32(
            self.0.fill_inplace().await?.consume::<4>().as_slice(),
        ))
    }
}
