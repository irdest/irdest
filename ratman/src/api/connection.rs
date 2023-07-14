use crate::{context::RatmanContext, storage::addrs::StorageAddress};

use super::{
    client::{BaseClient, OnlineClient},
    io::Io,
};
use async_std::{fs::File, io::ReadExt, path::PathBuf, sync::Arc};
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

async fn load_users(context: &RatmanContext, path: PathBuf) -> Vec<Address> {
    debug!("Loading registered users from file {:?}", path);
    let mut f = match File::open(path).await {
        Ok(f) => f,
        Err(_) => return vec![],
    };

    let mut json = String::new();
    match f.read_to_string(&mut json).await {
        Ok(_) => {}
        Err(_) => return vec![],
    }

    match serde_json::from_str::<Vec<StorageAddress>>(&json) {
        Ok(vec) => {
            for StorageAddress { ref id, .. } in &vec {
                trace!("Loading addr {}", id);

                // TODO: implement key loading
                if let Err(e) = context.load_existing_address(*id, &[0]).await {
                    warn!("Failed to load address: {}", id);
                }
            }

            vec.into_iter().map(|l| l.id).collect()
        }
        Err(_) => vec![],
    }
}
