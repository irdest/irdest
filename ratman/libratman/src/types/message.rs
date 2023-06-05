// SPDX-FileCopyrightText: 2019-2023 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

#[cfg(feature = "proto")]
pub use crate::types::proto::message::Message as ProtoMessage;

use crate::types::{timepair::TimePair, Address, Id};
use serde::{Deserialize, Serialize};

/// Specify the message recipient
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Recipient {
    Standard(Vec<Address>),
    Flood(Address),
}

impl Recipient {
    /// Return the scope of this recipient
    ///
    /// This is either the target user identity or the flood scope
    pub fn scope(&self) -> Option<Address> {
        match self {
            Self::Standard(id) => id.first().map(|x| x.clone()),
            Self::Flood(scope) => Some(*scope),
        }
    }

    /// Create a standard message recipient
    pub fn standard<T: Into<Vec<Address>>>(addrs: T) -> Self {
        Self::Standard(addrs.into())
    }
    /// Create a flood message recipient
    pub fn flood(namespace: Address) -> Self {
        Self::Flood(namespace)
    }
}

impl From<Vec<Address>> for Recipient {
    fn from(vec: Vec<Address>) -> Self {
        Self::Standard(vec)
    }
}

/// Main Ratman message type
///
/// A message can either be addressed to a single recipient, or a
/// namespace on the network.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    id: Id,
    sender: Address,
    recipient: Recipient,
    time: TimePair,
    payload: Vec<u8>,
    signature: Vec<u8>,
}

impl Message {
    pub fn new(
        sender: Address,
        recipient: impl Into<Recipient>,
        payload: Vec<u8>,
        signature: Vec<u8>,
    ) -> Self {
        Message {
            id: Id::random(),
            recipient: recipient.into(),
            time: TimePair::sending(),
            sender,
            payload,
            signature,
        }
    }

    pub fn get_id(&self) -> Id {
        self.id.clone()
    }

    pub fn get_sender(&self) -> Address {
        self.sender.clone()
    }

    pub fn get_recipient(&self) -> Recipient {
        self.recipient.clone()
    }

    pub fn get_time(&self) -> TimePair {
        self.time.clone()
    }

    pub fn get_payload(&self) -> Vec<u8> {
        self.payload.clone()
    }

    pub fn get_signature(&self) -> Vec<u8> {
        self.signature.clone()
    }

    // return protobuf type Message.
    #[cfg(feature = "proto")]
    pub fn received(
        id: Address,
        sender: Address,
        recipient: Recipient,
        payload: Vec<u8>,
        timesig: String,
        sign: Vec<u8>,
    ) -> ProtoMessage {
        let mut inner = ProtoMessage::new();
        inner.set_id(id.as_bytes().to_vec());
        inner.set_sender(sender.as_bytes().to_vec());

        use crate::types::proto::message::{Recipient as ProtoRecipient, StandardRecipient};

        let mut r = ProtoRecipient::new();
        match recipient {
            Recipient::Standard(addrs) => {
                let mut std_r = StandardRecipient::new();
                std_r.set_standard(
                    addrs
                        .into_iter()
                        .map(|id| id.as_bytes().to_vec())
                        .collect::<Vec<_>>()
                        .into(),
                );
                r.set_std(std_r);
            }
            Recipient::Flood(ns) => {
                r.set_flood_scope(ns.as_bytes().to_vec().into());
            }
        }

        inner.set_recipient(r);
        inner.set_time(timesig);
        inner.set_payload(payload);
        inner.set_signature(sign);

        inner
    }
}

/// Implement RAW `From` protobuf type message
#[cfg(feature = "proto")]
impl From<ProtoMessage> for Message {
    fn from(mut msg: ProtoMessage) -> Self {
        use crate::types::proto::message::Recipient_oneof_inner;

        let mut r = msg.take_recipient();
        let recipient = match r.inner {
            Some(Recipient_oneof_inner::std(ref mut std)) => Recipient::Standard(
                std.take_standard()
                    .into_iter()
                    .map(|id| Address::from_bytes(&id))
                    .collect(),
            ),
            Some(Recipient_oneof_inner::flood_scope(ref id)) => {
                Recipient::Flood(Address::from_bytes(id))
            }
            _ => unreachable!(),
        };

        Message {
            id: Address::from_bytes(msg.get_id()),
            sender: Address::from_bytes(msg.get_id()),
            recipient,
            time: TimePair::from_string(msg.get_time()),
            payload: msg.get_payload().to_vec(),
            signature: msg.get_signature().to_vec(),
        }
    }
}

/// Implement protobuf type message `From` RAW
#[cfg(feature = "proto")]
impl From<Message> for ProtoMessage {
    fn from(msg: Message) -> ProtoMessage {
        let mut inner = ProtoMessage::new();
        inner.set_sender(msg.sender.as_bytes().to_vec());

        use crate::types::proto::message::{Recipient as ProtoRecipient, StandardRecipient};

        let mut r = ProtoRecipient::new();
        match msg.recipient {
            Recipient::Standard(addrs) => {
                let mut std_r = StandardRecipient::new();
                std_r.set_standard(
                    addrs
                        .into_iter()
                        .map(|id| id.as_bytes().to_vec())
                        .collect::<Vec<_>>()
                        .into(),
                );
                r.set_std(std_r);
            }
            Recipient::Flood(ns) => {
                r.set_flood_scope(ns.as_bytes().to_vec().into());
            }
        }

        inner.set_recipient(r);
        inner.set_payload(msg.payload);
        inner.set_signature(msg.signature);
        inner
    }
}
