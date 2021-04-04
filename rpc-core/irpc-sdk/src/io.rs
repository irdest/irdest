//! I/O utility module

use crate::{builders::_internal, error::RpcResult};
use async_std::{net::TcpStream, prelude::*};
use byteorder::{BigEndian, ByteOrder};
use identity::Identity;

/// A message buffer to send or receive
pub struct Message {
    pub id: Identity,
    pub to: String,
    pub from: String,
    pub data: Vec<u8>,
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
    pub fn reply(self, from: String, data: Vec<u8>) -> Self {
        Self {
            to: self.from,
            from,
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

    let mut data = vec![0; len as usize];
    trace!("Reading {} byte message from stream", len);
    s.read_exact(&mut data).await?;

    // Parse the carrier message type
    let (id, to, from, data) = _internal::from(data)?;
    Ok(Message { id, to, from, data })
}

/// Send a message with frame
pub async fn send(s: &mut TcpStream, msg: Message) -> RpcResult<()> {
    // Serialise into carrier message type
    let mut msg_buf = _internal::to(msg);

    let mut buffer = vec![0; 8];
    BigEndian::write_u64(&mut buffer, msg_buf.len() as u64);
    buffer.append(&mut msg_buf);

    trace!("Writing {} (+8) bytes to stream", msg_buf.len());
    Ok(s.write_all(&buffer).await?)
}

/// Allow any type to be written to a Capnproto message buffer
///
/// On a qrpc bus, a service exposes its API via an `-sdk` crate (for
/// example `libqaul-sdk`), paired with a `-type` crate (such as
/// `libqaul-types`).  In order to cut down on potential boilerplate
/// in converting between the networking types and the internal
/// library types this trait is meant to facilitate the
/// transformation.
pub trait WriteToBuf {
    /// Take an instance object and turn it into a packed byte buffer
    fn to_vec(&self) -> RpcResult<Vec<u8>>;
}
