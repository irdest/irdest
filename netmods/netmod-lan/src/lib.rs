// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2019-2022 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore
#![allow(warnings)]
#[macro_use]
extern crate tracing;

mod addrs;
pub(crate) use addrs::AddrTable;

mod socket;
pub(crate) use socket::Socket;

mod framing;
pub(crate) use framing::MemoryEnvelopeExt;

use async_std::{sync::Arc, task};
use async_trait::async_trait;
use libratman::netmod::{Endpoint as EndpointExt, InMemoryEnvelope, Target};
use libratman::types::Result;
use pnet_datalink::interfaces;

#[derive(Clone)]
pub struct Endpoint {
    socket: Arc<Socket>,
    addrs: Arc<AddrTable>,
}

impl Endpoint {
    /// Create a new endpoint and spawn a dispatch task
    pub fn spawn(iface: Option<String>, port: u16) -> std::result::Result<Arc<Self>, &'static str> {
        let iface_string = iface
            .or_else(|| {
                default_iface().map(|iface| {
                    info!(
                        "Auto-selected interface '{}' for local peer discovery.  \
                       (You can override the interface via the ratmand configuration)",
                        iface
                    );
                    iface
                })
            })
            .ok_or_else(|| "Could not find an interface to bind on.")?;

        Ok(task::block_on(async move {
            let addrs = Arc::new(AddrTable::new());
            Arc::new(Self {
                socket: Socket::new(&iface_string, port, Arc::clone(&addrs)).await,
                addrs,
            })
        }))
    }

    #[cfg(test)]
    pub async fn peers(&self) -> usize {
        self.addrs.all().await.len()
    }
}

#[async_trait]
impl EndpointExt for Endpoint {
    fn size_hint(&self) -> usize {
        0
    }

    async fn send(
        &self,
        envelope: InMemoryEnvelope,
        target: Target,
        exclude: Option<u16>,
    ) -> Result<()> {
        match target {
            Target::Single(ref id) => {
                self.socket
                    // todo: do we need to prefix a length here ???
                    .send(&envelope, self.addrs.ip(*id).await.unwrap())
                    .await
            }

            // When `exclude` is_some we can assume that we are in the
            // process of re-flooding something.  Because netmod-lan
            // is not segmented (i.e. all peers all also know each
            // other) we can just not bother to send the message
            // again (hopefully)
            Target::Flood if exclude.is_none() => {
                self.socket.multicast(&envelope).await;
            }
            _ => {}
        }

        Ok(())
    }

    async fn next(&self) -> Result<(InMemoryEnvelope, Target)> {
        let fe = self.socket.next().await;
        Ok((fe.0, fe.1))
    }
}

/// Try to get a "default" interface for LAN discovery
pub fn default_iface() -> Option<String> {
    let all = interfaces();
    all.into_iter()
        .find(|e| e.is_up() && !e.is_loopback() && e.ips.iter().any(|ip| ip.is_ipv6()))
        .map(|iface| iface.name)
}
