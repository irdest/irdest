// SPDX-FileCopyrightText: 2019-2021 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2019-2021 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! UDP overlay protocol and framing

use libratman::{
    netmod::{InMemoryEnvelope, Target},
    types::{
        frames::{CarrierFrameHeader, FrameGenerator, FrameParser},
        Address, NonfatalError,
    },
    EncodingError, Result,
};
use serde::{Deserialize, Serialize};

/// A framing device to encapsulate the UDP overlay protocol
///
/// Multiple UDP endpoints need to be able to discover each other,
/// which is done with a simple protocol where announcements are
/// periodically sent via multicast to advertise an IP as a valid
/// endpoint.
///
/// These do not have to track what IDs are reachable via them, only
/// what internal ID they are represented by.  All other routing is
/// then done via Ratman and the netmod API which considers target
/// state.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Handshake {
    /// Announcing an endpoint via multicast
    Announce,
    /// Reply to an announce
    Reply,
}

pub(crate) mod modes {
    pub const HANDSHAKE_ANNOUNCE: u16 = 32;
    pub const HANDSHAKE_REPLY: u16 = 33;
}

impl Handshake {
    pub(crate) fn from_carrier(env: &InMemoryEnvelope) -> Result<Self> {
        match env.header.get_modes() {
            modes::HANDSHAKE_ANNOUNCE | modes::HANDSHAKE_REPLY => {
                bincode::deserialize(env.get_payload_slice())
                    .map_err(|e| EncodingError::Parsing(e.to_string()).into())
            }
            _ => Err(NonfatalError::MismatchedEncodingTypes.into()),
        }
    }

    pub(crate) fn to_carrier(self) -> Result<InMemoryEnvelope> {
        let (mut payload, modes) = match self {
            ref hello @ Self::Announce => (
                bincode::serialize(hello).expect("failed to encode Handshake::Hello"),
                modes::HANDSHAKE_ANNOUNCE,
            ),
            ref ack @ Self::Reply => (
                bincode::serialize(ack).expect("failed to encode Handshake::Ack"),
                modes::HANDSHAKE_REPLY,
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

/// A frame wrapped with the ID that it was targeted with
///
/// The ID can be resolved via the AddrTable to find out where to send
/// a payload
#[derive(Debug, Clone)]
pub(crate) struct MemoryEnvelopeExt(pub(crate) InMemoryEnvelope, pub(crate) Target);
