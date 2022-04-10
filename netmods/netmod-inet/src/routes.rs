use crate::peer::Peer;
use async_std::sync::{Arc, RwLock};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU16, Ordering},
};

pub(crate) type Target = u16;

#[derive(Default)]
pub(crate) struct Routes {
    latest: AtomicU16,
    inner: RwLock<BTreeMap<Target, Arc<Peer>>>,
}

impl Routes {
    /// Create a new empty routing table
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Get the next valid target number for a peer
    pub(crate) fn next_target(self: &Arc<Self>) -> u16 {
        self.latest.fetch_add(1, Ordering::AcqRel)
    }

    /// Add a new peer routing routing map
    ///
    /// This is done by the server when adding a new peer, or by the
    /// session manager when creating a connection
    pub(crate) async fn add_peer(self: &Arc<Self>, target: Target, peer: Arc<Peer>) {
        let mut inner = self.inner.write().await;
        inner.insert(target, peer);
    }

    /// Remove a peer from the routing map
    ///
    /// This should only be done by the peer itself, when it closes
    /// its stream
    pub(crate) async fn remove_peer(self: &Arc<Self>, target: Target) {
        let mut inner = self.inner.write().await;
        inner.remove(&target);
    }

    /// Return the peer associated with a particular target ID
    pub(crate) async fn get_peer_by_id(self: &Arc<Self>, target: Target) -> Option<Arc<Peer>> {
        let inner = self.inner.read().await;
        inner.get(&target).map(Clone::clone)
    }
}
