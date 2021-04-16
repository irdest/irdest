//! I/O utility module

use crate::{error::RpcResult, ENCODING_JSON};
use async_std::{net::TcpStream, prelude::*};
use byteorder::{BigEndian, ByteOrder};
use identity::Identity;
use serde::{de::DeserializeOwned, Serialize};

/// A message buffer to send or receive
#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Identity,
    pub to: String,
    pub from: String,
    pub data: Vec<u8>,
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Message {{ id: {}, to: {}, from: {}, data: {} }}",
            self.id,
            self.to,
            self.from,
            std::str::from_utf8(&self.data).unwrap_or("<unprintable>")
        )
    }
}

impl Message {
    /// Create a new message to an address
    pub fn to_addr(to: &str, from: &str, data: Vec<u8>) -> Self {
        Self {
            id: Identity::random(),
            to: to.into(),
            from: from.into(),
            data,
        }
    }

    /// Create a reply to a message ID
    pub fn reply(self, from: &str, data: Vec<u8>) -> Self {
        Self {
            to: self.from,
            from: from.into(),
            data,
            ..self
        }
    }
}

/// Read a framed message from a socket
pub async fn recv(s: &mut TcpStream) -> RpcResult<Message> {
    let mut len_buf = vec![0; 8];
    s.read_exact(&mut len_buf).await?;
    let len = BigEndian::read_u64(&len_buf);

    // Read the incoming message buffer
    let mut data = vec![0; len as usize];
    trace!("Reading {} byte message from stream", len);
    s.read_exact(&mut data).await?;

    // Take the encoding byte and match to deserialize
    let enc = data.remove(0);
    decode(enc, &data)
}

/// Send a message with frame
pub async fn send(s: &mut TcpStream, enc: u8, msg: &Message) -> RpcResult<()> {
    let mut payload = encode(enc, msg)?;

    // Add the encoding byte to length
    let len = payload.len() + 1;

    // Create big endian length buffer
    let mut buffer = vec![0; 8];
    BigEndian::write_u64(&mut buffer, len as u64);

    // Append the encoding byte
    buffer.append(&mut vec![enc]);

    // Append the data buffer
    buffer.append(&mut payload);

    // Write buffer to socket
    trace!("Writing {} bytes to stream", buffer.len());
    Ok(s.write_all(&buffer).await?)
}

/// A generic encoding utility
pub fn encode<S: Serialize>(enc: u8, msg: &S) -> RpcResult<Vec<u8>> {
    Ok(match enc {
        ENCODING_JSON => serde_json::to_string(msg).map(|s| s.into_bytes())?,
        _ => todo!(),
    })
}

pub fn decode<D: DeserializeOwned>(enc: u8, data: &Vec<u8>) -> RpcResult<D> {
    Ok(match enc {
        ENCODING_JSON => serde_json::from_str(std::str::from_utf8(data).unwrap())?,
        _ => todo!(), // Old broker won't support new encoding
    })
}
