use crate::{
    crypto::{CipherText, CryReqPayload, CryRespPayload, ReqPayload, ResponsePayload},
    meta::KeyStore,
};
use async_std::{
    channel::{self, Receiver, Sender},
    sync::Arc,
    task,
};
use id::Identity;

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
        while let Ok(CryReqPayload { resp, user, op }) = self.rx.recv().await {
            match op {
                ReqPayload::Encrypt(ref payload) => self.encrypt(resp, payload, user).await,
                ReqPayload::Decrypt(ref payload) => self.decrypt(resp, payload, user).await,
            }
        }
    }

    /// Execute an encryption request for a particular user ID
    async fn encrypt(&self, resp: Sender<CryRespPayload>, payload: &Vec<u8>, user: Identity) {
        let (pub_, sec_) = self.keys.get_pair(&user).await.unwrap();
        let payload = pub_.seal(payload.as_slice(), sec_.as_ref());

        resp.send(CryRespPayload {
            status: 0,
            payload: ResponsePayload::Encrypted(payload),
        });
    }

    /// Execute a decryption request for a particular user ID
    async fn decrypt(&self, resp: Sender<CryRespPayload>, payload: &CipherText, user: Identity) {
        let (pub_, sec_) = self.keys.get_pair(&user).await.unwrap();
        let response = match sec_.open(payload, pub_.as_ref()) {
            Ok(payload) => CryRespPayload {
                status: 0,
                payload: ResponsePayload::Clear(payload),
            },
            Err(e) => {
                error!("Oh no!  An error has occured in an asynchronous process (decrypting payload): {}", e);
                CryRespPayload {
                    status: 1,
                    payload: ResponsePayload::Clear(vec![]),
                }
            }
        };

        resp.send(response).await;
    }
}
