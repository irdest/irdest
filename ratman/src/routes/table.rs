// SPDX-FileCopyrightText: 2019-2024 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 embr <hi@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{
    storage::{
        route::{RouteData, RouteEntry, RouteState},
        MetadataDb,
    },
    util::IoPair,
};
use chrono::Utc;
use libratman::{
    frame::carrier::AnnounceFrameV1,
    tokio::sync::mpsc::channel,
    types::{Address, Ident32, Neighbour},
    NetmodError, RatmanError, Result,
};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, iter::FromIterator, sync::Arc};

/// Main Ratman routing table
///
/// It keeps track of available addresses and their types (i.e. remote
/// or local, and an address key or a namespace key).  New addresses
/// can be polled via the `new` announce channel.
pub(crate) struct RouteTable {
    meta_db: Arc<MetadataDb>,
    // routes: Arc<Mutex<BTreeMap<Address, RouteType>>>,
    new: IoPair<Address>,
    #[cfg(feature = "dashboard")]
    metrics: metrics::RouteTableMetrics,
}

impl RouteTable {
    pub(crate) fn new(meta_db: Arc<MetadataDb>) -> Arc<Self> {
        Arc::new(Self {
            meta_db,
            // routes: Default::default(),
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

pub(crate) async fn exec_route_table(_table: &Arc<RouteTable>) {
    // Do route things ??
}

/////////////////////////////////// SNIP ///////////////////////////////////

/// A netmod endpoint ID and an endpoint target ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) struct EpNeighbourPair(pub(crate) usize, pub(crate) Ident32);

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
    pub(crate) async fn update(
        self: &Arc<Self>,
        ep_neighbour: EpNeighbourPair,
        peer_addr: Address,
        announce_f: AnnounceFrameV1,
    ) -> Result<()> {
        // let mut tbl = self.routes.lock().await;
        // let route = RouteType::Remote(EpNeighbourPair(ifid, t));

        let new_route;
        match self.meta_db.routes.get(&peer_addr.to_string())? {
            Some(RouteData {
                peer,
                mut link_id,
                route_id,
                mut route,
            }) => {
                // If current ifid is contained in route set AND not currently
                // the highest priority anyway (AND set not empty)
                if link_id.contains(&ep_neighbour) && link_id.get(0) != Some(&ep_neighbour) {
                    // filter current ifid from set and write back
                    link_id = link_id.into_iter().filter(|x| x != &ep_neighbour).collect();
                    // then push current ifid to front
                    link_id.push_front(ep_neighbour);
                }

                // Update other bits of metadata
                route.last_seen = Utc::now();
                route.state = RouteState::Active;
                route.data = announce_f.route;

                new_route = RouteData {
                    peer,
                    link_id,
                    route_id,
                    route,
                };
            }
            None => {
                new_route = RouteData {
                    peer: peer_addr,
                    link_id: VecDeque::from_iter(vec![ep_neighbour].into_iter()),
                    route_id: Ident32::random(),
                    route: RouteEntry {
                        data: announce_f.route,
                        state: RouteState::Active,
                        first_seen: Utc::now(),
                        last_seen: Utc::now(),
                    },
                };
            }
        }

        if self.meta_db.routes.get(&peer_addr.to_string())?.is_some() {
            // spawn_local(async move { s.new.0.send(id).await });
        }

        // Then update the caches and on-disk table
        self.meta_db
            .routes
            .insert(peer_addr.to_string(), &new_route)?;

        // #[cfg(feature = "dashboard")]
        // self.metrics
        //     .routes_count
        //     .get_or_create(&metrics::RouteLabels { kind: route })
        //     .inc();

        Ok(())
    }

    /// Poll the set of newly discovered users
    pub(crate) async fn discover(&mut self) -> Address {
        self.new.1.recv().await.unwrap()
    }

    /// Check if a user is locally known
    pub(crate) async fn local(&self, id: Address) -> Result<()> {
        self.meta_db
            .addrs
            .get(&id.to_string())?
            .ok_or(RatmanError::NoSuchAddress(id))
            .map(|_| ())
    }

    /// Get all users in the routing table
    pub(crate) async fn all(&self) -> Vec<(Address, RouteEntry)> {
        self.meta_db
            .routes
            .iter()
            .map(|(id, data)| (Address::from_string(&id), data.route))
            .collect()
    }

    /// Get the endpoint and target ID for a peer's address
    pub(crate) async fn resolve(&self, peer_addr: Address) -> Option<EpNeighbourPair> {
        self.meta_db
            .routes
            .get(&peer_addr.to_string())
            .ok()
            .flatten()
            .and_then(|route_data| route_data.link_id.get(0).copied())
    }

    /// Check if an ID is reachable via currently known routes
    pub(crate) async fn reachable(&self, peer_addr: Address) -> Option<RouteState> {
        self.meta_db
            .routes
            .get(&peer_addr.to_string())
            .ok()
            .flatten()
            .map(|route_data| route_data.route.state)
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
