use crate::{
    error::{RpcError, RpcResult},
    io, Identity,
};
use async_std::{
    channel::{bounded, Receiver, Sender},
    sync::RwLock,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::BTreeMap,
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

pub(crate) type ArcBool = Arc<AtomicBool>;

/// A generic subscription type
///
/// Use this type in your component SDK to make it possible for users
/// to get updates for a particular stream of data.  Use
/// [`SubscriptionCmd`](crate::proto::SubscriptionCmd) to encode the
/// subscription creation handshake.  A subscription object is then
/// generic over the type returned by the subscription stream.
///
/// Following is an overview of the subscription message flow.
///
/// ```text
/// [ Your Service ]                     [ Remote Service ]
///     SubscriptionCmd::Register ----------->
///            <------------- SdkReply::Identity
///
///     ...
///
///            <------------- SubscriptionCmd::Push
///            <------------- SubscriptionCmd::Push
///
///     ...
///
///     SubscriptionCmd::Unregister --------->
///            <------------- SdkReply::Ok
/// ```
///
/// Because subscriptions need code running on both ends of the RPC
/// connection, there are two utility types you can use to map
/// subscriptions to and from the RPC connection.
///
/// * [`SubSwitch`](crate::SubSwitch) - maps from RPC to
/// SDK-Subscription (this type)
/// * [`SubManager`](crate::SubManager) - maps service side
/// subscriptions to the RPC stream
///
/// ## How to create a subscription
///
/// Because all of this is still very theoretical, let's walk through
/// a complete example.  It's recomended to wrap all of this code in
/// abstractions so that users of your SDK don't have to worry about
/// this, but the following example doesn't use any extra
/// abstractions.
///
/// ```rust
/// # use irpc_sdk::{*, error::RpcResult};
/// # async fn test() -> RpcResult<()> {
/// # let (addr, port) = default_socket_path();
///
/// // Create an RPC socket and register a service
/// let socket = RpcSocket::connect(addr, port).await?;
/// let mut service = Service::new("sub.test", 1, "Testing subscriptions");
/// service.register(&socket, Capabilities::basic_json()).await?;
///
/// //
/// # Ok(()) }
/// ```
pub struct Subscription<T>
where
    T: DeserializeOwned,
{
    _type: PhantomData<T>,
    rx: Receiver<Vec<u8>>,
    id: Identity,
    encoding: u8,
}

impl<T> Subscription<T>
where
    T: DeserializeOwned,
{
    pub(crate) fn new(rx: Receiver<Vec<u8>>, encoding: u8, id: Identity) -> Self {
        Self {
            _type: PhantomData,
            rx,
            id,
            encoding,
        }
    }

    /// Wait for the next subscription event
    pub async fn next(&self) -> RpcResult<T> {
        self.rx
            .recv()
            .await
            .map_err(|_| RpcError::SubscriptionEnded)
            .and_then(|vec| io::decode(self.encoding, &vec))
    }

    /// Get the subscription ID
    pub fn id(&self) -> Identity {
        self.id
    }
}

/// Map between an RPC connection and subscription objects
#[derive(Clone, Default)]
pub struct SubSwitch {
    enc: u8,
    map: Arc<RwLock<BTreeMap<Identity, Sender<Vec<u8>>>>>,
}

impl SubSwitch {
    /// Create a new map for RPC subscriptions
    pub fn new(enc: u8) -> Arc<Self> {
        Arc::new(Self {
            enc,
            ..Default::default()
        })
    }

    /// Create new subscription on the switch
    pub async fn create<T>(&self, encoding: u8, id: Identity) -> Subscription<T>
    where
        T: DeserializeOwned,
    {
        let (tx, rx) = bounded(8);
        self.map.write().await.insert(id.clone(), tx);
        Subscription::new(rx, encoding, id)
    }

    /// Send message push data to subscription handler
    ///
    /// When calling `forward` you may want to peel the concrete
    /// message type of your subscription object from the carrier that
    /// your service is notified with (depending on your service
    /// protocol).  This ensures that the subscription is typed
    /// correctly and can read the incoming stream.  A second
    /// serialisation is done in this function.
    pub async fn forward<T: Serialize>(&self, id: Identity, vec: T) -> RpcResult<()> {
        let map = self.map.read().await;
        let sender = map.get(&id).ok_or(RpcError::NoSuchSubscription)?;
        sender.send(io::encode(self.enc, &vec)?).await.unwrap();
        Ok(())
    }
}

/// Keep track of server side subs
///
/// uwu
#[derive(Default)]
pub struct SubManager {
    map: RwLock<BTreeMap<Identity, ArcBool>>,
}

impl SubManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Keep track of a new subscription via its ID
    pub async fn insert(&self, id: Identity) -> ArcBool {
        let b = ArcBool::new(true.into());
        self.map.write().await.insert(id, Arc::clone(&b));
        b
    }

    /// Disable the subscription
    pub async fn stop(&self, id: Identity) {
        if let Some(b) = self.map.write().await.remove(&id) {
            b.store(false, Ordering::Relaxed);
        }
    }
}
