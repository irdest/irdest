// SPDX-FileCopyrightText: 2019-2021 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use async_std::channel::{unbounded, Receiver, Sender};
use libratman::{
    endpoint::EndpointExt,
    types::{Ident32, InMemoryEnvelope, Neighbour},
    RatmanError,
};

pub struct FuzzEndpoint {
    tx: Sender<(InMemoryEnvelope, Neighbour)>,
    rx: Receiver<(InMemoryEnvelope, Neighbour)>,
}

impl FuzzEndpoint {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        Self { tx, rx }
    }

    pub async fn recv(&self, buf: &[u8]) {
        if let Ok(env) = InMemoryEnvelope::parse_from_buffer(buf.to_vec()) {
            let _ = self
                .tx
                .send((env, Neighbour::Single(Ident32::random())))
                .await;
        }
    }
}

#[async_trait::async_trait]
impl EndpointExt for FuzzEndpoint {
    fn size_hint(&self) -> usize {
        0
    }

    async fn send(
        &self,
        _: InMemoryEnvelope,
        _: Neighbour,
        _: Option<Ident32>,
    ) -> Result<(), RatmanError> {
        Ok(())
    }

    async fn next(&self) -> Result<(InMemoryEnvelope, Neighbour), RatmanError> {
        Ok(self.rx.recv().await.unwrap())
    }
}
