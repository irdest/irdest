use crate::{types::Chunk, Result};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncReadExt};

/// A structure capable of reading a certain length of data
pub struct AsyncReader<const L: usize, T: AsyncRead + Unpin>(Chunk<L>, T);

impl<const L: usize, T: AsyncRead + Unpin> AsyncReader<L, T> {
    /// Incrementally read
    pub async fn read_to_fill(&mut self) -> Result<()> {
        (&mut self.0).fill_with_reader(&mut self.1).await?;
        Ok(())
    }

    /// Read some amount of bytes, then return how many
    pub async fn read(&mut self) -> Result<usize> {
        (&mut self.0).read(&mut self.1).await?;
        Ok(self.0.1) // I am regretting these tuple structs...
    }

    /// Take an AsyncRead type, construct a temporary reader from it,
    /// then read until a chunk is filled and return that.
    pub async fn read_to_chunk(reader: T) -> Result<Chunk<L>> {
        let mut reader = Self(Chunk::alloc(), reader);
        reader.read_to_fill().await?;
        Ok(reader.0)
    }
}
