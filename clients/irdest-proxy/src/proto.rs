use ratman_client::Address;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Envelope {
    pub session: Address,
    pub data: Option<Vec<u8>>,
}

impl Envelope {
    pub fn with_session(session: Address, data: Vec<u8>) -> Self {
        Self {
            session,
            data: Some(data),
        }
    }

    pub fn end(session: Address) -> Self {
        Self {
            session,
            data: None,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        bincode::serialize(self).expect("failed to encode envelope")
    }

    pub fn decode(buf: &Vec<u8>) -> Self {
        bincode::deserialize(buf).expect("failed to decode envelope")
    }
}
