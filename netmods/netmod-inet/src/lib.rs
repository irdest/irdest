//! A tcp overlay netmod to connect router across the internet
#![allow(unused)]

use std::time::Duration;

use async_std::{channel::unbounded, io::WriteExt, net::TcpListener, task};
use netmod::Frame;
use routes::Routes;
use serde::{Deserialize, Serialize};
use server::Server;
use session::{SessionData, SessionManager};

#[macro_use]
extern crate tracing;

mod peer;
mod proto;
mod routes;
mod server;
mod session;

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
    routes: routes::Routes,
}

#[async_std::test]
async fn blorp() {
    ///////// "SERVER" SIDE

    let server = Server::bind("[::]", 12000).await.unwrap();
    let a_routes = Routes::new();
    let (a_tx, a_rx) = unbounded();

    // Spawn the server to listen to connections
    task::spawn(server.run(a_tx, a_routes.clone()));

    ////////// "CLIENT" SIDE
    let b_routes = Routes::new();
    let (b_tx, b_rx) = unbounded();

    let session_data = SessionData {
        id: b_routes.next_target(),
        tt: PeerType::Standard,
        addr: "[::1]:12000".parse().unwrap(),
        self_port: 0,
    };

    let mut ctr = 0;
    let tcp_stream = SessionManager::connect(&mut ctr, &session_data)
        .await
        .unwrap();
    let peer = SessionManager::handshake(&session_data, b_tx.clone(), tcp_stream)
        .await
        .unwrap();
    b_routes.add_peer(peer.id(), peer).await;

    ////////// TESTING TIME BABAY

    let dummy = Frame::dummy();

    {
        // Node A
        let (target, frame) = (0, dummy.clone());

        let peer = b_routes.get_peer_by_id(target).await.unwrap();
        peer.send(&frame).await.unwrap();
    }

    {
        // Node B
        let recv_frame = a_rx.recv().await.unwrap().1;
        assert_eq!(recv_frame, dummy);
    }
}
