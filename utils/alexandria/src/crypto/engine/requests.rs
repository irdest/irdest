use async_std::channel::{bounded, Receiver, Sender};
use id::Identity;

use crate::crypto::CipherText;

/// A single encryption request to the engine
pub(crate) struct CryReqPayload {
    pub resp: Sender<CryRespPayload>,
    pub user: Identity,
    pub op: ReqPayload,
}

impl CryReqPayload {
    fn new(user: Identity, op: ReqPayload) -> (Self, Receiver<CryRespPayload>) {
        let (resp, rx) = bounded(1);
        (Self { resp, user, op }, rx)
    }

    pub(crate) fn decrypt(user: Identity, payload: CipherText) -> (Self, Receiver<CryRespPayload>) {
        Self::new(user, ReqPayload::Decrypt(payload))
    }

    pub(crate) fn encrypt(user: Identity, payload: Vec<u8>) -> (Self, Receiver<CryRespPayload>) {
        Self::new(user, ReqPayload::Encrypt(payload))
    }
}

/// Indicate which operation to perform
pub(crate) enum ReqPayload {
    Encrypt(Vec<u8>),
    Decrypt(CipherText),
}

pub(crate) enum ResponsePayload {
    Clear(Vec<u8>),
    Encrypted(CipherText),
}

/// Response to the cryptographic operation
pub(crate) struct CryRespPayload {
    pub status: u32,
    pub payload: ResponsePayload,
}
