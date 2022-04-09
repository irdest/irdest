use async_std::{
    io::{ReadExt, Result, WriteExt},
    net::TcpStream,
};
use byteorder::ByteOrder;
use serde::{de::DeserializeOwned, Serialize};

pub(crate) async fn read<T: DeserializeOwned>(mut rx: &TcpStream) -> Result<T> {
    let mut len_buf = [0; 8];
    rx.read_exact(&mut len_buf).await?;
    let len = byteorder::BigEndian::read_u64(&len_buf);

    let mut buf = Vec::with_capacity(len as usize);
    rx.read_exact(&mut buf).await?;
    Ok(bincode::deserialize(&buf).unwrap())
}

pub(crate) async fn write<T: Serialize>(mut tx: &TcpStream, f: &T) -> Result<()> {
    let mut encode = bincode::serialize(f).unwrap();
    let mut len_buf = Vec::with_capacity(8);
    byteorder::BigEndian::write_u64(&mut len_buf, encode.len() as u64);

    len_buf.append(&mut encode);
    tx.write(&len_buf).await?;
    Ok(())
}

/// A simple handshake type to send across a newly created connection
pub(crate) enum Handshake {
    Hello { cross: bool, port: u16 },
    Ack,
}
