use crate::Result;
use std::sync::Arc;
use tokio::io::{AsyncWrite, AsyncWriteExt};

pub struct AsyncWriter<'buf, T: 'buf + AsyncWriteExt + Unpin> {
    pub buffer: &'buf [u8],
    pub writer: &'buf mut T,
}

impl<'buf, T: 'buf + AsyncWriteExt + Unpin> AsyncWriter<'buf, T> {
    pub fn new(buffer: &'buf [u8], writer: &'buf mut T) -> Self {
        Self { buffer, writer }
    }

    pub async fn write_buffer(mut self) -> Result<()> {
        let Self {
            buffer,
            ref mut writer,
        } = self;
        writer.write_all(&buffer).await?;
        Ok(())
    }
}
