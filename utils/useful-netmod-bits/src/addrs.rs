// SPDX-FileCopyrightText: 2019-2021 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2019-2021 Yureka Lilian <yuka@yuka.dev>
// SPDX-FileCopyrightText: 2022 Christopher A. Grant <grantchristophera@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Address resolution table module

use libratman::{tokio::sync::RwLock, types::Ident32};
use std::{collections::BTreeMap, sync::Arc};

pub struct AddrTable<T> {
    addrs: Arc<RwLock<BTreeMap<Ident32, T>>>,
    ids: Arc<RwLock<BTreeMap<T, Ident32>>>,
}

impl<T> AddrTable<T>
where
    T: Default + Copy + Ord,
{
    /// Create a new address lookup table
    pub fn new() -> Self {
        Self {
            addrs: Default::default(),
            ids: Default::default(),
        }
    }

    /// Insert a given Ethernet address into the table, returning its ID
    ///
    /// Topology changes are handled additively.
    pub async fn set(&self, addr: T, pk_id: Ident32) {
        let peer = addr.into();
        self.addrs.write().await.insert(pk_id, peer);
        self.ids.write().await.insert(peer, pk_id);
    }

    /// Get the ID for a given Peer address
    pub async fn id(&self, peer: T) -> Option<Ident32> {
        self.ids.read().await.get(&peer).cloned()
    }

    /// Get the Peer for a given internal ID
    pub async fn addr(&self, id: Ident32) -> Option<T> {
        self.addrs.read().await.get(&id).cloned()
    }

    #[allow(unused)]
    pub async fn all(&self) -> Vec<T> {
        self.addrs.read().await.values().cloned().collect()
    }
}
