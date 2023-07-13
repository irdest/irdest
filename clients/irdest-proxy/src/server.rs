// SPDX-FileCopyrightText: 2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{
    config::{Config, InOrOut, Routes},
    inlet::Inlet,
    outlet::Outlet,
};
use async_std::{
    net::TcpStream,
    sync::{Arc, RwLock},
};
use libratman::client::Address;
use std::collections::BTreeMap;

pub type SessionMap = Arc<RwLock<BTreeMap<Address, TcpStream>>>;

/// The main proxy server state
pub struct Server {
    cfg: Config,
    routes: Routes,
    map: SessionMap,
}

impl Server {
    pub fn new(cfg: Config, routes: Routes) -> Self {
        Self {
            cfg,
            routes,
            map: SessionMap::default(),
        }
    }

    /// Run this server
    pub async fn run(&self, bind: Option<&str>) {
        for (ip, (io, addr)) in self.routes.iter() {
            debug!("Loading: {:?} // {:?} // {}", ip, io, addr);

            if let Err(e) = match io {
                InOrOut::In => Inlet::new(bind, ip, *addr, self.cfg.get_address(&ip)),
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
