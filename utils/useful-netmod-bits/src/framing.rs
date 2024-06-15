// SPDX-FileCopyrightText: 2019-2021 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2019-2021 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! UDP overlay protocol and framing

use libratman::types::{Ident32, InMemoryEnvelope, Neighbour};
use serde::{Deserialize, Serialize};

/// A framing device to encapsulate the ethernet overlay protocol
///
/// Multiple ethernet endpoints need to be able to discover each other, which
/// is done with a simple protocol where announcements are periodically sent
/// via multicast to advertise an IP as a valid endpoint.
///
/// These do not have to track what IDs are reachable via them, only what
/// internal ID they are represented by.  All other routing is then done via
/// Ratman and the netmod API which considers target state.
///
/// This is the same as the UDP envelope. They are intentionally
/// familiar because the two protocols are simple and similar.
///
/// Factoring might be reasonable?
#[derive(Debug, Serialize, Deserialize)]
pub enum Envelope {
    /// Announcing an endpoint via multicast
    Announce(Ident32),
    /// Reply to an announce
    Reply(Ident32),
    /// A raw data frame
    Data(Vec<u8>),
}

impl Envelope {
    pub fn as_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn from_bytes(vec: &Vec<u8>) -> Self {
        bincode::deserialize(&vec).unwrap()
    }
}

/// A frame wrapped with the ID that it was targeted with
///
/// The ID can be resolved via the AddrTable to find out where to send
/// a payload
#[derive(Debug, Clone)]
pub struct FrameExt(pub InMemoryEnvelope, pub Neighbour);
