use crate::{
    error::{RpcError, RpcResult},
    io, Identity, ENCODING_JSON,
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
/// to get updates for a particular stream of data.  **There is no
/// universal subsciption command**, which means that your SDK
/// protocol needs to implement this.  The following example looks at
/// the `irdest-core` interface, provided by `irdest-sdk`.
///
/// Following is an overview of the subscription message flow.
///
/// ```text
/// [ Your Service ]                     [ Remote Service ]
///     Messages::Subscription(...) ----------->
///            <------------- Reply::Subscription(Identity)
///
///     ...
///
///            <------------- Reply::Messages(Message)
///            <------------- Reply::Messages(Message)
///
///     ...
///
///     Messages::StopSubscription ------------>
///            <------------- Reply::Messages(Ok)
/// ```
///
/// Because subscriptions need code running on both ends of the RPC
/// connection, there are two utility types you SHOULD use to map
/// subscriptions to and from the RPC connection.
///
/// * [`SubManager`](crate::SubManager) - run by the server, and maps
///   subscriptions to an RPC connection
/// * [`SubSwitch`](crate::SubSwitch) - run by the client, and maps
///   incoming RPC messages to the `Subscription` type to poll
///
/// TODO: create a simple code example that creates a subscription
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
    pub async fn forward<T: Serialize>(&self, id: Identity, msg: &T) -> RpcResult<()> {
        let map = self.map.read().await;
        let sender = map.get(&id).ok_or(RpcError::NoSuchSubscription)?;
        sender.send(io::encode(ENCODING_JSON, &msg)?).await.unwrap();
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
