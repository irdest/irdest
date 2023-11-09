use crate::{types::chunk::Chunk, Result};
use std::{
    collections::VecDeque,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::{
    io::{self, AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader, ReadBuf},
    sync::mpsc::{channel, Receiver, Sender},
};

/// Reads chunks from a channel and
pub struct ChunkIter<const L: usize> {
    /// A sender and receiver for new chunks
    source: Receiver<Chunk<L>>,
    /// The current chunk that is being read from
    current: Option<Chunk<L>>,
    cursor: usize,
    /// Previous chunks that haven't been garbage collected yet
    _garbage: Vec<Chunk<L>>,
}

impl<const L: usize> ChunkIter<L> {
    pub fn new() -> (Sender<Chunk<L>>, ChunkIter<L>) {
        let (tx, rx) = channel(32);

        (
            tx,
            Self {
                source: rx,
                current: None,
                cursor: 0,
                _garbage: vec![],
            },
        )
    }

    pub async fn next_chunk(&mut self) {
        let current = self.source.recv().await;
        self.current = current;
        self.cursor = 0;
    }

    pub async fn read_current_chunk(&mut self, buf: &mut ReadBuf<'_>) {
        let current = self.current.as_mut().unwrap();
        current.read_with_buf(buf).await;
    }
}

impl<const L: usize> AsyncRead for ChunkIter<L> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        ctx: &mut Context,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        // Chunk sizes are chosen as such that one chunk can ALWAYS
        // produce at least one full block of data.  The exception is
        // at the tail of a message, where the last chunk MAY contain
        // less.
        match self.current {
            // If we are already reading from a chunk we construct a
            // future (pin it for future iterations) and poll it until
            // our buffer is satisfied.
            Some(ref mut chunk) => {
                let chunk_read = self.read_current_chunk(buf);
                tokio::pin!(chunk_read);
                chunk_read.as_mut().as_mut().poll(ctx).map(|_| Ok(()))
            }
            // If we don't currently have a current chunk we try to
            // get one from the channel via a local pinned future
            None => {
                let source_poll = self.next_chunk();
                tokio::pin!(source_poll);
                source_poll.as_mut().as_mut().poll(ctx).map(|_| Ok(()))
            }
        }
    }
}
