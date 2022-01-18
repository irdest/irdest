//! A tcp overlay netmod to connect router across the internet

#[macro_use]
extern crate tracing;

mod error;
mod io;
mod peer;
mod proto;
mod ptr;
mod resolve;
mod routes;
mod server;

pub use error::{Error, Result};

pub(crate) use io::IoPair;
pub(crate) use peer::{DstAddr, Peer, PeerState, SourceAddr};
pub(crate) use proto::{Packet, PacketBuilder};
pub(crate) use ptr::AtomPtr;
pub(crate) use resolve::Resolver;
pub(crate) use routes::Routes;
pub(crate) use server::{LockedStream, Server};

use async_std::{net::SocketAddr, sync::Arc};
use async_trait::async_trait;
use netmod::{self, Endpoint as EndpointExt, Frame, Target};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

/// Define the runtime mode for this endpount
///
/// In dynamic mode any new peer can introduce itself to start a link,
/// while in static mode only known peers will be accepted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Mode {
    Static,
    Dynamic,
}

/// Specify the conneciton types used by this node
///
/// By default netmod-tcp tries to establish bi-directional
/// connections, meaning that two nodes each have a dedicated
/// transmission (tx) and receiving (rx) channels.  However on some
/// networks this isn't possible.  While `Bidirect` is a good default,
/// it's possible to override this behaviour.
///
/// `Limited` will open connections to peers with a special flag that
/// makes it use a different reverse-channel strategy.  The server
/// won't try to create full reverse channels, and instead use the
/// incoming message stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LinkType {
    /// Default connection type
    Bidirect,
    /// Fallback connection type
    Limited,
}

impl Default for LinkType {
    fn default() -> Self {
        Self::Bidirect
    }
}

#[derive(Clone)]
pub struct Endpoint {
    pessimistic: Arc<AtomicBool>,
    server: Arc<Server>,
    routes: Arc<Routes>,
}

impl Endpoint {
    /// Create a new endpoint on an interface and port
    #[tracing::instrument(level = "info")]
    pub async fn new(bind: &str, name: &str, mode: Mode) -> Result<Arc<Self>> {
        info!("Initialising Tcp backend");

        let pessimistic = Arc::new(false.into());
        let socket: SocketAddr = bind.parse().map_err(|_| Error::InvalidAddr)?;
        let routes = Routes::new(socket.port());
        let server = Server::new(
            Arc::clone(&routes),
            socket,
            socket.port(),
            mode,
            Arc::clone(&pessimistic),
        )
        .await?;

        server.run();
        Ok(Arc::new(Self {
            pessimistic,
            server,
            routes,
        }))
    }

    /// Get the current runtime mode
    pub fn mode(&self) -> Mode {
        self.server.mode()
    }

    /// Get the port this netmod is bound to
    pub fn port(&self) -> u16 {
        self.server.port()
    }

    /// Mark this endpoint as "pessimistic"
    ///
    /// The "pessimistic" flag determines whether the default
    /// connection logic for peers should fall-back to a Limited
    /// connection type after a short timeout during which no reverse
    /// connection was established.
    pub fn pessimistic(&self) {
        self.pessimistic.fetch_or(true, Ordering::Relaxed);
    }

    pub async fn stop(&self) {
        self.server.stop();
        self.routes.stop_all().await;
    }

    /// Insert a set of peers into the routing table
    ///
    /// Each peer will spawn a worker that periodically attempts to
    /// connect to it.  Connections might not be recipricated if the
    /// peer doesn't know the local IP or is rejecting unknown
    /// connections.
    pub async fn add_peers(&self, peers: Vec<String>) -> Result<()> {
        for p in peers.into_iter() {
            if &p == "" && continue {}

            let peer = match Resolver::resolve(&p).await {
                Some(p) => p,
                None => {
                    warn!("Failed to parse peer: '{}'... skipping", p);
                    continue;
                }
            };

            trace!("Adding peer: {} ({})", peer, "",);
            self.routes.add_via_dst(peer, LinkType::Bidirect).await;
        }

        Ok(())
    }
}

#[async_trait]
impl EndpointExt for Endpoint {
    fn size_hint(&self) -> usize {
        0
    }

    async fn send(&self, frame: Frame, target: Target) -> netmod::Result<()> {
        match target {
            Target::Flood => {
                let dsts = self.routes.all_dst().await;
                for peer in dsts {
                    peer.send(Packet::Frame(frame.clone())).await;
                }
            }
            Target::Single(id) => {
                let peer = match self.routes.get_peer(id as usize).await {
                    Some(p) => Ok(p),
                    None => Err(netmod::Error::ConnectionLost),
                }?;
                peer.send(Packet::Frame(frame)).await;
            }
        }

        Ok(())
    }

    async fn next(&self) -> netmod::Result<(Frame, Target)> {
        Ok(self.server.next().await)
    }
}
