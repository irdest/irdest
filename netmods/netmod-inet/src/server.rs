// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{
    peer::{FrameSender, Peer},
    proto::{self, Handshake},
    routes::Routes,
    session::SessionData,
};
use libratman::{
    tokio::{
        net::{TcpListener, TcpStream},
        task::spawn_local,
    },
    types::Ident32,
    NetmodError, RatmanError,
};
use std::{
    io,
    net::{AddrParseError, SocketAddr},
    sync::Arc,
};

#[allow(unused)]
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
    self_router_key_id: Ident32,
    #[allow(unused)]
    ipv4_listen: Option<TcpListener>,
    ipv6_listen: TcpListener,
}

impl Server {
    /// Attempt to bind the server socket
    pub(crate) async fn bind(
        bind: &str,
        self_router_key_id: Ident32,
    ) -> Result<Server, RatmanError> {
        let addr: SocketAddr = bind
            .parse()
            .map_err(|e: AddrParseError| RatmanError::Netmod(e.into()))?;
        if addr.is_ipv4() {
            error!("IPv4 binds are not supported (yet)");
            return Err(RatmanError::Netmod(NetmodError::NotSupported));
        }

        let ipv6_listen = TcpListener::bind(addr).await?;

        Ok(Self {
            self_router_key_id,
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
        loop {
            match self.ipv6_listen.accept().await {
                Ok((stream, _)) => {
                    let r = Arc::clone(&r);
                    spawn_local(handle_stream(
                        stream,
                        sender.clone(),
                        r,
                        self.self_router_key_id,
                    ));
                }
                Err(e) => {
                    warn!("Invalid incoming stream: {}", e);
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
async fn handle_stream(s: TcpStream, sender: FrameSender, r: Arc<Routes>, self_key_id: Ident32) {
    let peer = match accept_connection(s, sender, &r, self_key_id).await {
        Ok(peer) => peer,
        Err(e) => {
            error!("Failed to connect to peer: {}", e);
            return;
        }
    };

    // Spawn a task to listen for packets for this peer
    let this_peer = Arc::clone(&peer);
    spawn_local(async move { this_peer.run().await });

    // Also add the peer to the routing table
    r.add_peer(peer.session.peer_router_key_id, peer).await;
}

async fn accept_connection(
    s: TcpStream,
    sender: FrameSender,
    r: &Arc<Routes>,
    self_key_id: Ident32,
) -> Result<Arc<Peer>, io::Error> {
    let addr = s.peer_addr()?;
    let (mut read_stream, mut write_stream) = s.into_split();

    // First we read the handshake structure from the socket
    let frame = proto::read_blocking(&mut read_stream).await.unwrap();
    let handshake = Handshake::from_carrier(&frame).unwrap();

    let (tt, r_key_id) = match handshake {
        Handshake::Hello { tt, r_key_id, .. } => (tt, r_key_id),
        Handshake::Ack { .. } => {
            drop((read_stream, write_stream));
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid handshake data",
            ));
        }
    };

    // Send back an ACK to the client so it can chill out a bit
    let ack = Handshake::Ack {
        tt,
        r_key_id: self_key_id,
    };
    let envelope = ack.to_carrier().unwrap();
    proto::write(&mut write_stream, &envelope).await.unwrap();

    let target = r.next_target();
    let data = SessionData {
        self_port: 0,
        self_router_key_id: self_key_id,
        peer_router_key_id: r_key_id,
        id: target,
        tt,
        addr,
    };

    info!(
        "Successfully connected with new peer #{} ({:?}) :)",
        target, addr,
    );
    Ok(Peer::standard(
        data,
        sender,
        None,
        write_stream,
        read_stream,
    ))
}
