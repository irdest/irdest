use std::collections::BTreeMap;
use libratman::types::{Address, Id};

/// Represents an application connected to the Ratman API
#[derive(Default)]
pub struct OnlineClient {
    /// A list of addresses
    ///
    /// The first address in the list is considered the "default"
    /// address for this client.
    addrs: Vec<Address>,
}

impl OnlineClient {
    
}

pub type OnlineClientMap = BTreeMap<Id, OnlineClient>;
