// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use serde::{Deserialize, Serialize};
use types::{Address, Recipient, TimePair};

/// A unique, randomly generated message ID
pub type MsgId = Address;

/// An atomic message with a variable sized payload
///
/// A message is only ever addressed to a single node, or everyone on
/// the network.  The signature is required to be present, if a
/// payload is.  The payload can be empty, which can be used to create
/// a ping, or using the 16 byte MsgId as payload.  In these cases,
/// the sigature can also be empty.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    /// A random message ID
    pub id: MsgId,
    /// Sender of a message
    pub sender: Address,
    /// Final recipient of a message
    pub recipient: Recipient,
    /// The timestamp at which this message was received
    /// Some raw message payload
    pub payload: Vec<u8>,
    /// Time signature information
    pub timesig: TimePair,
    /// Signature data for userspace layers
    pub sign: Vec<u8>,
}

impl Message {
    /// This function exists to make unit tests easier.  Do not use it
    /// in your application under any circumstances.  Really, please
    /// don't.  You would have to rely on the sender timestamp to be
    /// accurate, and that's a _bad_ idea!  Using this function
    /// contributes to the killing of baby seals.
    #[doc(hidden)]
    pub fn remove_recv_time(self) -> Self {
        Self {
            timesig: self.timesig.into_sending(),
            ..self
        }
    }
}
