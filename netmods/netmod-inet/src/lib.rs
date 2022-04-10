//! A tcp overlay netmod to connect router across the internet
#![allow(unused)]

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
use netmod::{Error, Frame};
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

#[derive(Debug, thiserror::Error)]
pub enum InetError {
    #[error("failed configuring the TCP server: {}", 0)]
    Server(server::ServerError),
    #[error("provided invalid settings to inet")]
    Settings,
}

impl From<InetError> for Error {
    fn from(_: InetError) -> Error {
        Error::ConnectionLost
    }
}

impl From<server::ServerError> for InetError {
    fn from(err: server::ServerError) -> Self {
        Self::Server(err)
    }
}

/// Internet overlay endpoint for Ratman
pub struct InetEndpoint {
    port: u16,
    routes: Arc<Routes>,
    channel: (FrameSender, FrameReceiver),
}

impl InetEndpoint {
    /// Start a basic inet endpoint on a particular bind address
    pub async fn start(bind: &str) -> Result<Arc<Self>, InetError> {
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
    pub async fn add_peers(&self, peers: Vec<String>) -> Result<(), InetError> {
        for p in peers.into_iter() {
            if &p == "" && continue {}

            let peer = match Resolver::resolve(&p).await {
                Some(p) => p,
                None => {
                    warn!("Failed to parse peer: '{}'... skipping", p);
                    continue;
                }
            };

            trace!("Adding peer: {}", peer);
            let session_data = SessionData {
                id: self.routes.next_target(),
                tt: PeerType::Standard,
                addr: peer,
                self_port: 0, // not used
            };

            {
                let routes = Arc::clone(&self.routes);
                let sender = self.channel.0.clone();
                match start_connection(session_data, Arc::clone(&routes), sender.clone()).await {
                    Ok(rx) => setup_cleanuptask(rx, sender, &routes).await,
                    Err(e) => {
                        error!("failed to establish session with {}: {}", peer, e);
                        continue;
                    }
                };
            }
        }

        Ok(())
    }

    /// Send a single frame to a single friend
    ///
    /// Either the peer is currently active and we can send a message
    /// to it, or it is currently being restarted, and we queue
    /// something for it.
    ///
    pub async fn send(&self, target: Target, frame: Frame) -> Result<(), InetError> {
        let valid = self.routes.exists(target).await;
        if valid {
            trace!("Target {} exists {}", target, valid);
            let peer = self.routes.get_peer_by_id(target).await.unwrap();
            match peer.send(&frame).await {
                // In case the connection was dropped, we remove the peer from the routing table
                Err(_) => {
                    let peer = self.routes.remove_peer(target).await;
                }
                _ => {}
            };
        }

        Ok(())
    }

    pub async fn send_all(&self, frame: Frame) -> Result<(), InetError> {
        let all = self.routes.get_all_valid().await;
        for peer in all {
            if let Err(e) = peer.send(&frame).await {
                error!("failed to send frame to peer {}: {}", peer.id(), e);
                self.routes.remove_peer(peer.id()).await;
            }
        }

        Ok(())
    }

    /// Get the next (Target, Frame) tuple from this endpoint
    // TODO: properly map error here
    pub async fn next(&self) -> Option<(Target, Frame)> {
        self.channel.1.recv().await.ok()
    }
}

#[async_trait::async_trait]
impl netmod::Endpoint for InetEndpoint {
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
    async fn send(&self, frame: Frame, target: netmod::Target) -> Result<(), Error> {
        trace!("Sending message to {:?}", target);
        match target {
            netmod::Target::Single(target) => self.send(target, frame).await?,
            netmod::Target::Flood(_) => self.send_all(frame).await?,
        }

        Ok(())
    }

    /// Poll for the next available Frame from this interface
    ///
    /// It's recommended to return transmission errors, even if there
    /// are no ways to correct the situation from the router's POV,
    /// simply to feed packet drop metrics.
    async fn next(&self) -> Result<(Frame, netmod::Target), Error> {
        self.next()
            .await
            .ok_or(Error::ConnectionLost)
            .map(|(target, frame)| (frame, netmod::Target::Single(target)))
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
    client
        .add_peers(vec!["[::1]:12000".to_string()])
        .await
        .unwrap();

    let data = Frame::dummy();
    client.send(0, data.clone()).await.unwrap();

    let (_, received_data) = server.next().await.unwrap();

    assert_eq!(data, received_data);
}
