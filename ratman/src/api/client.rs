use super::io::Io;
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
    /// A secret (ish) token that must be provided by this client on
    /// every future connection handshake
    token: Id,
    /// A list of addresses
    ///
    /// The first address in the list is considered the "default"
    /// address for this client.
    addrs: Vec<Address>,
    /// Last connection timestamp
    ///
    /// If the client is currently connected this time refers to the
    /// connection handshake timestamp (i.e. how long has the client
    /// been connected).  If the client is not currently connected it
    /// refers to the connection close/ drop timestamp (i.e. since
    /// when has the client been disconnected).
    last_connection: DateTime<Utc>,
}

impl BaseClient {
    /// Register a new BaseClient with its first known address and the
    /// current time for the connection timestamp.
    pub(crate) fn register(first_addr: Address) -> Self {
        Self {
            token: Id::random(),
            addrs: vec![first_addr],
            last_connection: Utc::now(),
        }
    }

    /// Take an existing BaseClient and augment it with an I/O socket
    pub(crate) fn connect(&mut self, io: Io) -> OnlineClient {
        OnlineClient { base: self, io }
    }
}

/// Represents an application connected to the Ratman API
pub struct OnlineClient<'base> {
    /// An online client consists of a corresponding base client
    base: &'base mut BaseClient,
    /// Hold the current connection socket
    io: Io,
}

impl<'base> OnlineClient<'base> {}

pub type BaseClientMap = BTreeMap<Id, BaseClient>;
pub type OnlineClientMap<'base> = BTreeMap<Id, OnlineClient<'base>>;
