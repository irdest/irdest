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
    netmod::InMemoryEnvelope,
    types::{
        frames::{CarrierFrameHeader, FrameGenerator, FrameParser},
        Address, NonfatalError,
    },
    EncodingError, RatmanError, Result,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Read 8 bytes
#[inline]
pub(crate) async fn read_blocking(mut rx: &TcpStream) -> Result<InMemoryEnvelope> {
    let mut len_buf = [0; 4];
    rx.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf);
    // trace!("Reading {len} bytes from socket");

    if len > 4196 {
        warn!("Receiving a message larger than 4169 bytes.  This might be a DOS attempt..");
    }

    let mut buffer = vec![0; len as usize];
    rx.read_exact(&mut buffer).await?;

    // trace!("Read length buffer: {:?}", len_buf);
    // trace!("Read envelope buffer: {:?}", buffer);

    InMemoryEnvelope::parse_from_buffer(buffer)
}

/// Attempt to read from the socket and return NoData if not enough
/// data was present to be read.  YOU MUST NOT PANIC ON THIS ERROR
/// TYPE
pub(crate) async fn read(mut rx: &TcpStream) -> Result<InMemoryEnvelope> {
    let mut len_buf = [0; 4];
    if rx.peek(&mut len_buf).await? < 4 {
        // return Err(NonfatalError::NoData.into());
        return Err(EncodingError::NoData.into());
    }

    Ok(read_blocking(rx).await?)
}

pub(crate) async fn write(mut tx: &TcpStream, envelope: &InMemoryEnvelope) -> Result<()> {
    // trace!("Writing {} bytes to buffer", envelope.buffer.len());
    let mut len_buf = (envelope.buffer.len() as u32).to_be_bytes();
    // trace!("Writing length buffer: {:?}", len_buf);
    // trace!("Writing envelope buffer: {:?}", envelope.buffer);

    tx.write(&len_buf).await?;
    tx.write(&envelope.buffer).await?;
    Ok(())
}

mod modes {
    pub const HANDSHAKE_HELLO: u16 = 32;
    pub const HANDSHAKE_ACK: u16 = 33;
}

/// A simple handshake type to send across a newly created connection
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum Handshake {
    Hello { tt: PeerType, self_port: u16 },
    Ack { tt: PeerType },
}

impl Handshake {
    pub(crate) fn from_carrier(env: &InMemoryEnvelope) -> Result<Self> {
        match env.header.get_modes() {
            modes::HANDSHAKE_HELLO | modes::HANDSHAKE_ACK => {
                bincode::deserialize(env.get_payload_slice())
                    .map_err(|e| EncodingError::Parsing(e.to_string()).into())
            }
            _ => Err(NonfatalError::MismatchedEncodingTypes.into()),
        }
    }

    pub(crate) fn to_carrier(self) -> Result<InMemoryEnvelope> {
        let (mut payload, modes) = match self {
            ref hello @ Self::Hello { .. } => (
                bincode::serialize(hello).expect("failed to encode Handshake::Hello"),
                modes::HANDSHAKE_HELLO,
            ),
            ref ack @ Self::Ack { .. } => (
                bincode::serialize(ack).expect("failed to encode Handshake::Ack"),
                modes::HANDSHAKE_ACK,
            ),
        };

        InMemoryEnvelope::from_header_and_payload(
            CarrierFrameHeader::new_netmodproto_frame(
                modes,
                // todo: get router root key!
                Address::random(),
                payload.len() as u16,
            ),
            payload,
        )
    }
}

#[test]
fn encode_decode_handshake() {
    let hello = Handshake::Hello {
        tt: PeerType::Standard,
        self_port: 12,
    };

    let envelope = hello.clone().to_carrier().unwrap();
    println!(
        "Envelope payload length: {}",
        envelope.header.get_payload_length()
    );
    println!("Envelope payload: {:?}", envelope.get_payload_slice());
    println!("Full Envelope: {:?}", envelope.buffer);
    let hello2 = Handshake::from_carrier(&envelope).unwrap();

    assert_eq!(hello, hello2);
}

#[test]
fn decode_wellknown() {
    let raw = vec![
        1, 0, 32, 37, 84, 247, 144, 109, 131, 170, 164, 136, 214, 192, 145, 92, 9, 50, 64, 179,
        239, 68, 23, 201, 246, 171, 240, 209, 11, 114, 121, 208, 71, 90, 251, 0, 0, 0, 0, 10, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];

    let envelope = InMemoryEnvelope::parse_from_buffer(raw).unwrap();
    println!("Envelope header: {:?}", envelope.header);

    let payload = envelope.get_payload_slice();
    println!("Envelope payload is: {:?}", payload);
    let h: Handshake = bincode::deserialize(&payload).unwrap();
}
