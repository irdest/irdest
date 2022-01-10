//! Ratman message abstraction

pub use crate::proto::message::Message;
use ratman_identity::Identity;

/// Create a new `Message`
pub fn new(
    sender: Identity,
    recipients: Vec<Identity>,
    payload: Vec<u8>,
    signature: Vec<u8>,
) -> Message {
    let mut inner = Message::new();
    inner.set_sender(sender.as_bytes().to_vec());
    inner.set_recipients(
        recipients
            .iter()
            .map(|r| r.as_bytes().to_vec())
            .collect::<Vec<_>>()
            .into(),
    );
    inner.set_payload(payload);
    inner.set_signature(signature);
    inner
}
