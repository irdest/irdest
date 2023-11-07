// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 embr <hi@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Protocol generation module
//!
//! The routing protocol, and micro messages (analogous to micro
//! code), are much better documented in the `R.A.T.M.A.N.` design
//! specification/paper. But here's a brief overview, and
//! implementation:
//!
//! - `Announce` is sent when a node comes online
//! - `Sync` is a reply to an `Announce`, only omitted when `no_sync` is set

mod announce;

use crate::{
    config::ConfigTree, context::RatmanContext, core::Core, protocol::announce::AddressAnnouncer,
};
use libratman::types::{
    frames::{self, CarrierFrameHeader},
    Address, RatmanError, Result,
};

use async_std::{
    sync::{Arc, Mutex},
    task,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

/// A payload that represents a RATMAN-protocol message
#[derive(Debug, Serialize, Deserialize)]
enum ProtoPayload {
    /// A network-wide announcement message
    Announce { id: Address, no_sync: bool },
}

/// Provide a builder API to construct different types of Messages
#[derive(Default)]
pub(crate) struct Protocol {
    online: Mutex<BTreeMap<Address, Arc<AtomicBool>>>,
    #[cfg(feature = "dashboard")]
    metrics: metrics::Metrics,
}

impl Protocol {
    pub(crate) fn new() -> Arc<Self> {
        Default::default()
    }

    #[cfg(feature = "dashboard")]
    pub fn register_metrics(&self, registry: &mut prometheus_client::registry::Registry) {
        self.metrics.register(registry);
    }

    /// Dispatch a task to announce a user periodically
    pub(crate) async fn online(
        self: Arc<Self>,
        address: Address,
        ctx: Arc<RatmanContext>,
    ) -> Result<()> {
        let mut map = self.online.lock().await;
        if map.get(&address).map(|arc| arc.load(Ordering::Relaxed)) == Some(true) {
            // If a user is already online we don't have to do anything
            return Ok(());
        }

        info!("Setting address {} to 'online'", address);

        let b = Arc::new(AtomicBool::new(true));
        map.insert(address, Arc::clone(&b));
        drop(map);

        let announce_delay = ctx
            .config
            .get_subtree("ratmand")
            .and_then(|subtree| subtree.get_number_value("announce_delay"))
            .unwrap_or_else(|| {
                debug!("ratmand/announce_delay was not set, assuming default of 2 seconds");
                2
            }) as u16;

        task::spawn(async move {
            AddressAnnouncer(address).run(b, announce_delay, ctx).await;
        });

        Ok(())
    }

    pub(crate) async fn offline(&self, id: Address) -> Result<()> {
        info!("Setting address {} to 'offline'", id);
        self.online
            .lock()
            .await
            .get(&id)
            .map(|b| b.swap(false, Ordering::Relaxed))
            .map_or(Err(RatmanError::NoSuchAddress(id)), |_| Ok(()))
    }

    // /// Try to parse a frame as an announcement
    // pub(crate) fn is_announce(f: &Frame) -> Option<Address> {
    //     let Frame { ref payload, .. } = f;

    //     bincode::deserialize(payload)
    //         .map(|p| match p {
    //             ProtoPayload::Announce { id, .. } => id,
    //         })
    //         .ok()
    // }

    // /// Build an announcement message for a user
    // fn announce(sender: Address) -> Frame {
    //     let payload = bincode::serialize(&ProtoPayload::Announce {
    //         id: sender,
    //         no_sync: true,
    //     })
    //     .unwrap();

    //     // Currently we just use the sender address as the "scope" of the
    //     Frame::inline_flood(sender, sender, payload)
    // }
}

/// Match the carrier modes bitfield to decide what kind of frame
pub fn parse(carrier_meta: CarrierFrameHeader, bin_envelope: Vec<u8>) {
    match carrier_meta.get_modes() {
        frames::modes::ANNOUNCE => {}
        frames::modes::DATA => {}
        frames::modes::MANIFEST => {}
        _ => {
            warn!("received frame with malformed metadata: {:?}", carrier_meta);
        }
    }
}

#[cfg(feature = "dashboard")]
mod metrics {
    use prometheus_client::{metrics::counter::Counter, registry::Registry};

    #[derive(Default)]
    pub(super) struct Metrics {
        pub announcements_total: Counter,
    }

    impl Metrics {
        pub fn register(&self, registry: &mut Registry) {
            registry.register(
                "ratman_proto_announcements",
                "Total number of announcements sent",
                Box::new(self.announcements_total.clone()),
            );
        }
    }
}
