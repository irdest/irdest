use crate::{
    microframe::{client_modes::*, MicroframeHeader},
    rt::reader::{AsyncReader, AsyncVecReader, LengthReader},
    types::{
        frames::parse::{self, take_u16_slice, FrameParser},
        ScheduleError,
    },
    EncodingError, RatmanError, Result,
};
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

struct RawSocketHandle(TcpStream, Sender<()>);

impl RawSocketHandle {
    async fn read_frame(&mut self) -> Result<MicroframeHeader> {
        let lr = LengthReader::<'_, 4, TcpStream>::new(&mut self.0);
        let length = lr.read_u32().await?;
        let frame_buffer = AsyncVecReader::new(length as usize, &mut self.0)
            .read_to_vec()
            .await?;
        Ok(MicroframeHeader::parse(frame_buffer.as_slice())
            .map_err(Into::<EncodingError>::into)?
            .1?)
    }

    /// Perform a single ping-pong transaction between Router and client
    pub async fn transaction(&mut self) -> Result<()> {
        // We start by writing the (version) HELLO o/ message
        self.0.write_all(&[1]).await?;

        // We then WAIT for a client to answer with a request
        let MicroframeHeader {
            modes,
            metadata,
            payload_size,
        } = tokio::time::timeout(
            // We give a 10 seconds timeout before we drop
            Duration::from_secs(10),
            async { self.read_frame().await },
        )
        .await
        .map_err(Into::<ScheduleError>::into)??;

        // Signal that everything went OK
        Ok(())
    }
}
