use async_std::{net::TcpStream, sync::RwLock};
use irpc_sdk::{
    error::{RpcError, RpcResult},
    Capabilities, Identity,
};
use std::{collections::BTreeMap, sync::Arc};

/// Represents a service to the broker
#[allow(unused)]
pub(crate) struct ServiceEntry {
    name: String,
    version: u16,
    description: String,
    caps: Capabilities,
    hash_id: Identity,
    io: TcpStream,
}

#[derive(Default)]
pub struct ServiceMap {
    inner: RwLock<BTreeMap<String, ServiceEntry>>,
}

impl ServiceMap {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Check if a particular service is already known to the broker
    pub(crate) async fn caps(&self, name: &String) -> RpcResult<Capabilities> {
        self.inner
            .read()
            .await
            .get(name)
            .as_ref()
            .map(|s| s.caps.clone())
            .ok_or_else(|| RpcError::NoSuchService(name.clone()))
    }

    /// Get the hash ID for a particular service name
    pub(crate) async fn hash_id(&self, name: &String) -> RpcResult<Identity> {
        self.inner
            .read()
            .await
            .get(name)
            .map(|serv| serv.hash_id.clone())
            .ok_or_else(|| RpcError::NoSuchService(name.clone()))
    }

    /// Register a new service
    pub(crate) async fn register(
        &self,
        name: String,
        version: u16,
        description: String,
        caps: Capabilities,
        io: &TcpStream,
    ) -> Identity {
        let hash_id = Identity::random();
        self.inner.write().await.insert(
            name.clone(),
            ServiceEntry {
                name,
                version,
                description,
                caps,
                hash_id: hash_id.clone(),
                io: io.clone(),
            },
        );

        hash_id
    }

    pub(crate) async fn match_id(&self, name: &String, id: &Identity) -> RpcResult<()> {
        self.inner
            .read()
            .await
            .get(name)
            .ok_or_else(|| RpcError::NoSuchService(name.clone()))
            .and_then(|entry| {
                if &entry.hash_id == id {
                    Ok(())
                } else {
                    Err(RpcError::NotAuthorised)
                }
            })
    }

    pub(crate) async fn shutdown(&self, name: &String) {
        let entry = self.inner.write().await.remove(name).unwrap();
        entry.io.shutdown(std::net::Shutdown::Both).unwrap();
    }
}
