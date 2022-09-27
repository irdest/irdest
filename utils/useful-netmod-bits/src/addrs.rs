// SPDX-FileCopyrightText: 2019-2021 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2019-2021 Yureka Lilian <yuka@yuka.dev>
// SPDX-FileCopyrightText: 2022 Christopher A. Grant <grantchristophera@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Address resolution table module

use async_std::sync::{Arc, RwLock};
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

pub struct AddrTable<T> {
    factory: IdMaker,
    addrs: Arc<RwLock<BTreeMap<u16, T>>>,
    ids: Arc<RwLock<BTreeMap<T, u16>>>,
}

impl<T> AddrTable<T>
where
    T: Default + Copy + Ord,
{
    /// Create a new address lookup table
    pub fn new() -> Self {
        Self {
            factory: IdMaker {
                last: Default::default(),
            },
            addrs: Default::default(),
            ids: Default::default(),
        }
    }

    /// Insert a given Ethernet address into the table, returning its ID
    ///
    /// Topology changes are handled additively.
    pub async fn set(&self, addr: T) -> u16 {
        let id = self.factory.incr().await.curr().await;
        let peer = addr.into();
        self.addrs.write().await.insert(id, peer);
        self.ids.write().await.insert(peer, id);
        id
    }

    /// Get the ID for a given Peer address
    pub async fn id(&self, peer: T) -> Option<u16> {
        self.ids.read().await.get(&peer).cloned()
    }

    /// Get the Peer for a given internal ID
    pub async fn addr(&self, id: u16) -> Option<T> {
        self.addrs.read().await.get(&id).cloned()
    }

    #[allow(unused)]
    pub async fn all(&self) -> Vec<T> {
        self.addrs.read().await.values().cloned().collect()
    }
}
