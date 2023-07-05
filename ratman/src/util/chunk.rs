use async_std::stream::Stream;
use std::{
    io::Read,
    pin::Pin,
    task::{Context, Poll},
};

/// An in-line data chunk which can represent either a 1k or 32k data
/// chunk
pub enum DataChunk {
    K1([u8; 1024]),
    K32([u8; 1024 * 32]),
}

impl DataChunk {
    /// Initialise a data chunk at either 1K or 32K.
    pub fn with_size(size: usize, data: Vec<u8>) -> Self {
        match size {
            1024 => {
                let mut buf = [0 as u8; 1024];
                buf.copy_from_slice(data.as_slice());
                Self::K1(buf)
            }
            32_768 => {
                let mut buf = [0 as u8; 32_768];
                buf.copy_from_slice(data.as_slice());
                Self::K32(buf)
            }
            _ => unreachable!(),
        }
    }

    /// Get mutable access to the underlying data chunk buffer
    pub fn as_mut(&mut self) -> &mut [u8] {
        match self {
            Self::K1(buf) => buf.as_mut(),
            Self::K32(buf) => buf.as_mut(),
        }
    }
}

// /// Turn data into chunks
// pub struct Chunkotron<R: Read> {
//     read: R
// }

// impl<R: Read> Chunkotron<R> {

// }

// /// Reads from a TcpStream and turns this into a stream API
// pub struct TcpReader {
//     inner: TcpStream,
//     chunk_size: usize,
// }

// impl Stream for TcpReader {
//     type Item = DataChunk;

//     fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
//         let chunk = DataChunk::with_size(self.chunk_size);

//         todo!()
//     }
// }

/// Read from a stream with a pre-configured chunk size
pub struct ChunkStream<S>
where
    S: Stream<Item = u8> + Unpin,
{
    chunk_size: usize,
    stream: S,
}

impl<S> ChunkStream<S>
where
    S: Stream<Item = u8> + Unpin,
{
    pub fn new(chunk_size: usize, stream: S) -> Self {
        Self { chunk_size, stream }
    }
}

impl<S> Stream for ChunkStream<S>
where
    S: Stream<Item = u8> + Unpin,
{
    type Item = Vec<u8>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let _hint = self.stream.size_hint();
        // let v = self.stream.take(self.chunk_size).poll_next(cx);

        Poll::Pending
    }
}

// First, the struct:

/// A stream which counts from one to five
struct Counter {
    count: usize,
}

// we want our count to start at one, so let's add a new() method to help.
// This isn't strictly necessary, but is convenient. Note that we start
// `count` at zero, we'll see why in `next()`'s implementation below.
impl Counter {
    fn new() -> Counter {
        Counter { count: 0 }
    }
}

// Then, we implement `Stream` for our `Counter`:

impl Stream for Counter {
    // we will be counting with usize
    type Item = usize;

    // poll_next() is the only required method
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Increment our count. This is why we started at zero.
        self.count += 1;

        // Check to see if we've finished counting or not.
        if self.count < 6 {
            Poll::Ready(Some(self.count))
        } else {
            Poll::Ready(None)
        }
    }
}

// // And now we can use it!
// let mut counter = Counter::new();

// let x = counter.next().await.unwrap();
// println!("{}", x);

// let x = counter.next().await.unwrap();
// println!("{}", x);

// let x = counter.next().await.unwrap();
// println!("{}", x);

// let x = counter.next().await.unwrap();
// println!("{}", x);

// let x = counter.next().await.unwrap();
// println!("{}", x);
