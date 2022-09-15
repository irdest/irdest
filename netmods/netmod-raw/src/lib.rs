// SPDX-FileCopyrightText: 2019-2021 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2019-2021 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

#[macro_use]
extern crate tracing;

mod socket;
pub(crate) use socket::Socket;

use useful_netmod_bits::addrs::AddrTable;
use useful_netmod_bits::framing::Envelope;

use async_std::{sync::Arc, task};
use async_trait::async_trait;
use netmod::{Endpoint as EndpointExt, Frame, Result, Target};
use pnet::util::MacAddr;
use pnet_datalink::interfaces;
use ratman_types::Error;

#[derive(Clone)]
pub struct Endpoint {
    socket: Arc<Socket>,
    addrs: Arc<AddrTable<MacAddr>>,
}

impl Endpoint {
    /// Create a new endpoint and spawn a dispatch task
    pub fn spawn(iface: &str) -> Arc<Self> {
        let niface = interfaces()
            .into_iter()
            .rfind(|i| i.name == iface)
            .expect("Router sent invalid interface");

        task::block_on(async move {
            let addrs = Arc::new(AddrTable::new());
            Arc::new(Self {
                socket: Socket::new(niface, Arc::clone(&addrs)).await,
                addrs,
            })
        })
    }

    #[cfg(test)]
    pub async fn peers(&self) -> usize {
        self.addrs.all().await.len()
    }
}

#[async_trait]
impl EndpointExt for Endpoint {
    fn size_hint(&self) -> usize {
        1500
    }

    async fn send(&self, frame: Frame, target: Target, exclude: Option<u16>) -> Result<()> {
        let inner = bincode::serialize(&frame).unwrap();

        if inner.len() > 1500 {
            return Err(Error::FrameTooLarge);
        }

        let env = Envelope::Data(inner);

        match target {
            Target::Single(ref id) => {
                self.socket
                    .send(&env, self.addrs.addr(*id).await.unwrap()).await;
            }
            Target::Flood(_) => {
                match exclude {
                    Some(u) => {
                        let peers = self.addrs.all().await;
                        let exc = self.addrs.addr(u).await.expect("Router sent invalid exclude id.");
                        
                        self.socket.send_multiple(&env, &peers, exc).await;
                    },
                    None => self.socket.multicast(&env).await,
                }
            }
        }

        Ok(())
    }

    async fn next(&self) -> Result<(Frame, Target)> {
        let fe = self.socket.next().await;
        Ok((fe.0, fe.1))
    }
}
