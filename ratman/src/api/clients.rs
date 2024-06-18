// SPDX-FileCopyrightText: 2023-2024 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use async_eris::ReadCapability;
use chrono::{DateTime, Utc};
use libratman::{
    tokio::sync::{broadcast::Sender as BcastSender, Mutex, MutexGuard},
    types::{AddrAuth, Address, Ident32, LetterheadV1, Recipient},
    NonfatalError, RatmanError, Result,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type ActiveAuth = Mutex<BTreeMap<AddrAuth, Address>>;

pub type AuthGuard = Mutex<BTreeMap<AddrAuth, Address>>;

pub(crate) struct ConnectionManager {
    /// A map of client_id -> client metadata
    inner: Mutex<BTreeMap<Ident32, RouterClient>>,
    sync_listeners: Mutex<BTreeMap<Recipient, BcastSender<(LetterheadV1, ReadCapability)>>>,
    active_auth: Mutex<BTreeMap<AddrAuth, Address>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(BTreeMap::default()),
            sync_listeners: Mutex::new(BTreeMap::default()),
            active_auth: Mutex::new(BTreeMap::default()),
        }
    }

    pub async fn lock_inner<'a>(&'a self) -> MutexGuard<'a, BTreeMap<Ident32, RouterClient>> {
        self.inner.lock().await
    }

    pub fn active_auth(&self) -> &Mutex<BTreeMap<AddrAuth, Address>> {
        &self.active_auth
    }

    pub async fn insert_sync_listener(
        &self,
        recipient: Recipient,
        tx: BcastSender<(LetterheadV1, ReadCapability)>,
    ) {
        let mut sync_inner = self.sync_listeners.lock().await;
        sync_inner.insert(recipient, tx);
    }

    pub async fn remove_sync_listener(&self, recipient: Recipient) {
        let mut sync_inner = self.sync_listeners.lock().await;
        sync_inner.remove(&recipient);
    }

    pub async fn get_sync_listeners(
        &self,
        recipient: Recipient,
    ) -> Result<BcastSender<(LetterheadV1, ReadCapability)>> {
        self.sync_listeners
            .lock()
            .await
            .get(&recipient)
            .map(|vec| vec.clone())
            .ok_or(RatmanError::Nonfatal(NonfatalError::NoStream))
    }

    pub async fn client_exists_for_address(&self, addr: Address) -> bool {
        self.inner
            .lock()
            .await
            .iter()
            .find(|(_, client)| {
                client
                    .addrs
                    .iter()
                    .find(|client_addr| client_addr == &&addr)
                    .is_some()
            })
            .is_some()
    }
}

/// Represent an API client (application)'s base state
///
/// In this state Ratman knows about its set of registered addresses
/// and a secret token that must be provided on future handshakes, but
/// doesn't assume any ongoing connection details.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct RouterClient {
    /// A list of addresses
    ///
    /// The first address in the list is considered the "default" address for
    /// this client.  Each address has an associated "auth" token which prevents
    /// any client not in possesion of the address secret from using it
    pub(crate) addrs: Vec<Address>,
    /// Last connection timestamp
    pub(crate) last_connection: DateTime<Utc>,
}

impl RouterClient {
    pub(crate) fn add_address(&mut self, id: Address) {
        self.addrs.push(id);
    }

    /// Gets the primary address for a given client
    pub(crate) fn primary_address(&self) -> Address {
        *self
            .addrs
            .get(0)
            .expect("Router client had no primary address")
    }

    pub(crate) fn all_addrs(&self) -> Vec<Address> {
        self.addrs.clone()
    }
}
