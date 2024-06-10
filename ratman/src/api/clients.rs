use std::{collections::BTreeMap, sync::Arc};

use crate::{crypto::Keypair, util::IoPair};
use atomptr::AtomPtr;
use chrono::{DateTime, Utc};
use libratman::types::{Address, ClientAuth, Id};
use serde::{Deserialize, Serialize};

pub(crate) struct ConnectionManager {
    inner: BTreeMap<Id, Arc<RouterClient>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::default(),
        }
    }
}

/// Represent an API client (application)'s base state
///
/// In this state Ratman knows about its set of registered addresses
/// and a secret token that must be provided on future handshakes, but
/// doesn't assume any ongoing connection details.
#[derive(Clone, Serialize, Deserialize)]
pub struct RouterClient {
    /// A list of addresses
    ///
    /// The first address in the list is considered the "default" address for
    /// this client.  Each address has an associated "auth" token which prevents
    /// any client not in possesion of the address secret from using it
    pub(crate) addrs: Vec<(Address, ClientAuth)>,
    /// Last connection timestamp
    pub(crate) last_connection: DateTime<Utc>,
}

impl RouterClient {
    /// Register a new BaseClient with its first known address and the
    /// current time for the connection timestamp.
    pub(crate) fn new_with_registry(id: Address, auth: ClientAuth) -> Arc<Self> {
        Arc::new(Self {
            addrs: vec![(id, auth)],
            last_connection: Utc::now(),
        })
    }

    pub(crate) fn add_address(&mut self, id: Address, auth: ClientAuth) {
        self.addrs.push((id, auth));
    }

    /// Gets the primary address for a given client
    pub(crate) fn primary_address(&self) -> Address {
        self.addrs
            .get(0)
            .expect("Router client had no primary address")
            .0
    }

    pub(crate) fn all_addrs(&self) -> Vec<Address> {
        self.addrs.iter().map(|(a, _)| *a).collect()
    }
}
