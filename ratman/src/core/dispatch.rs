// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
// SPDX-FileCopyrightText: 2022 embr <hi@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Asynchronous Ratman routing core

use crate::core::{EpNeighbourPair, LinksMap, RouteTable};
use libratman::{
    types::Recipient,
    types::{InMemoryEnvelope, Neighbour},
    RatmanError, Result,
};
use std::sync::Arc;

/// Resolve the target address and dispatch the frame
///
/// Returns an error if resolving or sending failed
pub(crate) async fn dispatch_frame(
    routes: &Arc<RouteTable>,
    drivers: &Arc<LinksMap>,
    envelope: InMemoryEnvelope,
) -> Result<()> {
    trace!(
        "Dispatch frame in sequence {}",
        match envelope.header.get_seq_id() {
            Some(seq_id) => format!("{}", seq_id.hash),
            None => format!("<???>"),
        }
    );
    let target_address = match envelope.header.get_recipient() {
        Some(Recipient::Address(addr)) => addr,
        // fixme: introduce a better error kind here
        _ => unreachable!(),
    };

    let EpNeighbourPair(epid, nb) = match routes.resolve(target_address).await {
        // Return the endpoint/target ID pair from the resolver
        Some(resolve) => resolve,

        None => return Err(RatmanError::NoSuchAddress(target_address)),
    };

    let (_, ep) = drivers.get(epid as usize).await;
    ep.send(envelope, nb, None).await
}

// todo: implement the exception mechanism
pub(crate) async fn flood_frame(
    _routes: &Arc<RouteTable>,
    drivers: &Arc<LinksMap>,
    envelope: InMemoryEnvelope,
    _except: Option<(String, Neighbour)>,
) -> Result<()> {
    // Loop over every driver and send a version of the envelope to it
    for (ep_name, ep) in drivers.get_all().await.into_iter() {
        let env = envelope.clone();
        if let Err(e) = ep.send(env, Neighbour::Flood, None).await {
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

// #[cfg(feature = "dashboard")]
// mod metrics {
//     use libratman::types::{Address, ApiRecipient};
//     use prometheus_client::{
//         encoding::text::Encode,
//         metrics::{counter::Counter, family::Family},
//         registry::{Registry, Unit},
//     };

//     #[derive(Clone, Hash, PartialEq, Eq, Encode)]
//     pub(super) struct Labels {
//         pub recp_type: RecipientType,
//         pub recp_id: Address,
//     }

//     #[derive(Clone, Hash, PartialEq, Eq)]
//     pub(super) enum RecipientType {
//         Standard,
//         Flood,
//     }

//     impl From<&ApiRecipient> for RecipientType {
//         fn from(v: &ApiRecipient) -> Self {
//             match v {
//                 &ApiRecipient::Standard(_) => Self::Standard,
//                 &ApiRecipient::Flood(_) => Self::Flood,
//             }
//         }
//     }

//     // Manually implement Encode to produce eg. `recipient=standard` rather than `recipient=Standard`.
//     impl Encode for RecipientType {
//         fn encode(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
//             match self {
//                 Self::Standard => write!(w, "standard"),
//                 Self::Flood => write!(w, "flood"),
//             }
//         }
//     }

//     #[derive(Default)]
//     pub(super) struct Metrics {
//         pub messages_total: Family<Labels, Counter>,
//         pub frames_total: Family<Labels, Counter>,
//         pub bytes_total: Family<Labels, Counter>,
//     }

//     impl Metrics {
//         pub fn register(&self, registry: &mut Registry) {
//             registry.register(
//                 "ratman_dispatch_messages",
//                 "Total number of messages dispatched",
//                 Box::new(self.messages_total.clone()),
//             );
//             registry.register(
//                 "ratman_dispatch_frames",
//                 "Total number of frames dispatched",
//                 Box::new(self.frames_total.clone()),
//             );
//             registry.register_with_unit(
//                 "ratman_dispatch",
//                 "Total size of dispatched frames",
//                 Unit::Bytes,
//                 Box::new(self.bytes_total.clone()),
//             );
//         }
//     }
// }
