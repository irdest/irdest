// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Module only loaded when Ratman is running as a daemon

pub mod config;
mod parse;
mod peers;
pub mod pidfile;
pub mod startup;
mod state;
mod transform;

#[cfg(feature = "upnp")]
pub mod upnp;

#[cfg(not(feature = "upnp"))]
pub mod upnp {
    pub fn open_port(_: u16) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(feature = "dashboard")]
pub mod web;

#[cfg(not(feature = "dashboard"))]
pub mod web {
    use crate::Router;
    use prometheus_client::registry::Registry;
    pub async fn start(_: Router, _: Registry, _: &str, _: u16) -> async_std::io::Result<()> {
        Ok(())
    }
}

use std::net::SocketAddr;

use crate::{Message, Recipient, Router};
use async_std::{net::TcpListener, task::spawn};
use state::{DaemonState, OnlineMap};
use types::Result;

pub use peers::attach_peers;

async fn run_relay(r: Router, online: OnlineMap) {
    loop {
        let Message {
            id,
            sender,
            recipient,
            payload,
            timesig,
            sign,
        } = r.next().await;
        debug!("Receiving message for {:?}", recipient);
        let recv = types::api::receive_default(types::Message::received(
            id,
            sender,
            recipient.clone(),
            payload,
            format!("{:?}", timesig),
            sign,
        ));

        match recipient {
            ref recp @ Recipient::Standard(_) => {
                if let Some(Some(ref mut io)) = online
                    .lock()
                    .await
                    .get(&recp.scope().expect("empty recipient"))
                    .map(Clone::clone)
                {
                    info!("Forwarding message to online client!");
                    if let Err(e) = parse::forward_recv(io.as_io(), recv).await {
                        error!("Failed to forward received message: {}", e);
                    }
                }
            }
            Recipient::Flood(_) => {
                for (_, ref mut io) in online.lock().await.iter_mut() {
                    if io.is_none() && continue {}
                    if let Err(e) =
                        parse::forward_recv(io.as_mut().unwrap().as_io(), recv.clone()).await
                    {
                        error!("Failed to forward received message: {}", e);
                    }
                }
            }
        }
    }
}

/// Run the daemon!
pub async fn run(r: Router, addr: SocketAddr) -> Result<()> {
    info!("Listening for API connections on socket {:?}", addr);
    let listener = TcpListener::bind(addr).await?;
    let mut state = DaemonState::new(&listener, r.clone());
    let online = state.get_online().await;

    let relay = spawn(run_relay(r.clone(), online.clone()));

    while let Ok(io) = state.listen_for_connections().await {
        let (_self, io) = match io {
            Some(io) => io,
            None => continue,
        };

        info!("Established new client connection");
        spawn(parse::parse_stream(
            r.clone(),
            online.clone(),
            _self,
            io.clone(),
        ));
    }

    relay.cancel().await;
    Ok(())
}
