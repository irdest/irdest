use crate::storage::{addrs::StorageAddress, client::StorageClient};

use super::io::Io;
use async_std::sync::Arc;
use atomptr::AtomPtr;
use chrono::{DateTime, Utc};
use libratman::types::{Address, Id};
use std::collections::BTreeMap;

/// Represent an API client (application)'s base state
///
/// In this state Ratman knows about its set of registered addresses
/// and a secret token that must be provided on future handshakes, but
/// doesn't assume any ongoing connection details.
#[derive(Clone)]
pub struct BaseClient {
    /// Anonymous clients MUST be filtered when saving known clients
    pub(crate) anonymous: bool,

    /// A secret (ish) token that must be provided by this client on
    /// every future connection handshake
    // TODO: make it so that this doesn't have to be accessible from
    // everywhere in the daemon
    pub(crate) token: Id,
    /// A list of addresses
    ///
    /// The first address in the list is considered the "default"
    /// address for this client.
    pub(crate) addrs: Vec<StorageAddress>,
    /// Last connection timestamp
    ///
    /// If the client is currently connected this time refers to the
    /// connection handshake timestamp (i.e. how long has the client
    /// been connected).  If the client is not currently connected it
    /// refers to the connection close/ drop timestamp (i.e. since
    /// when has the client been disconnected).
    pub(crate) last_connection: AtomPtr<DateTime<Utc>>,
}

impl BaseClient {
    #[inline]
    fn new(addrs: Vec<StorageAddress>, anonymous: bool) -> Arc<Self> {
        Arc::new(Self {
            anonymous,
            addrs,
            token: Id::random(),
            last_connection: AtomPtr::new(Utc::now()),
        })
    }

    /// Load an existing client (StorageClient)
    // TODO: make this function zero-copy ?
    pub(crate) fn existing(
        StorageClient {
            id: _,
            token,
            addrs,
            last_connection,
        }: &StorageClient,
    ) -> Arc<Self> {
        Arc::new(Self {
            anonymous: false,
            addrs: addrs.clone(),
            token: *token,
            last_connection: AtomPtr::new(*last_connection),
        })
    }

    /// Register a new BaseClient with its first known address and the
    /// current time for the connection timestamp.
    pub(crate) fn register(first_addr: Address) -> Arc<Self> {
        Self::new(vec![StorageAddress::new(first_addr, &[])], false)
    }

    /// Create a new anonymous base client
    pub(crate) fn anonymous() -> Arc<Self> {
        Self::new(vec![], true)
    }

    /// Gets the primary address for a given client
    pub(crate) fn primary_address(&self) -> Address {
        self.addrs
            .get(0)
            .expect("BaseClient had no primary address")
            .id
    }

    /// Take an existing BaseClient and augment it with an I/O socket
    pub(crate) fn connect(self: &Arc<Self>, io: Io) -> OnlineClient {
        OnlineClient {
            base: Arc::clone(self),
            io,
        }
    }
}

/// Represents an application connected to the Ratman API
pub struct OnlineClient {
    /// An online client consists of a corresponding base client
    pub(crate) base: Arc<BaseClient>,
    /// Hold the current connection socket
    pub(crate) io: Io,
}

impl OnlineClient {}

pub type BaseClientMap = BTreeMap<Id, Arc<BaseClient>>;
pub type OnlineClientMap = BTreeMap<Id, OnlineClient>;
