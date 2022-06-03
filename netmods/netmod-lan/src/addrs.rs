// SPDX-FileCopyrightText: 2019-2021 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2019-2021 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Address resolution table module

use async_std::{
    net::SocketAddrV6,
    sync::{Arc, RwLock},
};
use std::collections::BTreeMap;

/// A small utility that creates sequential IDs
struct IdMaker {
    last: Arc<RwLock<u16>>,
}

impl IdMaker {
    async fn curr(&self) -> u16 {
        *self.last.read().await
    }

    async fn incr(&self) -> &Self {
        *self.last.write().await += 1;
        self
    }
}

pub(crate) struct AddrTable {
    factory: IdMaker,
    ips: Arc<RwLock<BTreeMap<u16, SocketAddrV6>>>,
    ids: Arc<RwLock<BTreeMap<SocketAddrV6, u16>>>,
}

impl AddrTable {
    /// Create a new address lookup table
    pub(crate) fn new() -> Self {
        Self {
            factory: IdMaker {
                last: Default::default(),
            },
            ips: Default::default(),
            ids: Default::default(),
        }
    }

    /// Insert a given IP into the table, returning it's ID
    ///
    /// Topology changes are handled additively, because it's not
    /// possible to find out what previous IP a node had, without
    /// performing deep packet inspection and looking at certain
    /// Identity information.  As such, this table can only grow.
    pub(crate) async fn set(&self, i: SocketAddrV6) -> u16 {
        let id = self.factory.incr().await.curr().await;
        let peer = i.into();
        self.ips.write().await.insert(id, peer);
        self.ids.write().await.insert(peer, id);
        id
    }

    /// Get the ID for a given Peer address
    pub(crate) async fn id(&self, peer: SocketAddrV6) -> Option<u16> {
        self.ids.read().await.get(&peer).cloned()
    }

    /// Get the Peer for a given internal ID
    pub(crate) async fn ip(&self, id: u16) -> Option<SocketAddrV6> {
        self.ips.read().await.get(&id).cloned()
    }

    #[allow(unused)]
    pub(crate) async fn all(&self) -> Vec<SocketAddrV6> {
        self.ips.read().await.values().cloned().collect()
    }
}
