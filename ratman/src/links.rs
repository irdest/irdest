// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use libratman::{endpoint::EndpointExt, tokio::sync::RwLock};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

/// A dynamicly allocated, generic driver in memory
pub(crate) type GenericEndpoint = dyn EndpointExt + 'static + Send + Sync;

/// Wrap around endpoints that can be removed
///
/// This way, when remove an interface, the ID's of other interfaces
/// don't have have to be updated or mapped, because their place in the list doesn't change.
enum EpWrap {
    Used(String, Arc<GenericEndpoint>),
    #[allow(unused)]
    Void,
}
type EpVec = Vec<EpWrap>;

/// A map of available endpoint drivers
///
/// Currently the removing of drivers isn't supported, but it's
/// possible to have the same endpoint in the map multiple times, with
/// unique IDs.
#[derive(Default)]
pub(crate) struct LinksMap {
    curr: AtomicUsize,
    map: RwLock<EpVec>,
}

impl LinksMap {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Insert a new endpoint to the set of known endpoints
    pub(crate) async fn add(&self, name: String, ep: Arc<GenericEndpoint>) -> usize {
        let mut map = self.map.write().await;
        let curr = self.curr.fetch_add(1, Ordering::SeqCst);
        map.push(EpWrap::Used(name.clone(), ep));
        curr
    }

    /// Remove an endpoint from the list
    #[allow(unused)]
    pub(crate) async fn remove(&self, id: usize) {
        let mut map = self.map.write().await;
        std::mem::swap(&mut map[id], &mut EpWrap::Void);
    }

    /// Get access to an endpoint via an Arc wrapper
    pub(crate) async fn get(&self, id: usize) -> (String, Arc<GenericEndpoint>) {
        let map = self.map.read().await;
        match map[id] {
            EpWrap::Used(ref name, ref ep) => (name.clone(), Arc::clone(ep)),
            EpWrap::Void => panic!("Trying to use a removed endpoint!"),
        }
    }

    /// Search through the driver list and get the first one with a given name
    pub(crate) async fn get_by_name(&self, name: &str) -> Option<Arc<GenericEndpoint>> {
        let map = self.map.read().await;
        map.iter()
            .filter_map(|entry| match entry {
                EpWrap::Used(ep_name, ep) if ep_name == name => Some(Arc::clone(&ep)),
                _ => None,
            })
            .next()
    }

    /// Get access to all endpoints wrapped in Arc
    pub(crate) async fn get_all(&self) -> Vec<(String, Arc<GenericEndpoint>)> {
        let map = self.map.read().await;
        map.iter()
            .filter_map(|ep| match ep {
                EpWrap::Used(ref name, ref ep) => Some((name.clone(), Arc::clone(ep))),
                _ => None,
            })
            .collect()
    }

    /// Get all endpoints, except for the one provided via the ID
    pub(crate) async fn get_with_ids(&self) -> Vec<(String, Arc<GenericEndpoint>, usize)> {
        let map = self.map.read().await;
        map.iter()
            .enumerate()
            .filter_map(|(i, ep)| match ep {
                EpWrap::Used(ref name, ref ep) => Some((name.clone(), Arc::clone(ep), i)),
                _ => None,
            })
            .collect()
    }
}
