// SPDX-FileCopyrightText: 2019-2021 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2019-2021 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Address resolution table module

use libratman::tokio::sync::RwLock;
use libratman::types::Ident32;
use std::{collections::BTreeMap, net::SocketAddrV6, sync::Arc};

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
    ips: Arc<RwLock<BTreeMap<Ident32, SocketAddrV6>>>,
    ids: Arc<RwLock<BTreeMap<SocketAddrV6, Ident32>>>,
}

impl AddrTable {
    /// Create a new address lookup table
    pub(crate) fn new() -> Self {
        Self {
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
    pub(crate) async fn set(&self, i: SocketAddrV6, peer_rk_id: Ident32) {
        let peer = i.into();
        self.ips.write().await.insert(peer_rk_id, peer);
        self.ids.write().await.insert(peer, peer_rk_id);
    }

    /// Get the ID for a given Peer address
    pub(crate) async fn id(&self, peer: SocketAddrV6) -> Option<Ident32> {
        self.ids.read().await.get(&peer).cloned()
    }

    /// Get the Peer for a given internal ID
    pub(crate) async fn ip(&self, id: Ident32) -> Option<SocketAddrV6> {
        self.ips.read().await.get(&id).cloned()
    }

    #[allow(unused)]
    pub(crate) async fn all(&self) -> Vec<SocketAddrV6> {
        self.ips.read().await.values().cloned().collect()
    }
}
