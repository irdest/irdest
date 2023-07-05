use super::{
    client::{BaseClient, OnlineClient},
    io::Io,
};
use async_std::sync::Arc;
use libratman::types::{Address, Id};
use std::collections::BTreeMap;

/// Handle new and existing connection states
pub(crate) struct ConnectionManager {
    pub(crate) clients: Arc<BTreeMap<Id, BaseClient>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(BTreeMap::new()),
        }
    }

    pub fn register(&mut self, initial_addr: Address, io: Io) -> OnlineClient {
        let id = Id::random();
        // self.clients.insert(id, BaseClient::register(initial_addr));
        // self.clients.get_mut(&id).unwrap().connect(io)
        todo!()
    }
}
