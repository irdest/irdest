// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! API encoding types for Ratman

#[cfg(feature = "proto")]
mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto_gen/mod.rs"));
}

#[cfg(feature = "proto")]
pub mod api;

// Export all the frame formats in their own module
pub mod frames;

mod error;
mod identifiers;
mod message;
mod recipient;
mod sequence_id;
mod timepair;

pub use crate::client::ClientError;
pub use error::{BlockError, EncodingError, NonfatalError, RatmanError, Result};
pub use identifiers::{address::Address, id::Id, ID_LEN};
pub use message::{ApiRecipient, Message};
pub use recipient::Recipient;
pub use sequence_id::SequenceIdV1;
pub use timepair::TimePair;

use async_std::{
    io::{Read, Write},
    prelude::*,
};
use byteorder::{BigEndian, ByteOrder};

#[cfg(feature = "proto")]
use {crate::types::api::ApiMessage, protobuf::Message as ProtoMessage};

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
#[cfg(feature = "proto")]
pub async fn parse_message<R: Read + Unpin>(r: &mut R) -> Result<ApiMessage> {
    let vec = read_with_length(r).await?;
    decode_message(&vec)
}

#[cfg(feature = "proto")]
#[inline]
pub fn decode_message(vec: &Vec<u8>) -> Result<ApiMessage> {
    Ok(ApiMessage::parse_from_bytes(vec)?)
}

/// Encode an ApiMessage into a binary payload you can then pass to
/// `write_with_length`
#[cfg(feature = "proto")]
#[inline]
pub fn encode_message(msg: ApiMessage) -> Result<Vec<u8>> {
    let mut buf = vec![];
    msg.write_to_vec(&mut buf)?;
    Ok(buf)
}
