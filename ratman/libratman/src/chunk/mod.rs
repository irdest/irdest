use crate::Result;
use bytes::{buf::BufMut, Buf};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{self, AsyncRead, AsyncReadExt, ReadBuf};

mod iter;

pub const CHUNK_1K: usize = 1024 /* KB */;
pub const CHUNK_32K: usize = 1024 * 32 /* KB */;
pub const CHUNK_256K: usize = 1024 * 256 /* KB */;
pub const CHUNK_1M: usize = 1024 * 1024 * 1 /* MB */;
pub const CHUNK_8M: usize = 1024 * 1024  * 8 /* MB */;
pub const CHUNK_64M: usize = 1024 * 1024  * 64 /* MB */;
pub const CHUNK_512M: usize = 1024 * 1024  * 512 /* MB */;

pub use self::iter::ChunkIter;

/// A const sized chunk of data and its "last position" marker
///
/// "Last position" is guaranteed to be less or equal to `L`
pub struct Chunk<const L: usize>(pub [u8; L], pub usize);

impl<const L: usize> Chunk<L> {
    /// Allocate an empty chunk on the stack
    pub fn alloc() -> Self {
        Self([0; L], 0)
    }

    pub async fn read_to_buf(&mut self, readbuf: &mut ReadBuf<'_>) -> Result<()> {
        readbuf.put(self);
        Ok(())
    }

    /// Read some amount of bytes into the chunk, then return how many
    pub async fn read_from_reader(&mut self, r: &mut (impl AsyncRead + Unpin)) -> Result<usize> {
        self.1 = r.read(&mut self.0).await?;
        Ok(self.1)
    }

    /// Take an async reader and fill this chunk with data from it
    ///
    /// When this function returns successfully you are guaranteed to
    /// have read a full chunk.
    pub async fn fill_from_reader(&mut self, r: &mut (impl AsyncRead + Unpin)) -> Result<()> {
        self.1 = r.read_exact(&mut self.0).await?;
        Ok(())
    }
}

// We implement this trait here to allow various async reader types
// (such as `ReadBuf`) to read from a chunk as a regular data buffer.
impl<const L: usize> Buf for Chunk<L> {
    fn remaining(&self) -> usize {
        L - self.1
    }
    fn advance(&mut self, cnt: usize) {
        self.1 += cnt;
    }
    fn chunk(&self) -> &[u8] {
        &self.0[self.1..]
    }
}

impl<const L: usize> AsyncRead for Chunk<L> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        ctx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let read = self.read_to_buf(buf);
        tokio::pin!(read);
        read.as_mut().as_mut().poll(ctx).map_err(Into::into)
    }
}
