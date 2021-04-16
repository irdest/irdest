use crate::{
    error::RpcResult,
    io::{self, Message},
    proto::{Registry, SdkCommand, SdkReply},
    Capabilities, RpcSocket, ENCODING_JSON,
};
use identity::Identity;
use serde::Serialize;
use std::{collections::BTreeMap, sync::Arc};

/// An async service representation on the irpc system
///
/// Use this struct to handle RPC connections to the network, and to
/// update any data you want your service to broadcast to other
/// participants on the irpc bus.
///
/// ## Example registration
///
/// ```rust
/// use irpc_sdk::{RpcSocket, Service, Capabilities};
/// # async fn test() -> irpc_sdk::error::RpcResult<()> {
/// let socket = RpcSocket::connect("localhost", 10222).await?;
/// let mut service = Service::new("com.example.test", 1, "A test service");
/// service.register(&socket, Capabilities::basic_json()).await?;
/// # Ok(())
/// # }
/// ```
pub struct Service {
    pub name: String,
    pub version: u16,
    pub description: String,
    caps: Option<Capabilities>,
    hash_id: Option<Identity>,
    socket: Option<Arc<RpcSocket>>,
}

impl Service {
    /// Create a new service representation
    ///
    /// By itself this representation doesn't do anything.  You need
    /// to connect it to an [`RpcSocket`](crate::RpcSocket) via the
    /// [`register()`](Self::register) function first.
    pub fn new<S: Into<String>>(name: S, version: u16, description: S) -> Self {
        Self {
            name: name.into(),
            version,
            description: description.into(),
            caps: None,
            hash_id: None,
            socket: None,
        }
    }

    /// Register this service with the RPC broker
    pub async fn register(
        &mut self,
        socket: &Arc<RpcSocket>,
        caps: Capabilities,
    ) -> RpcResult<Identity> {
        self.socket = Some(Arc::clone(&socket));
        self.caps = Some(caps.clone());

        // Construct a registry message
        let reg = Registry {
            name: self.name.clone(),
            version: self.version,
            description: self.description.clone(),
            caps,
        };

        let msg = self.to_broker(self.encoding(), &reg)?;

        let id = self
            .socket
            .as_ref()
            .unwrap()
            .send(msg, |reply| SdkReply::parse_identity(ENCODING_JSON, &reply))
            .await?;

        self.hash_id = Some(id);
        Ok(id)
    }

    /// Signal to the broker that this service is shutting down
    ///
    /// Calling this function is not _technically_ required, but good
    /// practise to let the broker know that it can close the
    /// connection, and indicate to other services that messages to
    /// this address will no longer be accepted.
    pub async fn shutdown(&self) -> RpcResult<()> {
        let shutdown = SdkCommand::Shutdown {
            name: self.name.clone(),
            hash_id: self.hash_id.unwrap(),
        };
        let msg = self.to_broker(self.encoding(), &shutdown)?;
        Ok(self
            .socket
            .as_ref()
            .unwrap()
            .send(msg, |reply| SdkReply::parse_ok(self.encoding(), &reply))
            .await?)
    }

    /// Get a reference to the id assigned to this service
    pub fn id(&self) -> Option<Identity> {
        self.hash_id
    }

    /// Get the RpcSocket for this service
    ///
    /// This function is called by other component SDKs to get access
    /// to the underlying RPC socket.
    pub fn socket(&self) -> Arc<RpcSocket> {
        Arc::clone(
            self.socket
                .as_ref()
                .expect("Can not get socket; service not registered!"),
        )
    }

    /// Get access to the supported service encoding
    pub fn encoding(&self) -> u8 {
        self.caps.as_ref().unwrap().encoding
    }

    /// Small utility function to send a message to the broker from
    /// the current service name
    fn to_broker<S: Serialize>(&self, enc: u8, data: &S) -> RpcResult<Message> {
        Ok(Message::to_addr(
            crate::DEFAULT_BROKER_ADDRESS,
            self.name.as_str(),
            io::encode(enc, data)?,
        ))
    }
}

/// A structure to express service runtime dependencies
#[derive(Default)]
pub struct Dependencies {
    inner: BTreeMap<String, u16>,
}

impl Dependencies {
    /// Create a new, empty depedency set
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a new service to the set of dependencies
    ///
    /// If a service is added twice, the previous version requirement
    /// is overwritten.  All service requirements are checked by the
    /// broker during registration.
    pub fn add<S: Into<String>>(mut self, name: S, version: u16) -> Self {
        self.inner.insert(name.into(), version);
        self
    }
}
