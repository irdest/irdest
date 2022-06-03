// SPDX-FileCopyrightText: 2019-2021 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2019-2021 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! UDP overlay protocol and framing

use netmod::{Frame, Target};
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
pub(crate) enum Envelope {
    /// Announcing an endpoint via multicast
    Announce,
    /// Reply to an announce
    Reply,
    /// A raw data frame
    Data(Vec<u8>),
}

impl Envelope {
    pub(crate) fn as_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub(crate) fn from_bytes(vec: &Vec<u8>) -> Self {
        bincode::deserialize(&vec).unwrap()
    }
}

/// A frame wrapped with the ID that it was targeted with
///
/// The ID can be resolved via the AddrTable to find out where to send
/// a payload
#[derive(Debug, Clone)]
pub(crate) struct FrameExt(pub(crate) Frame, pub(crate) Target);
