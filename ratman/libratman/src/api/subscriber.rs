use crate::{
    api::RawSocketHandle,
    types::{Ident32, LetterheadV1},
    NonfatalError, RatmanError, Result,
};
use tokio::io::AsyncReadExt;

pub struct SubscriptionHandle {
    pub id: Ident32,
    pub(crate) curr_stream: Option<LetterheadV1>,
    pub(crate) read_from_stream: usize,
    pub(crate) socket: RawSocketHandle,
}

#[allow(unused)]
fn zero_buf(array: &mut [u8]) {
    unsafe {
        let p = array.as_mut_ptr();
        core::ptr::write_bytes(p, 0, array.len());
    }
}

impl SubscriptionHandle {
    pub fn peer_info(&mut self) -> String {
        let peer_addr = self.socket.stream().peer_addr().unwrap();

        format!("{}:{}", peer_addr.ip(), peer_addr.port())
    }

    pub fn sub_id(&self) -> Ident32 {
        self.id
    }

    /// Wait for a stream letterhead which indicates an incoming stream
    ///
    /// When calling this function before a previous stream has completed it
    /// will return a `NonfatalError::OngoingStream`, which indicates that the
    /// previous stream must be completed before starting a new one
    pub async fn wait_for_stream(&mut self) -> Result<LetterheadV1> {
        if self.curr_stream.is_some() {
            return Err(NonfatalError::OngoingStream.into());
        }

        let (_, letterhead) = self.socket.read_microframe::<LetterheadV1>().await?;
        match letterhead {
            Ok(lh) => {
                self.curr_stream = Some(lh.clone());
                self.read_from_stream = 0;
                Ok(lh)
            }
            err => err,
        }
    }

    /// Read from the stream to fill a buffer
    ///
    /// The `amount_read` will be filled after every read to indicate how much
    /// of the buffer was filled (or not).  If the provided buffer is larger
    /// than what remains in the stream it will not be filled to the end.  Stale
    /// data may remain beyond.
    ///
    /// When the buffer is full the return type indicates whether the stream has
    /// more data to read (`Some(())`) or if the application has reached the end
    /// of stream (`None`).  In this case it is safe to call `wait_for_stream()`
    /// again.  I/O errors are returned as `Err(_)`.
    pub async fn read_to_buf(
        &mut self,
        buf: &mut [u8],
        amount_read: &mut usize,
    ) -> Result<Option<()>> {
        *amount_read = 0;
        let lh = self
            .curr_stream
            .as_ref()
            .ok_or(RatmanError::Nonfatal(NonfatalError::NoStream))?;
        let bytes_left = lh.payload_length as usize - self.read_from_stream;

        if buf.len() > bytes_left {
            let mut local_buf = vec![0; bytes_left];
            self.socket.stream().read_exact(&mut local_buf).await?;
            buf[0..].copy_from_slice(&local_buf);
            *amount_read = bytes_left;

            self.read_from_stream = 0;
            self.curr_stream = None;
            Ok(None)
        } else {
            self.socket.stream().read_exact(buf).await?;
            *amount_read = buf.len();
            Ok(Some(()))
        }
    }
}
