// SPDX-FileCopyrightText: 2019-2021 Yureka Lilian <yuka@yuka.dev>
// SPDX-FileCopyrightText: 2022 Christopher A. Grant <grantchristophera@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Socket handler module

use async_std::{
    future::{self, Future},
    pin::Pin,
    sync::{Arc, Mutex, RwLock},
    task::{self, Poll},
};
use libratman::netmod::{InMemoryEnvelope, Target};
use nix::errno::Errno;
use pnet::{
    packet::ethernet::{EtherType, EthernetPacket, MutableEthernetPacket},
    packet::Packet,
    util::MacAddr,
};
use pnet_datalink::{channel, Channel, DataLinkReceiver, DataLinkSender, NetworkInterface};
use std::collections::VecDeque;
use std::error::Error;
use task_notify::Notify;
use useful_netmod_bits::addrs::AddrTable;
use useful_netmod_bits::framing::{Envelope, FrameExt};

/// Wraps the pnet ethernet channel and the input queue
pub(crate) struct Socket {
    iface: NetworkInterface,
    tx: Arc<Mutex<Box<dyn DataLinkSender>>>,
    rx: Arc<Mutex<Box<dyn DataLinkReceiver>>>,
    inbox: Arc<RwLock<Notify<VecDeque<FrameExt>>>>,
}

const CUSTOM_ETHERTYPE: EtherType = EtherType(0xDE57);

impl Socket {
    /// Create a new socket handler and return a management reference
    pub(crate) async fn new(
        iface: NetworkInterface,
        table: Arc<AddrTable<MacAddr>>,
    ) -> Result<Arc<Self>, Box<dyn Error>> {
        let (tx, rx) = match channel(&iface, Default::default()) {
            Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => Err("Invalid channel type")?,
            Err(e) => Err(e)?,
        };

        let arc = Arc::new(Self {
            iface,
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
            inbox: Default::default(),
        });

        Self::incoming_handle(Arc::clone(&arc), table);
        arc.multicast(&Envelope::Announce).await;
        info!("Sent multicast announcement");
        Ok(arc)
    }

    /// Send a message to one specific client
    pub(crate) async fn send(&self, env: &Envelope, peer: MacAddr) {
        self.send_inner(env, peer).await;
    }

    /// Send a multicast with an Envelope
    pub(crate) async fn multicast(&self, env: &Envelope) {
        self.send_inner(env, MacAddr::broadcast()).await;
    }

    pub(crate) async fn send_multiple(&self, env: &Envelope, peers: &Vec<MacAddr>) {
        let mut tx = self.tx.lock_arc().await;

        let payload = env.as_bytes();
        let packet_size = payload.len() + EthernetPacket::minimum_packet_size();

        let mut index = 0;
        tx.build_and_send(peers.len(), packet_size, &mut |new_packet| {
            let mut new_packet = MutableEthernetPacket::new(new_packet).unwrap();

            new_packet.set_source(self.iface.mac.unwrap());

            new_packet.set_destination(peers[index]);
            index += 1;

            new_packet.set_ethertype(CUSTOM_ETHERTYPE);
            new_packet.set_payload(&payload);
        });
    }

    async fn send_inner(&self, env: &Envelope, peer: MacAddr) {
        let mut tx = self.tx.lock_arc().await;

        let payload = env.as_bytes();
        let packet_size = payload.len() + EthernetPacket::minimum_packet_size();

        tx.build_and_send(1, packet_size, &mut |new_packet| {
            let mut new_packet = MutableEthernetPacket::new(new_packet).unwrap();

            new_packet.set_source(self.iface.mac.unwrap());
            new_packet.set_destination(peer);
            new_packet.set_ethertype(CUSTOM_ETHERTYPE);
            new_packet.set_payload(&payload);
        });
    }

    pub(crate) async fn next(&self) -> FrameExt {
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
    fn incoming_handle(arc: Arc<Self>, table: Arc<AddrTable<MacAddr>>) {
        task::spawn_blocking(move || {
            loop {
                let mut rx = task::block_on(async { arc.rx.lock_arc().await });

                match rx.next() {
                    Ok(packet) => {
                        let packet = EthernetPacket::new(packet).unwrap();

                        if packet.get_ethertype() != CUSTOM_ETHERTYPE {
                            continue;
                        }

                        let peer = packet.get_source();

                        if peer == arc.iface.mac.unwrap() {
                            continue;
                        }

                        let buf = packet.payload().to_owned();

                        let env = Envelope::from_bytes(&buf);
                        task::block_on(async {
                            match env {
                                Envelope::Announce => {
                                    trace!("Receiving announce");
                                    table.set(peer).await;
                                    arc.multicast(&Envelope::Reply).await;
                                }
                                Envelope::Reply => {
                                    trace!("Receiving announce reply");
                                    table.set(peer).await;
                                }
                                Envelope::Data(buffer) => {
                                    trace!("Received data frame");
                                    let envelope = match InMemoryEnvelope::parse_from_buffer(buffer)
                                    {
                                        Ok(m) => m,
                                        Err(e) => {
                                            error!("failed to parse CarrierFrame header: {}", e);
                                            return;
                                        }
                                    };

                                    if let Some(id) = table.id(peer).await {
                                        // Append to the inbox and wake
                                        let mut inbox = arc.inbox.write().await;
                                        inbox.push_back(FrameExt(envelope, Target::Single(id)));
                                        Notify::wake(&mut inbox);
                                    }
                                }
                            }
                        })
                    }
                    Err(error) => {
                        //NOTE: See issue #86442. Nix hopefully won't be necessary for most of this
                        //in the future :D
                        match error.raw_os_error() {
                            Some(os_error) => match Errno::from_i32(os_error) {
                                Errno::ENETDOWN => {
                                    error!("Looks like the network is down! Please fix it.")
                                }
                                _ => panic!("{}", error),
                            },
                            None => panic!("{}", error),
                        }
                    }
                }
            }
        });
    }
}
