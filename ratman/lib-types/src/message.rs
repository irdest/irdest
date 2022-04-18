// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use serde::{Deserialize, Serialize};

#[cfg(feature = "proto")]
pub use crate::proto::message::Message as ProtoMessage;

use crate::{timepair::TimePair, Identity};

/// Specify the message recipient
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Recipient {
    Standard(Vec<Identity>),
    Flood(Identity),
}

impl Recipient {
    /// Return the scope of this recipient
    ///
    /// This is either the target user identity or the flood scope
    pub fn scope(&self) -> Option<Identity> {
        match self {
            Self::Standard(id) => id.first().map(|x| x.clone()),
            Self::Flood(scope) => Some(*scope),
        }
    }

    /// Create a standard message recipient
    pub fn standard<T: Into<Vec<Identity>>>(addrs: T) -> Self {
        Self::Standard(addrs.into())
    }
    /// Create a flood message recipient
    pub fn flood(namespace: Identity) -> Self {
        Self::Flood(namespace)
    }
}

impl From<Vec<Identity>> for Recipient {
    fn from(vec: Vec<Identity>) -> Self {
        Self::Standard(vec)
    }
}

/// Main Ratman message type
///
/// A message can either be addressed to a single recipient, or a
/// namespace on the network.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    id: Identity,
    sender: Identity,
    recipient: Recipient,
    time: TimePair,
    payload: Vec<u8>,
    signature: Vec<u8>,
}

impl Message {
    pub fn new(
        sender: Identity,
        recipient: impl Into<Recipient>,
        payload: Vec<u8>,
        signature: Vec<u8>,
    ) -> Self {
        Message {
            id: Identity::random(),
            recipient: recipient.into(),
            time: TimePair::sending(),
            sender,
            payload,
            signature,
        }
    }

    pub fn get_payload(&self) -> Vec<u8> {
        self.payload.clone()
    }

    pub fn get_sender(&self) -> Identity {
        self.sender.clone()
    }

    // return protobuf type Message.
    #[cfg(feature = "proto")]
    pub fn received(
        id: Identity,
        sender: Identity,
        recipient: Recipient,
        payload: Vec<u8>,
        timesig: String,
        sign: Vec<u8>,
    ) -> ProtoMessage {
        let mut inner = ProtoMessage::new();
        inner.set_id(id.as_bytes().to_vec());
        inner.set_sender(sender.as_bytes().to_vec());

        use crate::proto::message::{Recipient as ProtoRecipient, StandardRecipient};

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
        use crate::proto::message::Recipient_oneof_inner;

        let mut r = msg.take_recipient();
        let recipient = match r.inner {
            Some(Recipient_oneof_inner::std(ref mut std)) => Recipient::Standard(
                std.take_standard()
                    .into_iter()
                    .map(|id| Identity::from_bytes(&id))
                    .collect(),
            ),
            Some(Recipient_oneof_inner::flood_scope(ref id)) => {
                Recipient::Flood(Identity::from_bytes(id))
            }
            _ => unreachable!(),
        };

        Message {
            id: Identity::from_bytes(msg.get_id()),
            sender: Identity::from_bytes(msg.get_id()),
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

        use crate::proto::message::{Recipient as ProtoRecipient, StandardRecipient};

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
