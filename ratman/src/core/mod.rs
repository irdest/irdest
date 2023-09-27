// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 embr <hi@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Routing core components
//!
//! In previous designs (both code and docs) this was a single
//! component. This has proven to be a hard to maintain approach, so
//! instead the core has been split into several parts.

mod dispatch;
mod drivers;
mod journal;
mod routes;
mod switch;

pub(self) use dispatch::Dispatch;
pub(self) use drivers::DriverMap;
pub(self) use journal::Journal;
pub(self) use routes::{EpTargetPair, RouteTable, RouteType};
pub(self) use switch::Switch;

pub(crate) use drivers::GenericEndpoint;

use crate::dispatch::BlockCollector;
use async_std::sync::Arc;
use libratman::types::{Address, Frame, Message, RatmanError, Result};

/// The Ratman routing core interface
///
/// The core handles maintaining routing table state, sending message
/// streams, and re-assembling received frames into valid incoming
/// message streams.
///
/// Importantly, the core must be "booted", i.e. contains no state at
/// start.  All state handling must occur outside of the core.
/// Components that wish to have their state persisted must provide
/// efficient access to this in-memory state via the `Core` API
/// facade.
pub(crate) struct Core {
    collector: Arc<BlockCollector>,
    dispatch: Arc<Dispatch>,
    _journal: Arc<Journal>,
    routes: Arc<RouteTable>,
    switch: Arc<Switch>,
    drivers: Arc<DriverMap>,
}

impl Core {
    /// Initialises, but doesn't run the routing core
    pub(crate) fn init() -> Self {
        let drivers = DriverMap::new();
        let routes = RouteTable::new();
        let _journal = Journal::new();

        let (jtx, jrx) = async_std::channel::bounded(16);
        let collector = BlockCollector::new(jtx);
        let dispatch = Dispatch::new(
            Arc::clone(&routes),
            Arc::clone(&drivers),
            Arc::clone(&collector),
        );

        let switch = Switch::new(
            Arc::clone(&routes),
            Arc::clone(&_journal),
            Arc::clone(&dispatch),
            Arc::clone(&collector),
            Arc::clone(&drivers),
        );

        // Dispatch the runners
        Arc::clone(&switch).run();
        async_std::task::spawn(Arc::clone(&_journal).run(jrx));

        Self {
            dispatch,
            routes,
            collector,
            _journal,
            switch,
            drivers,
        }
    }

    /// Register metrics with a Prometheus registry.
    #[cfg(feature = "dashboard")]
    pub fn register_metrics(&self, registry: &mut prometheus_client::registry::Registry) {
        self.routes.register_metrics(registry);
        self.dispatch.register_metrics(registry);
        self.switch.register_metrics(registry);
    }

    /// Asynchronously send a Message
    pub(crate) async fn send(&self, msg: Message) -> Result<()> {
        self.dispatch.send_msg(msg).await
    }

    /// Send a frame directly, without message slicing
    ///
    /// Some components in Ratman, outside of the routing core, need
    /// access to direct frame intercepts, because protocol logic
    /// depends on unmodified frames.
    pub(crate) async fn raw_flood(&self, f: Frame) -> Result<()> {
        self.dispatch.flood(f).await
    }

    /// Poll for the incoming Message
    pub(crate) async fn next(&self) -> Message {
        self.collector.completed().await
    }

    /// Check if an Id is present in the routing table
    pub(crate) async fn known(&self, id: Address, local: bool) -> Result<()> {
        if local {
            self.routes.local(id).await
        } else {
            self.routes
                .resolve(id)
                .await
                .map_or(Err(RatmanError::NoSuchAddress(id)), |_| Ok(()))
        }
    }

    /// Returns users that were newly discovered in the network
    pub(crate) async fn discover(&self) -> Address {
        self.routes.discover().await
    }

    /// Insert a new endpoint
    pub(crate) async fn add_ep(&self, ep: Arc<GenericEndpoint>) -> usize {
        let id = self.drivers.add(ep).await;
        self.switch.add(id).await;
        id
    }

    /// Get an endpoint back from the driver set via it's ID
    pub(crate) async fn get_ep(&self, id: usize) -> Arc<GenericEndpoint> {
        self.drivers.get(id).await
    }

    /// Remove an endpoint
    pub(crate) async fn rm_ep(&self, id: usize) {
        self.drivers.remove(id).await;
    }

    /// Add a local address to the routing table
    pub(crate) async fn add_local_address(&self, id: Address) -> Result<()> {
        self.routes.add_local(id).await
    }

    /// Remove a local address from the routing table
    pub(crate) async fn remove_local_address(&self, id: Address) -> Result<()> {
        self.routes.delete(id).await
    }

    // FIXME: this is basically just moving the hard-coded value somewhere else
    pub(crate) fn get_route_mtu(&self, _recipient: Option<Address>) -> u16 {
        1300
    }

    /// Return all known addresses.  Most likely this function is less
    /// useful than either [`local_addresses`](Self::local_addresses)
    /// or [`remote_addresses`](Self::remote_addresses).
    pub(crate) async fn all_known_addresses(&self) -> Vec<(Address, bool)> {
        self.routes
            .all()
            .await
            .into_iter()
            .map(|(addr, tt)| (addr, tt == RouteType::Local))
            .collect()
    }

    pub(crate) async fn local_addresses(&self) -> Vec<Address> {
        self.routes
            .all()
            .await
            .into_iter()
            .filter_map(|(addr, tt)| match tt {
                RouteType::Local => Some(addr),
                _ => None,
            })
            .collect()
    }

    pub(crate) async fn remote_addresses(&self) -> Vec<Address> {
        self.routes
            .all()
            .await
            .into_iter()
            .filter_map(|(addr, tt)| match tt {
                RouteType::Remote(_) => Some(addr),
                _ => None,
            })
            .collect()
    }
}
