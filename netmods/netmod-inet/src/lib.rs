//! A tcp overlay netmod to connect router across the internet
#![allow(unused)]

use std::time::Duration;

use async_std::{io::WriteExt, net::TcpListener};
use serde::{Deserialize, Serialize};

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
    use async_std::net::{TcpListener, TcpStream};
    use async_std::prelude::*;

    let listener = TcpListener::bind("0.0.0.0:7000").await.unwrap();

    async_std::task::spawn(async move {
        async_std::task::sleep(Duration::from_millis(100)).await;

        let mut connection = TcpStream::connect("127.0.0.1:7000").await.unwrap();
        connection.write(&vec![]).await.unwrap();
    });

    let mut inc = listener.incoming();
    for stream in inc.next().await {
        let s = stream.unwrap();

        let mut len_buf = [0; 8];
        let peeked = s.peek(&mut len_buf).await.unwrap();
        println!("Peeked {} bytes...", peeked);
    }
}
