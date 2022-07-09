// SPDX-FileCopyrightText: 2019-2021 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use async_std::channel::{unbounded, Receiver, Sender};
use netmod::Frame;

pub struct FuzzEndpoint {
    tx: Sender<(Frame, netmod::Target)>,
    rx: Receiver<(Frame, netmod::Target)>,
}

impl FuzzEndpoint {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        Self { tx, rx }
    }

    pub async fn recv(&self, buf: &[u8]) {
        if let Ok(frame) = bincode::deserialize(&buf) {
            let _ = self.tx.send((frame, netmod::Target::Single(0))).await;
        }
    }
}

#[async_trait::async_trait]
impl netmod::Endpoint for FuzzEndpoint {
    fn size_hint(&self) -> usize {
        0
    }

    async fn send(&self, _: Frame, _: netmod::Target) -> Result<(), netmod::Error> {
        Ok(())
    }

    async fn next(&self) -> Result<(Frame, netmod::Target), netmod::Error> {
        Ok(self.rx.recv().await.unwrap())
    }
}
