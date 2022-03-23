use crate::{Message, MsgId, Recipient, TimePair};
use identity::Identity;
use types::api::{Send, Send_Type};

/// Turn an API `Send` to a `Message`
pub(crate) fn send_to_message(s: Send) -> Vec<Message> {
    // Take the set of recipients from the message and turn it into a
    // set of Ratman recipients
    let recipients: Vec<_> = match s.field_type {
        Send_Type::DEFAULT => s
            .get_msg()
            .recipients
            .iter()
            .map(|r| Recipient::User(Identity::from_bytes(&r)))
            .collect(),
        Send_Type::FLOOD => vec![Recipient::Flood(Identity::from_bytes(s.scope.as_slice()))],
    };
    let timesig = TimePair::sending();

    // Then create a new message for each recipient (if the type is
    // "flood" then only a single message gets created)
    recipients
        .into_iter()
        .map(|recipient| Message {
            id: MsgId::random(),
            sender: Identity::from_bytes(s.get_msg().sender.as_slice()),
            recipient,
            payload: s.get_msg().payload.clone(),
            timesig: timesig.clone(),
            sign: s.get_msg().signature.clone(),
        })
        .collect()
}
