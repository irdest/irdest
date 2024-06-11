use chrono::{DateTime, Utc};
use libratman::{
    tokio::sync::{Mutex, MutexGuard},
    types::{Address, Ident32},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(crate) struct ConnectionManager {
    /// A map of client_id -> client metadata
    inner: Mutex<BTreeMap<Ident32, RouterClient>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(BTreeMap::default()),
        }
    }

    pub async fn lock<'a>(&'a self) -> MutexGuard<'a, BTreeMap<Ident32, RouterClient>> {
        self.inner.lock().await
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
