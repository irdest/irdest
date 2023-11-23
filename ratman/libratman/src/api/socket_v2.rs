use crate::{
    chunk::Chunk,
    frame::{micro::MicroframeHeader, FrameGenerator, FrameParser},
    rt::{
        reader::{AsyncReader, AsyncVecReader, LengthReader},
        writer::AsyncWriter,
    },
    EncodingError, Result,
};
use tokio::{
    net::{tcp::OwnedReadHalf, TcpStream},
    sync::mpsc::Sender,
};

pub struct RawSocketHandle {
    reader: TcpStream,
    read_counter: usize,
    _sender: Sender<(MicroframeHeader, Vec<u8>)>,
}

pub async fn read_header(mut reader: &mut OwnedReadHalf) -> Result<MicroframeHeader> {
    let length = LengthReader::new(&mut reader).read_u32().await?;
    let frame_buffer = AsyncVecReader::new(length as usize, &mut reader)
        .read_to_vec()
        .await?;
    Ok(MicroframeHeader::parse(frame_buffer.as_slice())
        .map_err(Into::<EncodingError>::into)?
        .1?)
}

impl RawSocketHandle {
    pub fn new(reader: TcpStream, sender: Sender<(MicroframeHeader, Vec<u8>)>) -> Self {
        Self {
            reader,
            _sender: sender,
            read_counter: 0,
        }
    }

    pub fn read_counter(&self) -> usize {
        self.read_counter
    }

    pub fn reset_counter(&mut self) {
        self.read_counter = 0;
    }

    pub async fn read_header(&mut self) -> Result<MicroframeHeader> {
        let length = LengthReader::new(&mut self.reader).read_u32().await?;
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
        self.write_buffer(buf).await?;
        Ok(())
    }

    pub async fn write_buffer(&mut self, buf: Vec<u8>) -> Result<()> {
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
