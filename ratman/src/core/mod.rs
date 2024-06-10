// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 embr <hi@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Routing core components
//!
//! In previous designs (both code and docs) this was a single
//! component. This has proven to be a hard to maintain approach, so
//! instead the core has been split into several parts.

pub mod dispatch;
mod ingress;
mod links;
mod routes;

pub(crate) use crate::dispatch::BlockCollector;
pub(crate) use links::{GenericEndpoint, LinksMap};
pub(crate) use routes::{EpNeighbourPair, RouteTable, RouteType};

// use self::ingress::run_message_assembler;

// impl Core {

// /// Register metrics with a Prometheus registry.
// #[cfg(feature = "dashboard")]
// pub fn register_metrics(&self, registry: &mut prometheus_client::registry::Registry) {
//     self.routes.register_metrics(registry);
//     self.switch.register_metrics(registry);
// }

// /// Returns users that were newly discovered in the network
// pub(crate) async fn discover(&self) -> Address {
//     self.routes.discover().await
// }

// pub(crate) async fn next(&self) -> Message {
//     match self.journal.next_message().await {
//         Some(m) => m,
//         // fixme: handle this error a bit more graciously.  we may
//         // be able to restart the journal if it has crashed, and
//         // we should definitely check if the router _should_ even
//         // still be running and if this error is entirely
//         // expected.
//         None => panic!("Message queue ran to its end, but the router is still running ???"),
//     }
// }

// /// Insert a new endpoint
// pub(crate) async fn add_ep(&self, name: String, ep: Arc<GenericEndpoint>) -> usize {
//     let id = self.drivers.add(name, ep).await;
//     self.switch.add(id).await;
//     id
// }

// /// Get an endpoint back from the driver set via it's ID
// pub(crate) async fn get_ep(&self, id: usize) -> (String, Arc<GenericEndpoint>) {
//     self.drivers.get(id).await
// }

// /// Remove an endpoint
// pub(crate) async fn rm_ep(&self, id: usize) {
//     self.drivers.remove(id).await;
// }

// /// Add a local address to the routing table
// pub(crate) async fn add_local_address(&self, id: Address) -> Result<()> {
//     self.routes.add_local(id).await
// }

// /// Remove a local address from the routing table
// pub(crate) async fn remove_local_address(&self, id: Address) -> Result<()> {
//     self.routes.delete(id).await
// }

// // fixme: this is basically just moving the hard-coded value somewhere else
// pub(crate) fn get_route_mtu(&self, _recipient: Option<Recipient>) -> u16 {
//     1200
// }

//     /// Return all known addresses.  Most likely this function is less
//     /// useful than either [`local_addresses`](Self::local_addresses)
//     /// or [`remote_addresses`](Self::remote_addresses).
//     pub(crate) async fn all_known_addresses(&self) -> Vec<(Address, bool)> {
//         self.routes
//             .all()
//             .await
//             .into_iter()
//             .map(|(addr, tt)| (addr, tt == RouteType::Local))
//             .collect()
//     }

//     pub(crate) async fn local_addresses(&self) -> Vec<Address> {
//         self.routes
//             .all()
//             .await
//             .into_iter()
//             .filter_map(|(addr, tt)| match tt {
//                 RouteType::Local => Some(addr),
//                 _ => None,
//             })
//             .collect()
//     }

//     pub(crate) async fn remote_addresses(&self) -> Vec<Address> {
//         self.routes
//             .all()
//             .await
//             .into_iter()
//             .filter_map(|(addr, tt)| match tt {
//                 RouteType::Remote(_) => Some(addr),
//                 _ => None,
//             })
//             .collect()
//     }
// }
