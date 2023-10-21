// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 embr <hi@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Routing core components
//!
//! In previous designs (both code and docs) this was a single
//! component. This has proven to be a hard to maintain approach, so
//! instead the core has been split into several parts.

mod drivers;
mod journal;
mod routes;
mod switch;

pub(crate) use drivers::GenericEndpoint;
pub(crate) use journal::{Journal, JournalSender};

pub(self) use drivers::DriverMap;
pub(self) use routes::{EpTargetPair, RouteTable, RouteType};
pub(self) use switch::Switch;

use crate::dispatch::BlockCollector;
use async_std::sync::Arc;
use libratman::{
    netmod::{InMemoryEnvelope, Target},
    types::{Address, Message, RatmanError, Recipient, Result},
};

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
    journal: Arc<Journal>,
    routes: Arc<RouteTable>,
    switch: Arc<Switch>,
    drivers: Arc<DriverMap>,
}

impl Core {
    /// Initialises, but doesn't run the routing core
    pub(crate) fn init() -> Self {
        let drivers = DriverMap::new();
        let routes = RouteTable::new();
        let journal = Journal::new();

        let (jtx, jrx) = async_std::channel::bounded(16);
        let collector = BlockCollector::new(jtx);

        let switch = Switch::new(
            Arc::clone(&routes),
            Arc::clone(&journal),
            Arc::clone(&collector),
            Arc::clone(&drivers),
        );

        // Dispatch the runners
        Arc::clone(&switch).run();
        async_std::task::spawn(Arc::clone(&journal).run(jrx));

        Self {
            routes,
            collector,
            journal,
            switch,
            drivers,
        }
    }

    /// Register metrics with a Prometheus registry.
    #[cfg(feature = "dashboard")]
    pub fn register_metrics(&self, registry: &mut prometheus_client::registry::Registry) {
        self.routes.register_metrics(registry);
        self.switch.register_metrics(registry);
    }

    /// Dispatch a single frame
    pub(crate) async fn dispatch_frame(&self, envelope: InMemoryEnvelope) -> Result<()> {
        let target_address = match envelope.header.get_recipient() {
            Some(Recipient::Target(addr)) => addr,
            // fixme: introduce a better error kind here
            _ => unreachable!(),
        };

        let EpTargetPair(epid, trgt) = match self.routes.resolve(target_address).await {
            // Return the endpoint/target ID pair from the resolver
            Some(resolve) => resolve,

            None => return Err(RatmanError::NoSuchAddress(target_address)),
        };

        let (_, ep) = self.drivers.get(epid as usize).await;
        ep.send(envelope, trgt, None).await
    }

    pub(crate) async fn flood_frame(&self, envelope: InMemoryEnvelope) -> Result<()> {
        let flood_address = match envelope.header.get_recipient() {
            Some(Recipient::Flood(addr)) => addr,
            // fixme: introduce a better error kind here
            _ => unreachable!(),
        };

        // Loop over every driver and send a version of the envelope to it
        for (ep_name, ep) in self.drivers.get_all().await.into_iter() {
            let env = envelope.clone();
            let target = Target::Flood(flood_address);
            if let Err(e) = ep.send(env, target, None).await {
                error!(
                    "failed to flood frame {:?} on endpoint {}: {}",
                    envelope.header.get_seq_id(),
                    ep_name,
                    e
                );
            }
        }

        Ok(())
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

    pub(crate) async fn next(&self) -> Message {
        self.journal.next().await
    }

    /// Insert a new endpoint
    pub(crate) async fn add_ep(&self, name: String, ep: Arc<GenericEndpoint>) -> usize {
        let id = self.drivers.add(name, ep).await;
        self.switch.add(id).await;
        id
    }

    /// Get an endpoint back from the driver set via it's ID
    pub(crate) async fn get_ep(&self, id: usize) -> (String, Arc<GenericEndpoint>) {
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

    // fixme: this is basically just moving the hard-coded value somewhere else
    pub(crate) fn get_route_mtu(&self, _recipient: Option<Recipient>) -> u16 {
        400
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
