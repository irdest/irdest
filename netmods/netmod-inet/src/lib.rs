// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore
#![allow(warnings)]
//! A tcp overlay netmod to connect router across the internet

#[macro_use]
extern crate tracing;

mod peer;
mod proto;
mod resolve;
mod routes;
mod server;
mod session;

use peer::{FrameReceiver, FrameSender};
use routes::{Routes, Target};
use session::{setup_cleanuptask, start_connection, SessionData};
use {resolve::Resolver, server::Server};

use async_std::{channel::unbounded, io::WriteExt, net::TcpListener, sync::Arc, task};
use libratman::{
    netmod::{self, InMemoryEnvelope},
    types::RatmanError,
    NetmodError,
};
use serde::{Deserialize, Serialize};

/// The type of session being created
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum PeerType {
    /// Standard connections are client-server
    Standard,
    /// Cross connections are server-server
    Cross,
    /// Limited, one-way peering
    Limited(Direction),
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum Direction {
    Sending,
    Receiving,
}

/// Internet overlay endpoint for Ratman
pub struct InetEndpoint {
    port: u16,
    routes: Arc<Routes>,
    channel: (FrameSender, FrameReceiver),
}

impl InetEndpoint {
    /// Start a basic inet endpoint on a particular bind address
    pub async fn start(bind: &str) -> Result<Arc<Self>, RatmanError> {
        let server = Server::bind(bind).await?;
        let routes = Routes::new();
        let channel = unbounded(); // TODO: constraint the channel?
        let port = server.port(); // we don't store the server

        // Accept connections and spawn associated peers
        {
            let sender = channel.0.clone();
            task::spawn(server.run(sender, Arc::clone(&routes)));
        }

        Ok(Arc::new(Self {
            port,
            routes,
            channel,
        }))
    }

    /// Get the listening port for this server
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Insert a set of peers into the routing table
    ///
    /// Each peer will spawn a worker that periodically attempts to
    /// connect to it.  At the moment all connections are "Standard"
    /// connections as outlined in the user manual.
    async fn add_peer(&self, p: &str) -> Result<u16, RatmanError> {
        if p == "" {
            return Err(RatmanError::Netmod(NetmodError::InvalidPeer(p.into())));
        }

        let peer = match Resolver::resolve(&p).await {
            Some(p) => p,
            None => {
                warn!("Failed to parse peer: '{}'... skipping", p);
                return Err(RatmanError::Netmod(NetmodError::InvalidPeer(p.into())));
            }
        };

        trace!("Adding peer: {}", peer);
        let id = self.routes.next_target();
        let session_data = SessionData {
            id,
            tt: PeerType::Standard,
            addr: peer,
            self_port: 0, // not used
        };

        let routes = Arc::clone(&self.routes);
        let sender = self.channel.0.clone();
        match start_connection(session_data, Arc::clone(&routes), sender.clone()).await {
            Ok(rx) => setup_cleanuptask(rx, sender, &routes).await,
            Err(e) => {
                error!("failed to establish session with {}: {}", peer, e);
                return Err(RatmanError::Netmod(NetmodError::InvalidPeer(p.into())));
            }
        };

        Ok(id)
    }

    /// Send a single frame to a single friend
    ///
    /// Either the peer is currently active and we can send a message
    /// to it, or it is currently being restarted, and we queue
    /// something for it.
    ///
    pub async fn send(
        &self,
        target: Target,
        envelope: InMemoryEnvelope,
    ) -> Result<(), RatmanError> {
        let valid = self.routes.exists(target).await;
        if valid {
            trace!("Target {} exists {}", target, valid);
            let peer = self.routes.get_peer_by_id(target).await.unwrap();
            match peer.send(&envelope).await {
                // In case the connection was dropped, we remove the peer from the routing table
                Err(_) => {
                    let peer = self.routes.remove_peer(target).await;
                }
                _ => {}
            };
        } else {
            error!("Requested peer wasn't found: {:?}", target);
        }

        Ok(())
    }

    pub async fn send_all(
        &self,
        envelope: InMemoryEnvelope,
        exclude: Option<Target>,
    ) -> Result<(), RatmanError> {
        let all = self.routes.get_all_valid().await;
        for (peer, id) in all {
            match exclude {
                Some(exclude) if id == exclude => continue,
                _ => {}
            }

            if let Err(e) = peer.send(&envelope).await {
                error!("failed to send frame to peer {}: {}", peer.id(), e);
                self.routes.remove_peer(peer.id()).await;
            }
        }

        Ok(())
    }

    /// Get the next (Target, Frame) tuple from this endpoint
    // TODO: properly map error here
    pub async fn next(&self) -> Option<(Target, InMemoryEnvelope)> {
        self.channel.1.recv().await.ok()
    }
}

#[async_trait::async_trait]
impl netmod::Endpoint for InetEndpoint {
    async fn start_peering(&self, addr: &str) -> Result<u16, RatmanError> {
        self.add_peer(addr).await
    }

    /// Return a desired frame size in bytes
    ///
    /// A user of this library should use this metric to slice larger
    /// payloads into frame sequencies via the provided utilities.
    ///
    /// This metric is only a hint, and a router can choose to ignore
    /// it, if it then deals with possible "too large" errors during
    /// sending.  Choosing between a greedy or cautious approach to
    /// data slicing is left to the user of the interfaces.
    fn size_hint(&self) -> usize {
        0
    }

    /// Dispatch a `Frame` across this link
    ///
    /// Sending characteristics are entirely up to the implementation.
    /// As mentioned in the `size_hint()` documentation, this function
    /// **must not** panic on a `Frame` for size reasons, instead it
    /// should return `Error::FrameTooLarge`.
    ///
    /// The target ID is a way to instruct a netmod where to send a
    /// frame in a one-to-many mapping.  When implementing a
    /// one-to-one endpoint, this ID can be ignored (set to 0).
    async fn send(
        &self,
        envelope: InMemoryEnvelope,
        target: netmod::Target,
        exclude: Option<u16>,
    ) -> Result<(), RatmanError> {
        trace!("Sending message to {:?}", target);
        match target {
            netmod::Target::Single(target) => self.send(target, envelope).await?,
            netmod::Target::Flood(_) => self.send_all(envelope, exclude).await?,
        }

        Ok(())
    }

    /// Poll for the next available Frame from this interface
    ///
    /// It's recommended to return transmission errors, even if there
    /// are no ways to correct the situation from the router's POV,
    /// simply to feed packet drop metrics.
    async fn next(&self) -> Result<(InMemoryEnvelope, netmod::Target), RatmanError> {
        self.next()
            .await
            .ok_or(RatmanError::Netmod(NetmodError::ConnectionLost))
            .map(|(target, envelope)| (envelope, netmod::Target::Single(target)))
    }
}

#[async_std::test]
async fn simple_transmission() {
    ///////// "SERVER" SIDE

    pub fn setup_logging() {
        use tracing_subscriber::{filter::LevelFilter, fmt, EnvFilter};

        let filter = EnvFilter::default()
            .add_directive(LevelFilter::TRACE.into())
            .add_directive("async_io=error".parse().unwrap())
            .add_directive("async_std=error".parse().unwrap())
            .add_directive("mio=error".parse().unwrap())
            .add_directive("polling=error".parse().unwrap())
            .add_directive("tide=warn".parse().unwrap())
            .add_directive("trust_dns_proto=error".parse().unwrap())
            .add_directive("trust_dns_resolver=warn".parse().unwrap());

        // Initialise the logger
        fmt().with_env_filter(filter).init();
        info!("Initialised logger: welcome to ratmand!");
    }

    setup_logging();

    let server = InetEndpoint::start("[::]:12000").await.unwrap();

    let client = InetEndpoint::start("[::]:13000").await.unwrap();
    client.add_peer("[::1]:12000").await.unwrap();

    async_std::task::sleep(std::time::Duration::from_millis(1000)).await;
    info!("Waited for 1000ms, sending some data now");

    let data = InMemoryEnvelope::test_envelope();
    info!("============= SENDING =============");
    client.send(0, data.clone()).await.unwrap();
    info!("Data sent");

    let (_, received_data) = server.next().await.unwrap();
    info!("Data received!");

    assert_eq!(data, received_data);
}
