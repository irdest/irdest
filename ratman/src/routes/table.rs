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
    api::types::PeerEntry,
    frame::carrier::AnnounceFrameV1,
    tokio::{
        sync::{mpsc::channel, RwLock},
        task::{spawn_blocking, spawn_local},
        time::sleep,
    },
    types::{Address, Ident32},
    NonfatalError, RatmanError, Result,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, VecDeque},
    iter::FromIterator,
    sync::Arc,
    time::Duration,
};

/// Main Ratman routing table
///
/// It keeps track of available addresses and their types (i.e. remote
/// or local, and an address key or a namespace key).  New addresses
/// can be polled via the `new` announce channel.
pub(crate) struct RouteTable {
    meta_db: Arc<MetadataDb>,
    activity_tasks: Arc<RwLock<BTreeSet<Address>>>,
    // routes: Arc<Mutex<BTreeMap<Address, RouteType>>>,
    #[allow(unused)]
    new: IoPair<Address>,
    #[allow(unused)]
    #[cfg(feature = "dashboard")]
    metrics: metrics::RouteTableMetrics,
}

impl RouteTable {
    pub(crate) fn new(meta_db: Arc<MetadataDb>) -> Arc<Self> {
        let this = Arc::new(Self {
            meta_db,
            activity_tasks: Default::default(),
            // routes: Default::default(),
            new: channel(1),
            #[cfg(feature = "dashboard")]
            metrics: metrics::RouteTableMetrics::default(),
        });

        // When we have just come online we need to start a route activity
        // checker for every peer.  Peers that were stored as "Active" in the
        // database because they were the last time this instance ran, but have
        // since disappeared will appear online after startup, until the task
        // notices that they're not and marks them as such
        this.meta_db
            .routes
            .iter()
            .into_iter()
            .filter_map(|(_, entry)| match entry.route {
                Some(_) => Some(entry.peer),
                None => None,
            })
            .for_each(|peer| {
                this.clone().start_activity_task(peer);
            });

        this
    }
}

/////////////////////////////////// SNIP ///////////////////////////////////

