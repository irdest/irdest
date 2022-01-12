//! API encoding types for Ratman

mod error;
mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto_gen/mod.rs"));
}

pub mod api;
pub mod message;

pub use error::{Error, Result};
pub use ratman_identity::Identity;

use api::ApiMessage;
use async_std::{
    io::{Read, Write},
    prelude::*,
};
use byteorder::{BigEndian, ByteOrder};
use protobuf::Message;

/// First write the length as big-endian u64, then write the provided buffer
pub async fn write_with_length<T: Write + Unpin>(t: &mut T, buf: &Vec<u8>) -> Result<usize> {
    let mut len = vec![0; 8];
    BigEndian::write_u64(&mut len, buf.len() as u64);
    t.write_all(len.as_slice()).await?;
    t.write_all(buf.as_slice()).await?;
    Ok(len.len() + buf.len())
}

/// First read a big-endian u64, then read the number of bytes
pub async fn read_with_length<T: Read + Unpin>(r: &mut T) -> Result<Vec<u8>> {
    let mut len_buf = vec![0; 8];
    r.read_exact(&mut len_buf).await?;
    let len = BigEndian::read_u64(&len_buf);

    let mut vec = vec![0; len as usize]; // FIXME: this might break on 32bit systems
    r.read_exact(&mut vec).await?;
    Ok(vec)
}

/// Parse a single message from a reader stream
pub async fn parse_message<R: Read + Unpin>(r: &mut R) -> Result<ApiMessage> {
    let vec = read_with_length(r).await?;
    Ok(ApiMessage::parse_from_bytes(&vec)?)
}

/// Encode an ApiMessage into a binary payload you can then pass to
/// `write_with_length`
pub fn encode_message(msg: ApiMessage) -> Result<Vec<u8>> {
    let mut buf = vec![];
    msg.write_to_vec(&mut buf)?;
    Ok(buf)
}
