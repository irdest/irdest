// SPDX-FileCopyrightText: 2019-2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::types::{Address, Id, Recipient, SeqBuilder, SeqData, ID_LEN};
use serde::{Deserialize, Serialize};

/// A sequence of data, represented by a single network packet
///
/// Because a `Frame` is usually created in a sequence, the
/// constructors assume chainable operations, such as a `Vec<Frame>`
/// can be returned with all sequence ID information correctly setup.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[deprecated]
pub struct Frame {
    /// Sender information
    pub sender: Address,
    /// Recipient information
    pub recipient: Recipient,
    /// Data sequence identifiers
    pub seq: SeqData,
    /// Raw data payload
    pub payload: Vec<u8>,
}

impl Frame {
    /// Produce a new dummy frame that sends nonsense data from nowhere to everyone.
    pub fn dummy() -> Self {
        SeqBuilder::new(
            Address::from_bytes(&[0; ID_LEN]),
            Recipient::Flood(Address::random()),
            Id::random(),
        )
        .add(vec![0x41, 0x43, 0x41, 0x42])
        .build()
        .remove(0)
    }

    /// Build a one-off frame with inline payload
    pub fn inline_flood(sender: Address, scope: Address, payload: Vec<u8>) -> Frame {
        SeqBuilder::new(sender, Recipient::Flood(scope), Id::random())
            .add(payload)
            .build()
            .remove(0)
    }

    /// Return the sequence Id of a frame
    pub fn seqid(&self) -> Id {
        self.seq.seqid
    }
}
