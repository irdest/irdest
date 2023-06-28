use super::{
    client::{BaseClient, OnlineClient},
    io::Io,
};
use libratman::types::{Address, Id};
use std::collections::BTreeMap;
use async_std::sync::Arc;

/// Handle new and existing connection states
pub(crate) struct ConnectionManager {
    clients: Arc<BTreeMap<Id, BaseClient>>,
}

impl ConnectionManager {
    pub fn new(clients: Arc<BTreeMap<Id, BaseClient>>) -> Self {
        Self { clients }
    }

    pub fn register(&mut self, initial_addr: Address, io: Io) -> OnlineClient {
        let id = Id::random();
        // self.clients.insert(id, BaseClient::register(initial_addr));
        // self.clients.get_mut(&id).unwrap().connect(io)
        todo!()
    }
}
