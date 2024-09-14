// SPDX-FileCopyrightText: 2019-2021 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Socket handler module

use crate::{framing::HandshakeV1, AddrTable, MemoryEnvelopeExt};
use libratman::endpoint::NeighbourMetrics;
use libratman::futures::future::{self, Future};
use libratman::tokio::task::{spawn, spawn_local};
use libratman::tokio::time::{sleep, Instant};
use libratman::tokio::{net::UdpSocket, sync::RwLock, task};
use libratman::{
    frame::carrier::{modes, CarrierFrameHeader},
    types::{Ident32, InMemoryEnvelope, Neighbour},
};
use std::collections::{BTreeMap, VecDeque};
use std::ffi::CString;
use std::net::{IpAddr, Ipv6Addr, SocketAddr, SocketAddrV6};
use std::time::Duration;
use std::{pin::Pin, sync::Arc, task::Poll};
use task_notify::Notify;

const MULTI: Ipv6Addr = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x1312);
const SELF: Ipv6Addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0);

/// Wraps around a UDP socket an the input queue
pub(crate) struct Socket {
    port: u16,
    scope: u32,
    self_rk_id: Ident32,
    sock: Arc<UdpSocket>,
    inbox: Arc<RwLock<Notify<VecDeque<MemoryEnvelopeExt>>>>,
    pub metrics: Arc<MetricsTable>,
}

/// The metrics table keeps track of connection metrics for a given
pub struct MetricsTable {
    /// (Last time numbers were consolidated, Last period, Current accumulator)
    pub inner: RwLock<BTreeMap<SocketAddrV6, (Instant, NeighbourMetrics, NeighbourMetrics)>>,
}

impl MetricsTable {
    fn new() -> Self {
        Self {
            inner: RwLock::new(BTreeMap::new()),
        }
    }

    async fn append_write(self: &Arc<Self>, peer: SocketAddrV6, bytes: usize) {
        let this = Arc::clone(&self);
        let mut map = self.inner.write().await;

        map.entry(peer).or_insert_with(|| {
            spawn_local(async move {
                sleep(Duration::from_secs(12)).await;
                let mut map = this.inner.write().await;
                let (mut last_time, mut last_period, mut curr_acc) = map.get_mut(&peer).unwrap();
                last_time = Instant::now();
                last_period.write_bandwidth = curr_acc.write_bandwidth;
                curr_acc.write_bandwidth = 0;
            });
            (Instant::now(), Default::default(), Default::default())
        });
    }

    async fn append_read(self: &Arc<Self>, peer: SocketAddrV6, bytes: usize) {
        let this = Arc::clone(&self);
        let mut map = self.inner.write().await;

        map.entry(peer).or_insert_with(|| {
            spawn_local(async move {
                sleep(Duration::from_secs(12)).await;
                let mut map = this.inner.write().await;
                let (mut last_time, mut last_period, mut curr_acc) = map.get_mut(&peer).unwrap();
                last_time = Instant::now();
                last_period.read_bandwidth = curr_acc.read_bandwidth;
                curr_acc.read_bandwidth = 0;
            });
            (Instant::now(), Default::default(), Default::default())
        });
    }
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
    pub(crate) async fn new(
        iface: &str,
        port: u16,
        table: Arc<AddrTable>,
        r_key_id: Ident32,
    ) -> Arc<Self> {
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
            self_rk_id: r_key_id,
            sock: Arc::new(sock),
            inbox: Default::default(),
            metrics: Arc::new(MetricsTable::new()),
        });

        Self::incoming_handle(Arc::clone(&arc), table);
        arc.multicast(&HandshakeV1::Announce(arc.self_rk_id).to_carrier().unwrap())
            .await;

        arc
    }

    /// Send a message to one specific client
    pub(crate) async fn send(&self, env: &InMemoryEnvelope, peer: SocketAddrV6) {
        let bytes_written = self
            .sock
            .send_to(&env.buffer.as_slice(), peer)
            .await
            .unwrap();
        let metrics = Arc::clone(&self.metrics);
        spawn_local(async move { metrics.append_write(peer, bytes_written).await });
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
            Ok(_) => trace!("Sent multicast announcement"),
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
        spawn(async move {
            loop {
                // fixme: aaaaaaaaaaaaaaaaaaaaaaaaaah
                let mut buf = vec![0; 1024 * 16];

                match arc.sock.recv_from(&mut buf).await {
                    Ok((bytes_read, peer)) => {
                        let peer = match peer {
                            SocketAddr::V6(v6) => v6,
                            _ => {
                                // ignoring IPv4 packets
                                continue;
                            }
                        };

                        let metrics = Arc::clone(&arc.metrics);
                        spawn_local(async move { metrics.append_read(peer, bytes_read).await });

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
                        let payload = env.get_payload_slice();

                        trace!("Decoding carrier frame payload: {:?}", payload);

                        match env.header.get_modes() {
                            crate::framing::modes::HANDSHAKE_ANNOUNCE => {
                                let hshake: HandshakeV1 = bincode::deserialize(payload).unwrap();

                                trace!("Recieving announce");
                                table.set(peer, hshake.r_key_id()).await;
                                arc.multicast(
                                    &HandshakeV1::Reply(arc.self_rk_id).to_carrier().unwrap(),
                                )
                                .await;
                            }
                            crate::framing::modes::HANDSHAKE_REPLY => {
                                trace!("Recieving announce reply");
                                let hshake: HandshakeV1 = bincode::deserialize(payload).unwrap();
                                table.set(peer, hshake.r_key_id()).await;
                            }
                            _ => {
                                trace!("(Most likely) received data frame");
                                if let Some(id) = table.id(peer.into()).await {
                                    // Append to the inbox and wake
                                    let mut inbox = arc.inbox.write().await;
                                    inbox.push_back(MemoryEnvelopeExt(env, Neighbour::Single(id)));
                                    Notify::wake(&mut inbox);
                                }
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

#[cfg(test)]
use libratman::tokio;

#[ignore]
#[libratman::tokio::test]
async fn test_init() {
    let table = Arc::new(AddrTable::new());
    let kid = Ident32::random();
    let sock = Socket::new("br42", 12322, table, kid).await;
    println!("Multicasting");
    sock.multicast(&HandshakeV1::Announce(kid).to_carrier().unwrap())
        .await;
}
