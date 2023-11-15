//! Netmod driver for Android WiFi Direct

use async_std::{
    channel::{bounded, Receiver, Sender},
    sync::Arc,
    task,
};
use async_trait::async_trait;
use libratman::{
    endpoint::EndpointExt,
    types::{InMemoryEnvelope, Neighbour},
    Result,
};

pub struct WdMod {
    recv_queue: (
        Sender<(InMemoryEnvelope, Neighbour)>,
        Receiver<(InMemoryEnvelope, Neighbour)>,
    ),
    send_queue: (
        Sender<(InMemoryEnvelope, Neighbour)>,
        Receiver<(InMemoryEnvelope, Neighbour)>,
    ),
}

impl WdMod {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            recv_queue: bounded(1),
            send_queue: bounded(1),
        })
    }

    /// Give some data to this netmod, receiving it on the device
    ///
    /// This function is called by the java-android driver stack in
    /// android-support which is called by any app that implements the
    /// WifiDirect mode, It could also be used as a general FFI shim
    /// for other drivers.
    pub fn give(self: &Arc<Self>, f: InMemoryEnvelope, t: Neighbour) {
        let this = Arc::clone(self);
        task::spawn(async move { this.recv_queue.0.send((f, t)).await.unwrap() });
    }

    /// Block on taking a new
    pub fn take(self: &Arc<Self>) -> (InMemoryEnvelope, Neighbour) {
        task::block_on(async { self.send_queue.1.recv().await.unwrap() })
    }
}

#[async_trait]
impl EndpointExt for WdMod {
    fn size_hint(&self) -> usize {
        0
    }

    async fn send(&self, frame: InMemoryEnvelope, t: Neighbour, _: Option<u16>) -> Result<()> {
        self.send_queue.0.send((frame, t)).await.unwrap();
        Ok(())
    }

    async fn next(&self) -> Result<(InMemoryEnvelope, Neighbour)> {
        Ok(self.recv_queue.1.recv().await.unwrap())
    }
}
