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

use crate::{context::RatmanContext, protocol::announce::AddressAnnouncer};
use libratman::{
    frame::carrier::{modes as fmodes, CarrierFrameHeader},
    tokio::{
        select,
        sync::{oneshot, Mutex},
        task::{self, spawn_local},
    },
    types::{AddrAuth, Address, Ident32},
    NonfatalError, RatmanError, Result,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
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
    online: Mutex<BTreeMap<Address, oneshot::Sender<()>>>,
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
        auth: AddrAuth,
        client_id: Ident32,
        ctx: Arc<RatmanContext>,
    ) -> Result<()> {
        let mut map = self.online.lock().await;

        let (tx, mut rx) = oneshot::channel::<()>();
        map.insert(address, tx);
        let announce_delay = (&ctx)
            .config
            .get_subtree("ratmand")
            .and_then(|subtree| subtree.get_number_value("announce_delay"))
            .unwrap_or_else(|| {
                debug!("ratmand/announce_delay was not set, assuming default of 2 seconds");
                2
            }) as u16;

        spawn_local(async move {
            // Split into a separate function to make tracing it easier
            info!("Starting announcer task for {}", address.pretty_string());
            let anon = match AddressAnnouncer::new(address, auth, client_id, &ctx).await {
                Ok(an) => an,
                Err(e) => {
                    error!("failed to start address announcer task: {e}");
                    return;
                }
            };

            loop {
                let ctx = Arc::clone(&ctx);
                select! {
                    biased;
                    _ = ctx.tripwire.clone() => break,
                    _ = &mut rx => break,
                    res = anon.run(announce_delay, &ctx) => {
                        match res {
                            Ok(_) => {},
                            Err(e) => {
                                error!("failed to send announcement: {e}");
                                break;
                            }
                        }
                    }
                }
            }

            info!("Address announcer {} shut down!", address.pretty_string());
        });

        Ok(())
    }

    pub(crate) async fn offline(&self, addr: Address) -> Result<()> {
        info!("Setting address {} to 'offline'", addr.pretty_string());
        self.online
            .lock()
            .await
            .remove(&addr)
            .ok_or(RatmanError::Nonfatal(NonfatalError::UnknownAddress(addr)))?;
        Ok(())
    }
}

/// Match the carrier modes bitfield to decide what kind of frame
pub fn parse(carrier_meta: CarrierFrameHeader, _bin_envelope: Vec<u8>) {
    match carrier_meta.get_modes() {
        fmodes::ANNOUNCE => {}
        fmodes::DATA => {}
        fmodes::MANIFEST => {}
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
