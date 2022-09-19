// SPDX-FileCopyrightText: 2019-2021 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Socket handler module

use useful_netmod_bits::addrs::AddrTable;
use useful_netmod_bits::framing::{Envelope, FrameExt};

use async_std::{
    future::{self, Future},
    io::WriteExt,
    pin::Pin,
    sync::{Arc, Mutex, RwLock},
    task::{self, Poll},
};
use pnet::{
    packet::ethernet::{EtherType, EthernetPacket, MutableEthernetPacket},
    packet::Packet,
    util::MacAddr,
};
use pnet_datalink::{channel, Channel, DataLinkReceiver, DataLinkSender, NetworkInterface};

use netmod::Target;
use std::collections::VecDeque;
use std::error::Error;
use task_notify::Notify;

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
            iface: iface,
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
            inbox: Default::default(),
        });

        Self::incoming_handle(Arc::clone(&arc), table); //NOTE: I am not certain that cloning the
                                                        //ethernet channel is wise, but we are only ever
                                                        //either recieving or transmitting.
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

    pub(crate) async fn send_multiple(
        &self,
        env: &Envelope,
        peers: &Vec<MacAddr>,
        exclude: MacAddr,
    ) {
        let mut tx = self.tx.lock_arc().await;

        let payload = env.as_bytes();
        let packet_size = payload.len() + EthernetPacket::minimum_packet_size();

        let mut index = 0;

        tx.build_and_send(peers.len(), packet_size, &mut |new_packet| {
            let mut new_packet = MutableEthernetPacket::new(new_packet).unwrap();

            new_packet.set_source(self.iface.mac.unwrap());

            let mut dest = peers[index];
            if dest == exclude {
                index += 1;
                dest = peers[index];
            }

            new_packet.set_destination(dest);
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
        task::spawn(async move {
            dbg!("Spawned raw handler");
            loop {
                let mut buf = vec![0; 1500];

                let mut rx = arc.rx.lock_arc().await;

                match rx.next() {
                    //TODO: this probably blocks
                    Ok(packet) => {
                        let packet = EthernetPacket::new(packet).unwrap();

                        if packet.get_ethertype() != CUSTOM_ETHERTYPE {
                            // Immediately filter irrelevant
                            // packets. TODO: Check if pnet
                            // has a better way to do this.
                            continue;
                        }

                        dbg!(&packet);

                        let peer = packet.get_source();

                        match buf.write_all(packet.payload()).await {
                            Ok(()) => (),
                            Err(_) => {
                                warn!("Failed to write packet to buffer. Supported MTU is 1500.")
                            } //NOTE: Is this reasonable?
                        };

                        let env = Envelope::from_bytes(&buf);
                        match env {
                            Envelope::Announce => {
                                trace!("Recieving announce");
                                table.set(peer).await;
                                arc.multicast(&Envelope::Reply).await;
                            }
                            Envelope::Reply => {
                                trace!("Recieving announce reply");
                                table.set(peer).await;
                            }
                            Envelope::Data(vec) => {
                                trace!("Recieved data frame");
                                let frame = bincode::deserialize(&vec).unwrap();
                                let id = table.id(peer.into()).await.unwrap();

                                // Append to the inbox and wake
                                let mut inbox = arc.inbox.write().await;
                                inbox.push_back(FrameExt(frame, Target::Single(id)));
                                Notify::wake(&mut inbox);
                            }
                        }
                    }
                    val => {
                        // TODO: handle errors more gracefully
                        // This will occur if the network goes down.
                        error!("Crashed raw thread: {:#?}", val);
                        val.expect("Crashed raw thread!");
                    }
                }
            }
        });
    }
}
