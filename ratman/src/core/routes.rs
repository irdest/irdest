// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 embr <hi@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Routing table module

use crate::util::IoPair;
use libratman::{
    tokio::{
        sync::{mpsc::channel, Mutex},
        task,
    },
    types::{Address, Neighbour},
    RatmanError, Result,
};
use std::{collections::BTreeMap, sync::Arc};

/// Main Ratman routing table
///
/// It keeps track of available addresses and their types (i.e. remote
/// or local, and an address key or a namespace key).  New addresses
/// can be polled via the `new` announce channel.
pub(crate) struct RouteTable {
    routes: Arc<Mutex<BTreeMap<Address, RouteType>>>,
    new: IoPair<Address>,
    #[cfg(feature = "dashboard")]
    metrics: metrics::RouteTableMetrics,
}

impl RouteTable {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            routes: Default::default(),
            new: channel(1),
            #[cfg(feature = "dashboard")]
            metrics: metrics::RouteTableMetrics::default(),
        })
    }
}

/// Query the routing table on whether it knows a particular address
pub(crate) async fn query_known(table: &Arc<RouteTable>, addr: Address, local: bool) -> Result<()> {
    if local {
        table.local(addr).await
    } else {
        table
            .resolve(addr)
            .await
            .map_or(Err(RatmanError::NoSuchAddress(addr)), |_| Ok(()))
    }
}

pub(crate) async fn exec_route_table(tabel: &Arc<RouteTable>) {
    // Do route things ??
}

/////////////////////////////////// SNIP ///////////////////////////////////

/// A netmod endpoint ID and an endpoint target ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct EpNeighbourPair(pub(crate) u8, pub(crate) Neighbour);

/// Describes the reachability of a route
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum RouteType {
    Remote(EpNeighbourPair),
    Local,
}

/// An ephemeral routing table
///
/// It only captures the current state of best routes and has no
/// persistence relationships.  It can update entries for topology
/// changes, but these are not carried between sessions.

impl RouteTable {
    /// Register metrics with a Prometheus registry.
    #[cfg(feature = "dashboard")]
    pub(crate) fn register_metrics(&self, registry: &mut prometheus_client::registry::Registry) {
        self.metrics.register(registry);
    }

    /// Update or add an IDs entry in the routing table
    ///
    /// If the Id was not previously known to the router, it is queued
    /// to the `new` set which can be polled by calling `discovered().await`.
    pub(crate) async fn update(self: &Arc<Self>, if_: u8, t: Neighbour, id: Address) {
        let mut tbl = self.routes.lock().await;
        let route = RouteType::Remote(EpNeighbourPair(if_, t));

        // Only "announce" a new user if it was not known before
        if tbl.insert(id, route).is_none() {
            info!("Discovered new address {}", id);
            debug!("New routing table state is: {:#?}", tbl);

            #[cfg(feature = "dashboard")]
            self.metrics
                .routes_count
                .get_or_create(&metrics::RouteLabels { kind: route })
                .inc();
            let s = Arc::clone(&self);
            task::spawn(async move { s.new.0.send(id).await });
        }
    }

    /// Poll the set of newly discovered users
    pub(crate) async fn discover(&mut self) -> Address {
        self.new.1.recv().await.unwrap()
    }

    /// Track a local ID in the routes table
    pub(crate) async fn add_local(&self, id: Address) -> Result<()> {
        match self.routes.lock().await.insert(id, RouteType::Local) {
            Some(_) => Err(RatmanError::DuplicateAddress(id)),
            None => {
                #[cfg(feature = "dashboard")]
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
    pub(crate) async fn local(&self, id: Address) -> Result<()> {
        match self.reachable(id).await {
            Some(RouteType::Local) => Ok(()),
            _ => Err(RatmanError::NoSuchAddress(id)),
        }
    }

    /// Delete an entry from the routing table
    pub(crate) async fn delete(&self, id: Address) -> Result<()> {
        match self.routes.lock().await.remove(&id) {
            Some(_kind) => {
                #[cfg(feature = "dashboard")]
                self.metrics
                    .routes_count
                    .get_or_create(&metrics::RouteLabels { kind: _kind })
                    .dec();
                Ok(())
            }
            None => Err(RatmanError::NoSuchAddress(id)),
        }
    }

    /// Get all users in the routing table
    pub(crate) async fn all(&self) -> Vec<(Address, RouteType)> {
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
    pub(crate) async fn resolve(&self, id: Address) -> Option<EpNeighbourPair> {
        match self.routes.lock().await.get(&id).cloned()? {
            RouteType::Remote(ep) => Some(ep),
            RouteType::Local => None,
        }
    }

    /// Check if an ID is reachable via currently known routes
    pub(crate) async fn reachable(&self, id: Address) -> Option<RouteType> {
        self.routes.lock().await.get(&id).cloned()
    }
}

#[cfg(feature = "dashboard")]
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
                "ratman_routes_current",
                "Number of routes currently in the table",
                Box::new(self.routes_count.clone()),
            );
        }
    }
}
