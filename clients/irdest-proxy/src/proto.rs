use ratman_client::Identity;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Envelope {
    pub session: Identity,
    pub data: Option<Vec<u8>>,
}

impl Envelope {
    pub fn with_session(session: Identity, data: Vec<u8>) -> Self {
        Self {
            session,
            data: Some(data),
        }
    }

    pub fn end(session: Identity) -> Self {
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
