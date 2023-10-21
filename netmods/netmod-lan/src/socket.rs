// SPDX-FileCopyrightText: 2019-2021 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Socket handler module

use crate::{framing::Handshake, AddrTable, MemoryEnvelopeExt};
use async_std::{
    future::{self, Future},
    net::{Ipv6Addr, SocketAddr, SocketAddrV6, UdpSocket},
    pin::Pin,
    sync::{Arc, RwLock},
    task::{self, Poll},
};
use libratman::{
    netmod::{InMemoryEnvelope, Target},
    types::frames::{modes, CarrierFrameHeader},
};
use std::collections::VecDeque;
use std::ffi::CString;
use task_notify::Notify;

const MULTI: Ipv6Addr = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x1312);
const SELF: Ipv6Addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0);

/// Wraps around a UDP socket an the input queue
pub(crate) struct Socket {
    port: u16,
    scope: u32,
    sock: Arc<UdpSocket>,
    inbox: Arc<RwLock<Notify<VecDeque<MemoryEnvelopeExt>>>>,
}

fn if_nametoindex(name: &str) -> std::io::Result<u32> {
    use std::io::{Error, ErrorKind};

    let name = match CString::new(name) {
        Ok(cstr) => cstr,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "interface name contained a null",
            ))
        }
    };
    let res = unsafe { libc::if_nametoindex(name.as_ptr()) };
    if res != 0 {
        Ok(res)
    } else {
        Err(Error::last_os_error())
    }
}

impl Socket {
    /// Create a new socket handler and return a management reference
    pub(crate) async fn new(iface: &str, port: u16, table: Arc<AddrTable>) -> Arc<Self> {
        // FIXME: is this blocking?
        let scope = if_nametoindex(iface).expect("failed to turn interface name into index");
        let sock = UdpSocket::bind((SELF, port)).await.unwrap();
        sock.join_multicast_v6(&MULTI, scope)
            .expect("Failed to join multicast. Error");
        sock.set_multicast_loop_v6(false)
            .expect("Failed to set_multicast_loop_v6. Error");

        let arc = Arc::new(Self {
            port,
            scope,
            sock: Arc::new(sock),
            inbox: Default::default(),
        });

        Self::incoming_handle(Arc::clone(&arc), table);
        arc.multicast(&Handshake::Announce.to_carrier().unwrap())
            .await;

        arc
    }

    /// Send a message to one specific client
    pub(crate) async fn send(&self, env: &InMemoryEnvelope, peer: SocketAddrV6) {
        self.sock
            .send_to(&env.buffer.as_slice(), peer)
            .await
            .unwrap();
    }

    /// Send a multicast with an InMemoryEnvelope
    pub(crate) async fn multicast(&self, env: &InMemoryEnvelope) {
        match self
            .sock
            .send_to(
                &env.buffer.as_slice(),
                SocketAddrV6::new(MULTI.clone(), self.port, 0, self.scope),
            )
            .await
        {
            Ok(_) => info!("Sent multicast announcement"),
            Err(e) => error!("failed to multicast frame: {}", e),
        }
    }

    pub(crate) async fn next(&self) -> MemoryEnvelopeExt {
        future::poll_fn(|ctx| {
            let lock = &mut self.inbox.write();
            match unsafe { Pin::new_unchecked(lock).poll(ctx) } {
                Poll::Ready(ref mut inc) => match inc.pop_front() {
                    Some(f) => Poll::Ready(f),
                    None => {
                        Notify::clear_waker(inc);
                        Notify::register_waker(inc, ctx.waker());
                        Poll::Pending
                    }
                },
                Poll::Pending => Poll::Pending,
            }
        })
        .await
    }

    #[instrument(skip(arc, table), level = "trace")]
    fn incoming_handle(arc: Arc<Self>, table: Arc<AddrTable>) {
        task::spawn(async move {
            loop {
                // fixme: aaaaaaaaaaaaaaaaaaaaaaaaaah
                let mut buf = vec![0; 1024 * 16];

                match arc.sock.recv_from(&mut buf).await {
                    Ok((_, peer)) => {
                        let peer = match peer {
                            SocketAddr::V6(v6) => v6,
                            _ => {
                                // ignoring IPv4 packets
                                continue;
                            }
                        };

                        // Skip this frame if it came from self --
                        // this happens because multicast receives our
                        // own messages too
                        match arc.sock.local_addr() {
                            Ok(SocketAddr::V6(local)) if local == peer => continue,
                            Ok(_) => {}
                            _data => {
                                warn!("failed to verify local-loop integrety.  this might caus issues!");
                            }
                        };

                        let env = InMemoryEnvelope::parse_from_buffer(buf).unwrap();
                        match env.header.get_modes() {
                            crate::framing::modes::HANDSHAKE_ANNOUNCE => {
                                trace!("Recieving announce");
                                table.set(peer).await;
                                arc.multicast(&Handshake::Reply.to_carrier().unwrap()).await;
                            }
                            crate::framing::modes::HANDSHAKE_REPLY => {
                                trace!("Recieving announce reply");
                                table.set(peer).await;
                            }
                            _ => {
                                trace!("(Most likely) received data frame");
                                let id = table.id(peer.into()).await.unwrap();

                                // Append to the inbox and wake
                                let mut inbox = arc.inbox.write().await;
                                inbox.push_back(MemoryEnvelopeExt(env, Target::Single(id)));
                                Notify::wake(&mut inbox);
                            }
                        }
                    }
                    val => {
                        // TODO: handle errors more gracefully
                        error!("Crashed UDP thread: {:#?}", val);
                        val.expect("Crashed UDP thread!");
                    }
                }
            }
        });
    }
}

#[ignore]
#[test]
fn test_init() {
    task::block_on(async move {
        let table = Arc::new(AddrTable::new());
        let sock = Socket::new("br42", 12322, table).await;
        println!("Multicasting");
        sock.multicast(&Handshake::Announce.to_carrier().unwrap())
            .await;
    });
}
