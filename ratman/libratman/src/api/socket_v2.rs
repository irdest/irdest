use crate::{
    chunk::Chunk,
    frame::{micro::MicroframeHeader, FrameGenerator, FrameParser},
    rt::{
        reader::{AsyncReader, AsyncVecReader, LengthReader},
        writer::{write_u32, AsyncWriter},
    },
    EncodingError, Result,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{tcp::OwnedReadHalf, TcpStream},
};

pub struct RawSocketHandle {
    stream: TcpStream,
    read_counter: usize,
}

pub async fn read_header(mut stream: &mut OwnedReadHalf) -> Result<MicroframeHeader> {
    let length = LengthReader::new(&mut stream).read_u32().await?;
    let frame_buffer = AsyncVecReader::new(length as usize, &mut stream)
        .read_to_vec()
        .await?;
    Ok(MicroframeHeader::parse(frame_buffer.as_slice())
        .map_err(Into::<EncodingError>::into)?
        .1?)
}

impl RawSocketHandle {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            read_counter: 0,
        }
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        self.stream.shutdown().await?;
        Ok(())
    }

    pub fn read_counter(&self) -> usize {
        self.read_counter
    }

    pub fn reset_counter(&mut self) {
        self.read_counter = 0;
    }

    /// Read a full microframe header and payload
    pub async fn read_microframe<T: FrameParser>(
        &mut self,
    ) -> Result<(MicroframeHeader, T::Output)> {
        let header = self.read_header().await?;
        let payload_buf = self.read_buffer(header.payload_size as usize).await?;
        let (_remainder, payload) =
            T::parse(payload_buf.as_slice()).map_err(|p| EncodingError::Parsing(p.to_string()))?;
        assert_eq!(_remainder.len(), 0);
        Ok((header, payload))
    }

    /// Read a length-prepended microframe header
    pub async fn read_header(&mut self) -> Result<MicroframeHeader> {
        let length = LengthReader::new(&mut self.stream).read_u32().await?;
        let frame_buffer = AsyncVecReader::new(length as usize, &mut self.stream)
            .read_to_vec()
            .await?;
        Ok(MicroframeHeader::parse(frame_buffer.as_slice())
            .map_err(Into::<EncodingError>::into)?
            .1?)
    }

    /// Read a constant number of bytes into an array
    pub async fn read_buffer_const<const L: usize>(&mut self) -> Result<[u8; L]> {
        let mut buf = AsyncReader::<'_, L, _>::new(&mut self.stream);
        buf.read_to_fill().await?;
        Ok(buf.consume())
    }

    /// Read a specific number of bytes into a buffer
    pub async fn read_buffer(&mut self, len: usize) -> Result<Vec<u8>> {
        AsyncVecReader::new(len, &mut self.stream)
            .read_to_vec()
            .await
    }

    /// Read to fill a pre-allocated array
    pub async fn read_into<const L: usize>(&mut self, buf: &mut [u8; L]) -> Result<()> {
        self.stream.read(buf).await?;
        Ok(())
    }

    /// Read a const size chunk payload
    pub async fn read_chunk<const L: usize>(&mut self) -> Result<Chunk<L>> {
        let chunk = AsyncReader::<'_, L, _>::read_to_chunk(&mut self.stream).await?;
        self.read_counter += chunk.1; // Increment the read count
        Ok(chunk)
    }

    /// Write a length-prepended microframe header
    pub async fn write_header(&mut self, frame: MicroframeHeader) -> Result<()> {
        let mut buf = vec![];
        frame.generate(&mut buf)?;

        // First write a u32 for the header length
        write_u32(
            &mut self.stream,
            buf.len()
                .try_into()
                .expect("failed to convert usize -> u32: buffer size too large to send"),
        )
        .await?;

        self.write_buffer(buf).await?;
        Ok(())
    }

    pub async fn write_buffer(&mut self, buf: Vec<u8>) -> Result<()> {
        AsyncWriter::new(buf.as_slice(), &mut self.stream)
            .write_buffer()
            .await?;
        Ok(())
    }

    /// Take both the header and a payload encoder and write both
    pub async fn write_microframe<T: FrameGenerator>(
        &mut self,
        mut header: MicroframeHeader,
        payload: T,
    ) -> Result<()> {
        let mut buf = vec![];
        payload.generate(&mut buf)?;
        header.payload_size = buf
            .len()
            .try_into()
            .expect("payload too large for microframe");
        self.write_header(header).await?;
        self.write_buffer(buf).await?;
        Ok(())
    }
}

#[tokio::test]
async fn test_socket_write_header() -> Result<()> {
    use crate::frame::micro::client_modes as cm;
    use tokio::{
        net::{TcpListener, TcpStream},
        spawn,
    };

    let l = TcpListener::bind("127.0.0.1:19000").await?;

    // Create a fake header to transfer
    let header = MicroframeHeader {
        modes: cm::make(cm::ADDR, cm::CREATE),
        auth: None,
        payload_size: 0,
    };

    let reference = header.clone();
    // Since we are testing the SOCKET here, not the runtime, it's ok
    // to just do "spawn" instead of "spawn_local"
    spawn(async move {
        if let Ok((stream, _)) = l.accept().await {
            let mut raw = RawSocketHandle::new(stream);
            let header = raw.read_header().await.unwrap();
            assert_eq!(header, reference);
        }
    });

    let stream = TcpStream::connect("127.0.0.1:19000").await.unwrap();
    let mut raw = RawSocketHandle::new(stream);

    raw.write_header(header).await.unwrap();
    Ok(())
}
