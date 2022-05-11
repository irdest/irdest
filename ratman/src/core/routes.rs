// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 embr <hi@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Routing table module

use crate::{Error, IoPair, Result};
use async_std::{
    channel::bounded,
    sync::{Arc, Mutex},
    task,
};
use netmod::Target;
use std::collections::BTreeMap;
use types::Identity;

/// A netmod endpoint ID and an endpoint target ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct EpTargetPair(pub(crate) u8, pub(crate) Target);

/// Describes the reachability of a route
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum RouteType {
    Remote(EpTargetPair),
    Local,
}

/// An ephemeral routing table
///
/// It only captures the current state of best routes and has no
/// persistence relationships.  It can update entries for topology
/// changes, but these are not carried between sessions.
pub(crate) struct RouteTable {
    routes: Arc<Mutex<BTreeMap<Identity, RouteType>>>,
    new: IoPair<Identity>,
    #[cfg(feature = "webui")]
    metrics: metrics::RouteTableMetrics,
}

impl RouteTable {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            routes: Default::default(),
            new: bounded(1),
            #[cfg(feature = "webui")]
            metrics: metrics::RouteTableMetrics::default(),
        })
    }

    /// Register metrics with a Prometheus registry.
    #[cfg(feature = "webui")]
    pub(crate) fn register_metrics(&self, registry: &mut prometheus_client::registry::Registry) {
        self.metrics.register(registry);
    }

    /// Update or add an IDs entry in the routing table
    ///
    /// If the Id was not previously known to the router, it is queued
    /// to the `new` set which can be polled by calling `discovered().await`.
    pub(crate) async fn update(self: &Arc<Self>, if_: u8, t: Target, id: Identity) {
        let mut tbl = self.routes.lock().await;
        let route = RouteType::Remote(EpTargetPair(if_, t));

        // Only "announce" a new user if it was not known before
        if tbl.insert(id, route).is_none() {
            info!("Discovered new address {}", id);
            #[cfg(feature = "webui")]
            self.metrics
                .routes_count
                .get_or_create(&metrics::RouteLabels { kind: route })
                .inc();
            let s = Arc::clone(&self);
            task::spawn(async move { s.new.0.send(id).await });
        }
    }

    /// Poll the set of newly discovered users
    pub(crate) async fn discover(&self) -> Identity {
        self.new.1.recv().await.unwrap()
    }

    /// Track a local ID in the routes table
    pub(crate) async fn add_local(&self, id: Identity) -> Result<()> {
        match self.routes.lock().await.insert(id, RouteType::Local) {
            Some(_) => Err(Error::DuplicateAddress),
            None => {
                #[cfg(feature = "webui")]
                self.metrics
                    .routes_count
                    .get_or_create(&metrics::RouteLabels {
                        kind: RouteType::Local,
                    })
                    .inc();
                Ok(())
            }
        }
    }

    /// Check if a user is locally known
    pub(crate) async fn local(&self, id: Identity) -> Result<()> {
        match self.reachable(id).await {
            Some(RouteType::Local) => Ok(()),
            _ => Err(Error::NoAddress),
        }
    }

    /// Delete an entry from the routing table
    pub(crate) async fn delete(&self, id: Identity) -> Result<()> {
        match self.routes.lock().await.remove(&id) {
            Some(kind) => {
                #[cfg(feature = "webui")]
                self.metrics
                    .routes_count
                    .get_or_create(&metrics::RouteLabels { kind })
                    .dec();
                Ok(())
            }
            None => Err(Error::NoAddress),
        }
    }

    /// Get all users in the routing table
    pub(crate) async fn all(&self) -> Vec<(Identity, RouteType)> {
        self.routes
            .lock()
            .await
            .iter()
            .map(|(i, tt)| (*i, tt.clone()))
            .collect()
    }

    /// Get the endpoint and target ID for a user Identity
    ///
    /// **Note**: this function may panic if no entry was found, and
    /// returns `None` if the specified ID isn't remote.  To get more
    /// control over how the table is queried, use `reachable` instead
    pub(crate) async fn resolve(&self, id: Identity) -> Option<EpTargetPair> {
        match self.routes.lock().await.get(&id).cloned()? {
            RouteType::Remote(ep) => Some(ep),
            RouteType::Local => None,
        }
    }

    /// Check if an ID is reachable via currently known routes
    pub(crate) async fn reachable(&self, id: Identity) -> Option<RouteType> {
        self.routes.lock().await.get(&id).cloned()
    }
}

#[cfg(feature = "webui")]
mod metrics {
    //! Metric helpers.

    use prometheus_client::{
        encoding::text::Encode,
        metrics::{family::Family, gauge::Gauge},
        registry::Registry,
    };

    #[derive(Clone, Hash, PartialEq, Eq, Encode)]
    pub(super) struct RouteLabels {
        pub kind: super::RouteType,
    }

    impl Encode for super::RouteType {
        fn encode(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
            match self {
                Self::Local => write!(w, "local"),
                // TODO: Can we add more detail to this?
                Self::Remote(_) => write!(w, "remote"),
            }
        }
    }

    #[derive(Default)]
    pub(super) struct RouteTableMetrics {
        pub routes_count: Family<RouteLabels, Gauge>,
    }

    impl RouteTableMetrics {
        pub fn register(&self, registry: &mut Registry) {
            registry.register(
                "routes_count",
                "Number of routes currently in the table",
                Box::new(self.routes_count.clone()),
            );
        }
    }
}
