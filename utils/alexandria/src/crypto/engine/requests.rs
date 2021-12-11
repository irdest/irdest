use async_std::channel::{bounded, Receiver, Sender};

/// A single encryption request to the engine
pub struct CryReqPayload {
    pub resp: Sender<CryRespPayload>,
    pub payload: Vec<u8>,
    pub op: CryOp,
    // TODO: add credential authentication
}

impl CryReqPayload {
    fn new(payload: Vec<u8>, op: CryOp) -> (Self, Receiver<CryRespPayload>) {
        let (resp, rx) = bounded(1);
        (Self { resp, payload, op }, rx)
    }

    pub(crate) fn decrypt(payload: Vec<u8>) -> (Self, Receiver<CryRespPayload>) {
        Self::new(payload, CryOp::Decrypt)
    }

    pub(crate) fn encrypt(payload: Vec<u8>) -> (Self, Receiver<CryRespPayload>) {
        Self::new(payload, CryOp::Encrypt)
    }
}

/// Indicate which operation to perform
pub enum CryOp {
    Encrypt,
    Decrypt,
}

/// Response to the cryptographic operation
pub struct CryRespPayload {
    pub status: u32,
    pub payload: Vec<u8>,
}
