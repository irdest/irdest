use crate::Result;
use byteorder::{BigEndian, ByteOrder};
use tokio::io::AsyncWriteExt;

pub struct AsyncWriter<'buf, T: 'buf + AsyncWriteExt + Unpin> {
    pub buffer: &'buf [u8],
    pub writer: &'buf mut T,
}

impl<'buf, T: 'buf + AsyncWriteExt + Unpin> AsyncWriter<'buf, T> {
    pub fn new(buffer: &'buf [u8], writer: &'buf mut T) -> Self {
        Self { buffer, writer }
    }

    /// Write out the provided buffer to the given writer stream
    pub async fn write_buffer(mut self) -> Result<()> {
        let Self {
            buffer,
            ref mut writer,
        } = self;
        writer.write_all(&buffer).await?;
        Ok(())
    }
}

pub async fn write_u32<'r, T: 'r + AsyncWriteExt + Unpin>(w: &'r mut T, len: u32) -> Result<()> {
    let mut buf = vec![0; 4];
    BigEndian::write_u32(buf.as_mut_slice(), len);

    let writer = AsyncWriter::new(buf.as_slice(), w);
    writer.write_buffer().await
}