/// A netmod endpoint ID and an endpoint target ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) struct EpNeighbourPair(pub(crate) usize, pub(crate) Ident32);

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
        match self.meta_db.routes.get(&peer_addr.to_string()).await? {
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
                if let Some(ref mut route) = route.as_mut() {
                    route.last_seen = Utc::now();
                    route.state = RouteState::Active;
                    route.data = announce_f.route;
                }

                new_route = RouteData {
                    peer,
                    link_id,
                    route_id,
                    route,
                };

                trace!(
                    "Update existing route to {} with new link information {new_route:?}",
                    peer.pretty_string()
                );
            }
            None => {
                info!("Discovered new address: {}", peer_addr.pretty_string());
                new_route = RouteData {
                    peer: peer_addr,
                    link_id: VecDeque::from_iter(vec![ep_neighbour].into_iter()),
                    route_id: Ident32::random(),
                    route: Some(RouteEntry {
                        data: announce_f.route,
                        state: RouteState::Active,
                        first_seen: Utc::now(),
                        last_seen: Utc::now(),
                    }),
                };
            }
        }

        // Then update the caches and on-disk table
        self.meta_db
            .routes
            .insert(peer_addr.to_string(), &new_route)
            .await?;

        // If a route is declared active we want to keep checking whether it is
        // still active.  Inactive addresses don't need this since any new
        // announcement will set their state to active, which will then spawn
        // this task anyway :)
        if let Some(route) = new_route.route {
            if route.state == RouteState::Active {
                if self.activity_tasks.read().await.get(&peer_addr).is_none() {
                    Arc::clone(self).start_activity_task(peer_addr);
                }
            }
        }

        // #[cfg(feature = "dashboard")]
        // self.metrics
        //     .routes_count
        //     .get_or_create(&metrics::RouteLabels { kind: route })
        //     .inc();

        Ok(())
    }

    fn start_activity_task(self: Arc<Self>, peer_addr: Address) {
        spawn_local(async move {
            self.activity_tasks.write().await.insert(peer_addr);
            let announce_timeout = 30;
            let sleep_time = 10;
            loop {
                // Check every 30 seconds whether the last announcement
                // is older than 1 minute.  If so, we declare the route
                // DOWN and end this task
                let check = Utc::now();
                sleep(Duration::from_secs(sleep_time)).await;
                match self.meta_db.routes.get(&peer_addr.to_string()).await {
                    Ok(Some(mut entry)) => {
                        //
                        // If the route is None it is a local address and we don't care
                        if let Some(route) = entry.route.as_mut() {
                            // todo: make this timeout configurable
                            if (check - route.last_seen).num_seconds() > announce_timeout {
                                route.state = RouteState::Idle;
                                info!("No announcement in >{announce_timeout} seconds: marking address {peer_addr} as IDLE");

                                if let Err(e) = self
                                    .meta_db
                                    .routes
                                    .insert(peer_addr.to_string(), &entry)
                                    .await
                                {
                                    error!("failed to update activity status for peer: {e}, abort acitivy check task");
                                }

                                break;
                            }
                        }
                    }
                    _ => {
                        warn!(
                            "Route entry {} has disappeared, aborting activity check task",
                            peer_addr
                        );
                        break;
                    }
                }
            }

            // Remove this task from the active set
            self.activity_tasks.write().await.remove(&peer_addr);
        });
    }

    pub(crate) async fn list_remote(self: &Arc<Self>) -> Result<Vec<PeerEntry>> {
        let this = Arc::clone(&self);
        spawn_blocking(move || {
            Ok(this
                .meta_db
                .routes
                // todo: this function loads all routes into memory, which may
                // fail on very large nodes.  But that's a future problem
                .iter()
                .into_iter()
                .filter(|(_, entry)| entry.route.is_some())
                .map(|(_, entry)| PeerEntry {
                    addr: entry.peer,
                    first_connection: entry.route.unwrap().first_seen,
                    last_connection: entry.route.unwrap().last_seen,
                    active: entry.route.unwrap().state == RouteState::Active,
                })
                .collect())
        })
        .await?
    }

    pub(crate) async fn register_local_route(&self, local: Address) -> Result<()> {
        let local_addr = RouteData::local(local);
        self.meta_db
            .routes
            .insert(local.to_string(), &local_addr)
            .await?;
        Ok(())
    }

    pub(crate) async fn scrub_local(&self, local: Address) -> Result<()> {
        self.meta_db.routes.remove(local.to_string()).await?;
        Ok(())
    }

    pub(crate) async fn is_local(&self, maybe_local: Address) -> Result<bool> {
        Ok(self
            .meta_db
            .routes
            .get(&maybe_local.to_string())
            .await?
            .ok_or(RatmanError::Nonfatal(NonfatalError::UnknownAddress(
                maybe_local,
            )))
            // Local addresses don't have route data associated
            .is_ok_and(|rd| rd.route.is_none()))
    }

    /// Get the endpoint and target ID for a peer's address
    pub(crate) async fn resolve(&self, peer_addr: Address) -> Option<EpNeighbourPair> {
        self.meta_db
            .routes
            .get(&peer_addr.to_string())
            .await
            .ok()
            .flatten()
            .and_then(|route_data| route_data.link_id.get(0).copied())
    }

    /// Check if an ID is reachable via currently known routes
    ///
    /// - `Some(State)` indicates a remote address with a particular connection
    /// state
    ///
    /// - `None` indicates a local address, since the state is always
    /// "available".
    pub(crate) async fn reachable(&self, peer_addr: Address) -> Option<RouteState> {
        self.meta_db
            .routes
            .get(&peer_addr.to_string())
            .await
            .ok()
            .flatten()
            .and_then(|route_data| route_data.route.map(|r| r.state))
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
        pub kind: String,
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
