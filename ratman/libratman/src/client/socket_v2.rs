use crate::{
    microframe::{client_modes::*, MicroframeHeader},
    rt::{
        reader::{AsyncReader, AsyncVecReader, LengthReader},
        writer::AsyncWriter,
    },
    types::{
        frames::{
            parse::{self, take_u16_slice, FrameParser},
            FrameGenerator,
        },
        Chunk, ScheduleError,
    },
    EncodingError, RatmanError, Result,
};
use futures::TryFutureExt;
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc::Sender,
};

/// Api and state abstraction for a Ratman Api client
pub struct RatmanClient {}

/// Api and state abstraction, but on the side the router runs, with
/// some utilities the client may need.
pub struct RatmanSever {}

pub struct RawSocketHandle {
    reader: TcpStream,
    read_counter: usize,
    sender: Sender<()>, // ???
}

impl RawSocketHandle {
    pub fn new(reader: TcpStream, sender: Sender<()>) -> Self {
        Self {
            reader,
            sender,
            read_counter: 0,
        }
    }

    pub fn read_counter(&self) -> usize {
        self.read_counter
    }

    pub async fn read_header(&mut self) -> Result<MicroframeHeader> {
        let lr = LengthReader::<'_, 4, TcpStream>::new(&mut self.reader);
        let length = lr.read_u32().await?;
        let frame_buffer = AsyncVecReader::new(length as usize, &mut self.reader)
            .read_to_vec()
            .await?;
        Ok(MicroframeHeader::parse(frame_buffer.as_slice())
            .map_err(Into::<EncodingError>::into)?
            .1?)
    }

    pub async fn write_header(&mut self, frame: MicroframeHeader) -> Result<()> {
        let mut buf = vec![];
        frame.generate(&mut buf)?;
        AsyncWriter::new(buf.as_slice(), &mut self.reader)
            .write_buffer()
            .await?;
        Ok(())
    }

    /// Read a const size chunk payload
    pub async fn read_chunk<const L: usize>(&mut self) -> Result<Chunk<L>> {
        let chunk = AsyncReader::<'_, L, _>::read_to_chunk(&mut self.reader).await?;
        self.read_counter += chunk.1; // Increment the read count
        Ok(chunk)
    }
}
