use std::collections::{BTreeMap, BTreeSet};

use crate::{
    api::{
        client::{BaseClient, BaseClientMap, OnlineClientMap},
        io::Io,
    },
    context::RatmanContext,
    storage::{addrs::StorageAddress, client::StorageClient},
    util::Os,
};
use async_std::{
    fs::{File, OpenOptions},
    io::{ReadExt, WriteExt},
    sync::{Arc, Mutex, RwLock},
};
use libratman::{
    types::{Address, Id, Result},
    ClientError, RatmanError,
};

/// Handle known clients and active connections
///
/// An online client is derived from a `BaseClient`, which can be
/// persisted to disk and re-loaded at startup.
pub(crate) struct ConnectionManager {
    pub(crate) clients: Arc<Mutex<BaseClientMap>>,
    pub(crate) online: Arc<Mutex<OnlineClientMap>>,
    /// Map an address to a given client ID
    ///
    /// This is needed because each client can represent many
    /// different addresses, but these are opaque to the routing
    /// layer, meaning that we need to keep track of the relation
    ///
    /// Importantly this is a RwLock because it will be read _a lot_,
    /// but only written to occasionally.
    pub(crate) addr_map: Arc<RwLock<BTreeMap<Address, Id>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            clients: Default::default(),
            online: Default::default(),
            addr_map: Default::default(),
        }
    }

    /// Load an existing set of users from the storage path
    pub async fn load_users(&self, context: &RatmanContext) {
        let path = Os::match_os().data_path().join("users.json");
        debug!("Loading registered users from file {:?}", path);

        let json = {
            let mut json = String::new();
            match File::open(path).await {
                Ok(mut f) => {
                    let _ = f.read_to_string(&mut json).await;
                }
                // If the file doesn't exist, we use the default and
                // simply update the file on the next write
                Err(_) => json = format!("[ ]"),
            };

            json
        };

        let mut addr_map = self.addr_map.write().await;

        // Create a new BaseClientMap and populate it from the
        // StorageClient data that we just pulled from the json file
        let mut client_map = BaseClientMap::new();
        match serde_json::from_str::<Vec<StorageClient>>(&json) {
            Ok(vec) => {
                for store_client @ StorageClient { id, ref addrs, .. } in &vec {
                    // let address_set = addrs.iter().map(|sa| sa.id).collect::<BTreeSet<_>>();
                    // trace!("Loading address set {:?}", address_set);

                    // TODO: implement key loading
                    for StorageAddress {
                        id: address,
                        key: _,
                    } in addrs
                    {
                        if let Err(e) = context.load_existing_address(*address, &[0]).await {
                            warn!("Failed to load address: {}", id);
                        }

                        // Also insert the address into the addr_map
                        addr_map.insert(*address, *id);
                    }

                    client_map.insert(*id, BaseClient::existing(store_client));
                }
            }
            // If the json is in any way invalid, we just reset the
            // list and start fresh.  Yes this is very bad, but the
            // idea that we store client state in a json is bad to
            // begin with so whatever
            Err(_) => {}
        };

        *self.clients.lock().await = client_map;
    }

    /// Call this function after new user registrations to ensure we
    /// remember them next time
    pub(crate) async fn sync_users(&self) -> Result<()> {
        let clients = self.clients.lock().await;
        let set = clients.iter().filter(|(_, client)| !client.anonymous).fold(
            BTreeSet::default(),
            |mut set, (id, client)| {
                set.insert(StorageClient::new(*id, client));
                set
            },
        );

        let json = serde_json::to_string(&set)?;
        drop(clients);

        let path = Os::match_os().data_path().join("users.json");
        let mut f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .read(true)
            .open(path)
            .await?;
        f.write_all(json.as_bytes()).await?;
        Ok(())
    }

    /// Register a new client and immediately mark it as "online",
    /// return a registration token
    pub async fn register(&self, initial_addr: Address, io: Io) -> Id {
        let id = Id::random();
        let mut clients = self.clients.lock().await;
        let mut online = self.online.lock().await;
        let mut addr_map = self.addr_map.write().await;

        let base_client = BaseClient::register(initial_addr);
        let token = base_client.token;
        clients.insert(id, base_client);
        online.insert(id, clients.get(&id).unwrap().connect(io));
        addr_map.insert(initial_addr, id);
        token
    }

    /// Mark a given address and token combination as "online"
    pub async fn set_online(&self, id: Id, token: Id, io: Io) -> Result<()> {
        let clients = self.clients.lock().await;

        match clients.get(&id) {
            // If the token matches, we set the client to online
            Some(base_client) if base_client.token.compare_constant_time(&token) => {
                let mut online = self.online.lock().await;
                online.insert(id, base_client.connect(io));
                Ok(())
            }
            Some(_) => Err(ClientError::InvalidAuth.into()),
            None => Err(ClientError::NoAddress.into()),
        }
    }

    pub async fn get_client_for_address(&self, addr: &Address) -> Option<Id> {
        // This _should_ never fail because the router wouldn't have
        // picked this message up to be relayed locally if it didn't
        // already know the address was registered on this node.
        self.addr_map.read().await.get(addr).map(|id| *id)
    }

    /// Check a provided client token against its stored record
    pub async fn check_token(&self, client_id: &Id, token: &Id) -> bool {
        match self.clients.lock().await.get(client_id) {
            // If the record exists and the token matches
            Some(client) if client.token.compare_constant_time(token) => true,
            // Otherwise, the connection fails the sniff test
            _ => false,
        }
    }
}
