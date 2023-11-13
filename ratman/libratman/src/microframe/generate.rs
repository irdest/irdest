use crate::{types::frames::generate::any_as_u8_slice, Result};
use async_trait::async_trait;
use nom::IResult;
use tokio::io::AsyncRead;

/// Wraps around an input stream and provides an easy way to consume
/// variable length input.
pub struct AsyncInputHandle<'r, R: AsyncRead> {
    inner: &'r mut R,
}

#[async_trait]
pub trait AsyncFrameParser<'r, R: AsyncRead> {
    type Output;
    async fn parse(
        input: &'r AsyncInputHandle<R>,
    ) -> IResult<AsyncInputHandle<'r, R>, Self::Output>;
}

// pub trait MicroFrameGenerator {
//     fn generate(self, buf: &mut Vec<u8>) -> Result<()>;
// }

// impl<T: Sized> MicroFrameGenerator for T {
//     fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
//         buf.extend_from_slice(unsafe { any_as_u8_slice(&self) });
//         Ok(())
//     }
// }
