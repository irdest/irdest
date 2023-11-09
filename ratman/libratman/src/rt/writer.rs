use tokio::io::{AsyncWrite, AsyncWriteExt};

pub struct AsyncWriter<'buf, T: AsyncWriteExt>(&'buf [u8], &'buf mut T);
