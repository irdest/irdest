// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{
    peer::{FrameSender, Peer},
    proto::{self, Handshake},
    routes::Routes,
    session::SessionData,
    PeerType,
};
use async_std::{
    net::{SocketAddr, TcpListener, TcpStream},
    stream::StreamExt,
    sync::Arc,
    task,
};
use libratman::{NetmodError, RatmanError};
use std::{io, net::AddrParseError};

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("unknown error (check your logs)")]
    Unknown,
    #[error("failed to parse server bind: {}", 0)]
    InvalidBind(AddrParseError),
    #[error("failed to bind socket: {}", 0)]
    Io(io::Error),
}

impl From<io::Error> for ServerError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

/// Tcp connection listener taking on connections from peers,
/// configuring links, and spawning async peer handlers
pub struct Server {
    ipv4_listen: Option<TcpListener>,
    ipv6_listen: TcpListener,
}

impl Server {
    /// Attempt to bind the server socket
    pub(crate) async fn bind(bind: &str) -> Result<Server, RatmanError> {
        let addr: SocketAddr = bind
            .parse()
            .map_err(|e: AddrParseError| RatmanError::Netmod(e.into()))?;
        if addr.is_ipv4() {
            error!("IPv4 binds are not supported (yet)");
            return Err(RatmanError::Netmod(NetmodError::NotSupported));
        }

        let ipv6_listen = TcpListener::bind(addr).await?;

        Ok(Self {
            ipv4_listen: None,
            ipv6_listen,
        })
    }

    /// Grab the port this socket is running on for diagnostics
    pub(crate) fn port(&self) -> u16 {
        self.ipv6_listen.local_addr().unwrap().port()
    }

    /// Run in a loop to accept incoming connections
    pub(crate) async fn run(self, sender: FrameSender, r: Arc<Routes>) {
        let mut inc = self.ipv6_listen.incoming();

        loop {
            let stream = inc.next().await;
            debug!("New incoming connection!");

            match stream {
                Some(Ok(s)) => {
                    let r = Arc::clone(&r);
                    task::spawn(handle_stream(s, sender.clone(), r));
                }
                Some(Err(e)) => {
                    warn!("Invalid incoming stream: {}", e);
                    continue;
                }
                None => {
                    warn!("Incoming stream is 'None'");
                    continue;
                }
            }
        }
    }
}

/// Handle incoming streams and setup Peers
///
/// Currently only standard peer connections are supported, meaning
/// that no reverse channel is created anywhere in this block.
async fn handle_stream(s: TcpStream, sender: FrameSender, r: Arc<Routes>) {
    let peer = match accept_connection(s, sender, &r).await {
        Ok(peer) => peer,
        Err(e) => {
            error!("Failed to connect to peer: {}", e);
            return;
        }
    };

    // Spawn a task to listen for packets for this peer
    let this_peer = Arc::clone(&peer);
    task::spawn(async move {
        this_peer.run().await;
    });

    // Also add the peer to the routing table
    r.add_peer(peer.id(), peer).await;
}

async fn accept_connection(
    mut s: TcpStream,
    sender: FrameSender,
    r: &Arc<Routes>,
) -> Result<Arc<Peer>, io::Error> {
    let addr = s.peer_addr()?;

    // First we read the handshake structure from the socket
    let frame = proto::read_blocking(&mut s).await.unwrap();
    let handshake = Handshake::from_carrier(&frame).unwrap();

    let tt = match handshake {
        Handshake::Hello { tt, .. } => tt,
        Handshake::Ack { .. } => {
            drop(s);
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid handshake data",
            ));
        }
    };

    // Send back an ACK to the client so it can chill out a bit
    let ack = Handshake::Ack { tt };
    let envelope = ack.to_carrier().unwrap();
    proto::write(&mut s, &envelope).await.unwrap();

    let target = r.next_target();
    let data = SessionData {
        self_port: 0,
        id: target,
        tt,
        addr,
    };

    info!(
        "Successfully connected with new peer #{} ({:?}) :)",
        target,
        s.peer_addr()
    );
    Ok(Peer::standard(data, sender, None, s))
}
