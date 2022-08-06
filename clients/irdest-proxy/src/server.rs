// SPDX-FileCopyrightText: 2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{
    config::{Config, InOrOut, IpSpace},
    inlet::Inlet,
    outlet::Outlet,
};
use async_std::{
    io::{self, ReadExt, WriteExt},
    net::{TcpListener, TcpStream},
    stream::StreamExt,
    sync::{Arc, RwLock},
    task,
};
use ratman_client::{Identity, RatmanIpc};
use std::collections::BTreeMap;

pub type SessionMap = Arc<RwLock<BTreeMap<Identity, TcpStream>>>;

/// The main proxy server state
pub struct Server {
    cfg: Config,
    map: SessionMap,
}

async fn connect_with_address(bind: Option<&str>, addr: Identity) -> io::Result<RatmanIpc> {
    Ok(match bind {
        Some(bind) => RatmanIpc::connect(bind, Some(addr)).await,
        None => RatmanIpc::default_with_addr(addr).await,
    }?)
}

async fn spawn_inwards(
    _: &Config,
    bind: Option<&str>,
    ip: &IpSpace,
    addr: Identity,
) -> io::Result<()> {
    let socket_addr = ip.socket_addr().clone();
    let tcp = TcpListener::bind(&socket_addr).await?;
    let ipc = connect_with_address(bind, addr).await?;

    task::spawn(async move {
        let mut inc = tcp.incoming();
        while let Some(stream) = inc.next().await {
            let mut stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    warn!("invalid stream tried to connect to {}: {}", socket_addr, e);
                    continue;
                }
            };

            let ipc = ipc.clone();
            task::spawn(async move {
                loop {
                    let mut buffer = vec![0; 1024];
                    if let Err(e) = stream.read(&mut buffer).await {
                        error!("failed to read from stream: {}", e);
                        break;
                    }

                    if let Err(e) = ipc.send_to(addr, buffer).await {
                        error!("failed to forward payload to IPC backend: {}", e);
                        break;
                    }
                }
            });
        }
    });

    Ok(())
}

async fn spawn_outwards(
    _: &Config,
    bind: Option<&str>,
    ip: &IpSpace,
    addr: Identity,
) -> io::Result<()> {
    let socket_addr = ip.socket_addr().clone();
    let mut tcp = TcpStream::connect(socket_addr).await?;
    let ipc = connect_with_address(bind, addr).await?;

    task::spawn(async move {
        while let Some((_, msg)) = ipc.next().await {
            if let Err(e) = tcp.write_all(&msg.get_payload()).await {
                error!("failed to forward data to {}: {}", socket_addr, e);
                break;
            }
        }
    });

    Ok(())
}

impl Server {
    pub fn new(cfg: Config) -> Self {
        Self {
            cfg,
            map: SessionMap::default(),
        }
    }

    /// Run this server
    pub async fn run(&self, bind: Option<&str>) {
        for (ip, (io, addr)) in self.cfg.map.iter() {
            debug!("Loading: {:?} // {:?} // {}", ip, io, addr);

            if let Err(e) = match io {
                InOrOut::In => Inlet::new(bind, ip, *addr),
                InOrOut::Out => Outlet::new(&self.map, bind, ip, *addr),
            } {
                error!(
                    "failed to initialise {}: {}",
                    match io {
                        InOrOut::In => "inward socket",
                        InOrOut::Out => "outward socket",
                    },
                    e
                );
            }
        }

        // wowow this is a hack ;_;
        async_std::future::pending().await
    }
}
