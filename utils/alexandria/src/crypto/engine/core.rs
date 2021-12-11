use crate::{crypto::CryReqPayload, meta::KeyStore};
use async_std::{
    channel::{self, Receiver, Sender},
    sync::Arc,
    task,
};

/// A send-receive handle to the crypto engine
pub struct CryEngineHandle {
    tx: Sender<CryReqPayload>,
}

pub struct CryEngine {
    rx: Receiver<CryReqPayload>,
    keys: Arc<KeyStore>,
}

impl CryEngine {
    pub(crate) async fn new(keys: Arc<KeyStore>) -> CryEngineHandle {
        let (tx, rx) = channel::bounded(64);
        task::spawn(Self { rx, keys }.run());
        CryEngineHandle { tx }
    }

    async fn run(self) {
        while let Ok(req) = self.rx.recv().await {}
    }
}
