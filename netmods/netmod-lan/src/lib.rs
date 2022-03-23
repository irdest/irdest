//! netmod-udp is a UDP overlay for Ratman
#![allow(warnings)]

#[macro_use]
extern crate tracing;

mod addrs;
pub(crate) use addrs::AddrTable;

mod socket;
pub(crate) use socket::Socket;

mod framing;
pub(crate) use framing::{Envelope, FrameExt};

use async_std::{sync::Arc, task};
use async_trait::async_trait;
use netmod::{Endpoint as EndpointExt, Frame, Recipient, Result, Target};
use pnet::datalink::interfaces;
use std::net::ToSocketAddrs;

#[derive(Clone)]
pub struct Endpoint {
    socket: Arc<Socket>,
    addrs: Arc<AddrTable>,
}

impl Endpoint {
    /// Create a new endpoint and spawn a dispatch task
    pub fn spawn(iface: &str, port: u16) -> Arc<Self> {
        task::block_on(async move {
            let addrs = Arc::new(AddrTable::new());
            Arc::new(Self {
                socket: Socket::new(iface, port, Arc::clone(&addrs)).await,
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
        0
    }

    async fn send(&self, frame: Frame, target: Target) -> Result<()> {
        let inner = bincode::serialize(&frame).unwrap();
        let env = Envelope::Data(inner);
        match target {
            /// Sending to a user,
            Target::Single(ref id) => {
                self.socket
                    .send(&env, self.addrs.ip(*id).await.unwrap())
                    .await
            }
            Target::Flood(_) => {
                self.socket.multicast(&env).await;
            }
        }

        Ok(())
    }

    async fn next(&self) -> Result<(Frame, Target)> {
        let fe = self.socket.next().await;
        Ok((fe.0, fe.1))
    }
}

/// Try to get a "default" interface for LAN discovery
pub fn default_iface() -> Option<String> {
    let all = interfaces();
    all.into_iter()
        .find(|e| e.is_up() && !e.is_loopback() && !e.ips.is_empty())
        .map(|iface| iface.name)
}
