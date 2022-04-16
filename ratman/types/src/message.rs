/// A message is only ever addressed to a single node, or everyone on
/// the network.  The signature is required to be present, if a
/// payload is.  The payload can be empty, which can be used to create
/// a ping, or using the 16 byte id as payload.  In these cases,
/// the sigature can also be empty.
use serde::{Deserialize, Serialize};

#[cfg(feature = "proto")]
pub use crate::proto::message::Message as ProtoMessage;

use crate::timepair::TimePair;
use ratman_identity::Identity;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    id: Identity,
    sender: Identity,
    recipients: Vec<Identity>,
    time: TimePair,
    payload: Vec<u8>,
    signature: Vec<u8>,
}

impl Message {
    pub fn new(
        sender: Identity,
        recipients: Vec<Identity>,
        payload: Vec<u8>,
        signature: Vec<u8>,
    ) -> Self {
        Message {
            id: Identity::random(),
            sender: sender,
            recipients: recipients,
            time: TimePair::sending(),
            payload: payload,
            signature: signature,
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
        recipient: Option<Identity>,
        payload: Vec<u8>,
        timesig: String,
        sign: Vec<u8>,
    ) -> ProtoMessage {
        let mut inner = ProtoMessage::new();
        inner.set_id(id.as_bytes().to_vec());
        inner.set_sender(sender.as_bytes().to_vec());
        if let Some(r) = recipient {
            inner.set_recipients(vec![r.as_bytes().to_vec()].into());
        }
        inner.set_time(timesig);
        inner.set_payload(payload);
        inner.set_signature(sign);

        inner
    }
}

/// Implement RAW `From` protobuf type message
#[cfg(feature = "proto")]
impl From<ProtoMessage> for Message {
    fn from(msg: ProtoMessage) -> Self {
        Message {
            id: Identity::from_bytes(msg.get_id()),
            sender: Identity::from_bytes(msg.get_id()),
            recipients: msg
                .get_recipients()
                .iter()
                .map(|r| Identity::from_bytes(r))
                .collect::<Vec<Identity>>()
                .into(),
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
        inner.set_recipients(
            msg.recipients
                .iter()
                .map(|r| r.as_bytes().to_vec())
                .collect::<Vec<_>>()
                .into(),
        );
        inner.set_payload(msg.payload);
        inner.set_signature(msg.signature);
        inner
    }
}
