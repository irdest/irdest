// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::PeerType;
use async_std::{
    io::{self, ReadExt, WriteExt},
    net::TcpStream,
};
use byteorder::ByteOrder;
use libratman::{
    error, netmod::InMemoryEnvelope, types::frames::ProtoCarrierFrameMeta, EncodingError, Result,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

// #[derive(Debug, thiserror::Error)]
// pub enum ProtoError {
//     #[error("tried reading from socket but no data was present")]
//     NoData,
//     #[error("underlying I/O failure: {}", 0)]
//     Io(io::Error),
// }

// impl From<io::Error> for ProtoError {
//     fn from(err: io::Error) -> Self {
//         Self::Io(err)
//     }
// }

#[inline]
pub(crate) async fn read_blocking(mut rx: &TcpStream) -> Result<InMemoryEnvelope> {
    let mut len_buf = [0; 8];
    rx.read_exact(&mut len_buf).await?;
    let len = u64::from_be_bytes(len_buf);

    if len > 4196 {
        warn!("Receiving a message larger than 4169 bytes.  This might be a DOS attempt..");
    }

    let mut buffer = vec![0; len as usize];
    rx.read_exact(&mut buffer).await?;

    Ok(ProtoCarrierFrameMeta::from_peek(&buffer).map(|meta| InMemoryEnvelope { meta, buffer })?)
}

/// Attempt to read from the socket and return NoData if not enough
/// data was present to be read.  YOU MUST NOT PANIC ON THIS ERROR
/// TYPE
pub(crate) async fn read(mut rx: &TcpStream) -> Result<InMemoryEnvelope> {
    let mut len_buf = [0; 8];
    if rx.peek(&mut len_buf).await? < 8 {
        return Err(EncodingError::NoData.into());
    }

    Ok(read_blocking(rx).await?)
}

pub(crate) async fn write(mut tx: &TcpStream, envelope: &InMemoryEnvelope) -> Result<()> {
    let mut len_buf = (envelope.buffer.len() as u64).to_be_bytes();
    tx.write(&len_buf).await?;
    tx.write(&envelope.buffer).await?;
    Ok(())
}

/// A simple handshake type to send across a newly created connection
#[derive(Serialize, Deserialize)]
pub(crate) enum Handshake {
    Hello { tt: PeerType, self_port: u16 },
    Ack { tt: PeerType },
}
