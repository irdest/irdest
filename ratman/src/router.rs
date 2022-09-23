use crate::{
    clock::{ClockCtrl, Tasks},
    core::Core,
    crypto::Keystore,
    storage::addrs::LocalAddress,
    Address, Message, MsgId, Protocol, Recipient,
};
use async_std::sync::Arc;
use netmod::Endpoint;
use types::{Result, TimePair};

/// Primary async ratman router handle
///
/// Make sure you initialise endpoints before calling [`run`], as the
/// set of endpoints gets locked and can't be changed during runtime.
///
/// [`run`]: struct.Router.html#method.run
#[derive(Clone)]
pub struct Router {
    inner: Arc<Core>,
    proto: Arc<Protocol>,
    keys: Arc<Keystore>,
}

impl Router {
    /// Create a new and empty message router
    ///
    /// It's currently not possible to restore a router from stored
    /// state, which means that all routing tables are lost when the
    /// router is stopped.
    pub fn new() -> Self {
        let proto = Protocol::new();
        let inner = Arc::new(Core::init());
        let keys = Arc::new(Keystore::new());
        Self { inner, proto, keys }
    }

    /// Register metrics with a Prometheus registry.
    #[cfg(feature = "dashboard")]
    pub fn register_metrics(&self, registry: &mut prometheus_client::registry::Registry) {
        self.inner.register_metrics(registry);
        self.proto.register_metrics(registry);
    }

    /// Add a new endpoint to this router
    ///
    /// An endpoint is defined by the [`Endpoint`] trait from the
    /// `ratman-netmod` crate.  Once added, an endpoint can't be
    /// removed while in active operation: the router will have to be
    /// recreated without the endpoint you wish to remove.
    ///
    /// [`Endpoint`]: https://docs.rs/ratman-netmod/0.1.0/ratman_netmod/trait.Endpoint.html
    pub async fn add_endpoint(&self, ep: Arc<impl Endpoint + 'static + Send + Sync>) -> usize {
        self.inner.add_ep(ep).await
    }

    /// **Unstable fn:** get an endpoint from the driver set by ID
    #[doc(hidden)]
    pub async fn get_endpoint(&self, id: usize) -> Arc<dyn Endpoint + 'static + Send + Sync> {
        self.inner.get_ep(id).await
    }

    /// Remove an endpoint from the router by ID
    ///
    /// This function is primarily meant for testing purposes, and
    /// shouldn't be used in heavy operation.  The required ID is
    /// returned by `add_endpoint`.
    pub async fn del_endpoint(&self, id: usize) {
        self.inner.rm_ep(id).await;
    }

    /// Add an identity to the local set
    ///
    /// Ratman will listen for messages to local identities and offer
    /// them up for polling via the Router API.
    pub async fn add_user(&self) -> Result<Address> {
        let id = self.keys.create_address().await;
        self.inner.add_local(id).await.map(|_| id)
    }

    /// Load an existing address and key
    pub async fn load_address(&self, id: Address, key_data: &[u8]) -> Result<()> {
        self.keys.add_address(id, key_data).await.unwrap();
        self.inner.add_local(id).await
    }

    // This function used to be needed because we weren't creating an
    // ID and key internally.
    #[deprecated]
    pub async fn add_existing_user(&self, id: Address) -> Result<()> {
        self.inner.add_local(id).await
    }

    /// Get locally registered addresses
    pub async fn local_addrs(&self) -> Vec<LocalAddress> {
        self.keys.get_all().await
    }

    /// Remove a local address, discarding imcomplete messages
    ///
    /// Ratman will by default remove all cached frames from the
    /// collector.  Optionally these frames can be moved into the
    /// journal with low priority instead.
    pub async fn del_user(&self, id: Address, _keep: bool) -> Result<()> {
        self.inner.rm_local(id).await
    }

    /// Set an address as online and broadcast announcements
    ///
    /// This function will return an error if the address is already
    /// marked as online, or if no such address is known to the router
    pub async fn online(&self, id: Address) -> Result<()> {
        self.inner.known(id, true).await?;
        Arc::clone(&self.proto)
            .online(id, Arc::clone(&self.inner))
            .await
    }

    /// Set an address as offline and stop broadcasts
    pub async fn offline(&self, id: Address) -> Result<()> {
        self.inner.known(id, true).await?;
        self.proto.offline(id).await
    }

    /// Check the local routing table for a user ID
    pub async fn known(&self, id: Address) -> Result<()> {
        self.inner.known(id, false).await
    }

    /// Check for newly discovered users on the network
    pub async fn discover(&self) -> Address {
        self.inner.discover().await
    }

    /// Register a manual clock controller object for internal tasks
    pub fn clock(&self, _cc: ClockCtrl<Tasks>) -> Result<()> {
        unimplemented!()
    }

    /// Send a flood message into the network
    pub async fn flood(
        &self,
        sender: Address,
        scope: Address,
        payload: Vec<u8>,
        sign: Vec<u8>,
    ) -> Result<MsgId> {
        debug!("Sending flood to namespace {}", scope);
        let id = MsgId::random();
        self.inner
            .send(Message {
                id,
                sender,
                recipient: Recipient::Flood(scope),
                payload,
                timesig: TimePair::sending(),
                sign,
            })
            .await
            .map(|_| id)
    }

    /// Dispatch a message into a network
    ///
    /// This operation completes asynchronously, and will yield a
    /// result with information about any error that occured while
    /// sending.
    ///
    /// If you result is an `Error::DispatchFaled`, that just means
    /// that at least one of the packets your Message was sliced into
    /// didn't send properly.  As long as you're not changing the data
    /// layout of your payload, or the `MsgId`, it's safe to simply
    /// retry: the receiving collector/ journals on the way will still
    /// be able to associate the frames, and drop the ones that were
    /// already dispatched, essentially only filling in the missing
    /// gaps.
    pub async fn send(&self, msg: Message) -> Result<()> {
        self.inner.send(msg).await
    }

    /// Get the next available message from the router
    ///
    /// **Note**: This function can't ever really fail, because it
    /// only reads from a set of completed Messages that have been
    /// parsed and handled.  When an error occurs on an incoming
    /// Message, the errors are logged in the diagnostics module, and
    /// can be read from there asynchronously.
    pub async fn next(&self) -> Message {
        self.inner.next().await
    }

    /// Return a list of all known addresses
    pub async fn known_addresses(&self) -> Vec<(Address, bool)> {
        self.inner.all_addrs().await
    }
}

/// A very simple API level test to make sure that payloads remain the same
#[async_std::test]
async fn matching_payloads() {
    use crate::TimePair;
    use netmod_mem::MemMod;
    let (m1, m2) = MemMod::make_pair();

    let r1 = Router::new();
    let r2 = Router::new();

    r1.add_endpoint(m1).await;
    r2.add_endpoint(m2).await;

    let u1 = r1.add_user().await.unwrap();
    let u2 = r2.add_user().await.unwrap();

    r1.online(u1).await.unwrap();
    r2.online(u2).await.unwrap();

    let msg = Message {
        id: Address::random(),
        sender: u1,
        recipient: Recipient::Standard(vec![u2]),
        payload: vec![1, 3, 1, 2],
        timesig: TimePair::sending(),
        sign: vec!['a' as u8, 'c' as u8, 'a' as u8, 'b' as u8],
    };

    // Wait for the announcement to sync
    let _ = r1.discover().await;

    // Then send a message
    r1.send(msg.clone()).await.unwrap();

    let msg2 = r2.next().await;

    assert_eq!(msg2.remove_recv_time(), msg);
}
