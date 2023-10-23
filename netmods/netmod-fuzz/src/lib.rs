// SPDX-FileCopyrightText: 2019-2021 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use async_std::channel::{unbounded, Receiver, Sender};
use libratman::{
    netmod::{self, InMemoryEnvelope},
    types::frames::{CarrierFrameHeader, FrameParser},
    RatmanError,
};

pub struct FuzzEndpoint {
    tx: Sender<(InMemoryEnvelope, netmod::Target)>,
    rx: Receiver<(InMemoryEnvelope, netmod::Target)>,
}

impl FuzzEndpoint {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        Self { tx, rx }
    }

    pub async fn recv(&self, buf: &[u8]) {
        if let Ok(env) = InMemoryEnvelope::parse_from_buffer(buf.to_vec()) {
            let _ = self.tx.send((env, netmod::Target::Single(0))).await;
        }
    }
}

#[async_trait::async_trait]
impl netmod::Endpoint for FuzzEndpoint {
    fn size_hint(&self) -> usize {
        0
    }

    async fn send(
        &self,
        _: InMemoryEnvelope,
        _: netmod::Target,
        _: Option<u16>,
    ) -> Result<(), RatmanError> {
        Ok(())
    }

    async fn next(&self) -> Result<(InMemoryEnvelope, netmod::Target), RatmanError> {
        Ok(self.rx.recv().await.unwrap())
    }
}
