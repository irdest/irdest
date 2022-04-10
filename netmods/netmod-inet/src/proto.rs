use async_std::{
    io::{self, ReadExt, WriteExt},
    net::TcpStream,
};
use byteorder::ByteOrder;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::PeerType;

#[derive(Debug, thiserror::Error)]
pub enum ProtoError {
    #[error("tried reading from socket but no data was present")]
    NoData,
    #[error("underlying I/O failure: {}", 0)]
    Io(io::Error),
}

impl From<io::Error> for ProtoError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

#[inline]
pub(crate) async fn read_blocking<T: DeserializeOwned>(mut rx: &TcpStream) -> Result<T, io::Error> {
    let mut len_buf = [0; 8];
    rx.read_exact(&mut len_buf).await?;
    let len = byteorder::BigEndian::read_u64(&len_buf);

    if len > 4196 {
        warn!("Receiving a message larger than 4169 bytes.  This might be a DOS attempt..");
    }

    let mut buf = Vec::with_capacity(len as usize);
    rx.read_exact(&mut buf).await?;
    Ok(bincode::deserialize(&buf).unwrap())
}

/// Attempt to read from the socket and return NoData if not enough
/// data was present to be read.  YOU MUST NOT PANIC ON THIS ERROR
/// TYPE
pub(crate) async fn read<T: DeserializeOwned>(mut rx: &TcpStream) -> Result<T, ProtoError> {
    let mut len_buf = [0; 8];
    if rx.peek(&mut len_buf).await? < 8 {
        return Err(ProtoError::NoData);
    }

    Ok(read_blocking(rx).await?)
}

pub(crate) async fn write<T: Serialize>(mut tx: &TcpStream, f: &T) -> Result<(), io::Error> {
    let mut encode = bincode::serialize(f).unwrap();
    let mut len_buf = Vec::with_capacity(8);
    byteorder::BigEndian::write_u64(&mut len_buf, encode.len() as u64);

    len_buf.append(&mut encode);
    tx.write(&len_buf).await?;
    Ok(())
}

/// A simple handshake type to send across a newly created connection
#[derive(Serialize, Deserialize)]
pub(crate) enum Handshake {
    Hello { tt: PeerType, self_port: u16 },
    Ack { tt: PeerType },
}
