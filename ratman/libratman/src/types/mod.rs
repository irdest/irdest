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

mod error;
mod frame;
mod identifiers;
mod message;
mod seq;
mod timepair;

pub use crate::client::ClientError;
pub use error::{NonfatalError, RatmanError, Result};
pub use frame::Frame;
pub use identifiers::{address::Address, id::Id, ID_LEN};
pub use message::{Message, Recipient};
pub use seq::{SeqBuilder, SeqData, XxSignature};
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
    let mut len_buf: [u8; 8] = [0; 8];
    r.read_exact(&mut len_buf).await?;
    let len = BigEndian::read_u64(&len_buf);
    let mut vec = vec![0; len as usize]; // FIXME: this might break on 32bit systems
    r.read_exact(&mut vec).await?;
    Ok(vec)
}

/// First read a big-endian u64, then read the number of bytes
pub async fn read_with_length_nosey<T: Read + Unpin>(r: &mut T) -> Result<Vec<u8>> {
    let mut len_buf: [u8; 8] = [0; 8];
    println!("Waiting to read exactly {} bytes", len_buf.len());
    r.read_exact(&mut len_buf).await?;
    let len = BigEndian::read_u64(&len_buf);

    eprintln!(
        "Message length {} as: {}",
        len,
        len_buf
            .iter()
            .map(|i| format!("{:b}", i))
            .collect::<Vec<_>>()
            .join(" ")
    );

    let usize_len = dbg!((dbg!(len as u16)) as usize);

    println!("???????");
    // let mut vec = vec![0; 1024];

    // let mut vec = vec![0; 1024 * 16];
    let mut vec = vec![0; dbg!(usize_len)]; // FIXME: this might break on 32bit systems
    r.read_exact(&mut vec).await?;
    eprintln!("Message reads as: {}", String::from_utf8_lossy(&vec));
    Ok(vec)
}

/// Parse a single message from a reader stream
#[cfg(feature = "proto")]
pub async fn parse_message<R: Read + Unpin>(r: &mut R) -> Result<ApiMessage> {
    let vec = read_with_length(r).await?;
    decode_message(&vec)
}

/// Parse a single message from a reader stream
#[cfg(feature = "proto")]
pub async fn parse_message_nosey<R: Read + Unpin>(r: &mut R) -> Result<ApiMessage> {
    println!("Before read");
    let vec = read_with_length_nosey(r).await?;
    println!("Read message: {:?}", vec);
    println!("Read message: {}", String::from_utf8_lossy(vec.as_slice()));
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
