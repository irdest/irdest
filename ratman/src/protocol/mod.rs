// SPDX-FileCopyrightText: 2019-2024 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 embr <hi@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Protocol generation module

mod announce;
mod anycast;
mod router;

use crate::{context::RatmanContext, protocol::announce::AddressAnnouncer};
use libratman::{
    tokio::{
        select, spawn,
        sync::{
            mpsc::{self, Sender as MpscSender},
            oneshot, Mutex,
        },
    },
    types::{AddrAuth, Address, Namespace},
    NonfatalError, RatmanError, Result,
};
use std::{collections::BTreeMap, sync::Arc, time::Duration};

pub(crate) use router::RouterAnnouncement;

/// Provide a builder API to construct different types of Messages
#[derive(Default)]
pub(crate) struct Protocol {
    online: Mutex<BTreeMap<Address, oneshot::Sender<()>>>,
    anycasts: Mutex<BTreeMap<Namespace, MpscSender<(Address, Duration)>>>,
    online_namespaces: Mutex<BTreeMap<Namespace, Vec<Address>>>,
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

    pub(crate) async fn online_namespace(
        self: Arc<Self>,
        client_addr: Address,
        namespace: Namespace,
    ) -> Result<()> {
        let mut namespaces = self.online_namespaces.lock().await;
        namespaces.entry(namespace).or_default().push(client_addr);
        Ok(())
    }

    pub(crate) async fn offline_namespace(
        self: Arc<Self>,
        client_addr: Address,
        namespace: Namespace,
    ) -> Result<()> {
        let mut namespaces = self.online_namespaces.lock().await;

        if !namespaces.contains_key(&namespace) {
            return Err(RatmanError::ClientApi(libratman::ClientError::User(
                libratman::types::error::UserError::InvalidInput(
                    format!("Namespace {namespace} is not marked up"),
                    None,
                ),
            )));
        }
        let idx = namespaces
            .get(&namespace)
            .unwrap()
            .iter()
            .enumerate()
            .find(|(_, x)| *x == &client_addr)
            .map(|(idx, _)| idx)
            .ok_or(RatmanError::ClientApi(libratman::ClientError::User(
                libratman::types::error::UserError::InvalidInput(
                    format!("Client addrenn {client_addr} has not marked {namespace} as up"),
                    None,
                ),
            )))?;
        namespaces.get_mut(&namespace).unwrap().remove(idx);
        Ok(())
    }

    pub(crate) async fn get_namespace_listeners(
        self: &Arc<Self>,
        namespace: Namespace,
    ) -> Vec<Address> {
        self.online_namespaces
            .lock()
            .await
            .get(&namespace)
            .cloned()
            .unwrap_or(vec![])
    }

    /// Dispatch a task to announce a user periodically
    pub(crate) async fn online(
        self: Arc<Self>,
        address: Address,
        auth: AddrAuth,
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

        spawn(async move {
            // Split into a separate function to make tracing it easier
            info!("Starting address announcer for {}", address.pretty_string());
            let announcer = match AddressAnnouncer::new(address, auth, &ctx).await {
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
                    res = announcer.run(announce_delay, &ctx) => {
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

    pub(crate) async fn run_anycast_probe(
        &self,
        ctx: &Arc<RatmanContext>,
        self_addr: Address,
        namespace: Namespace,
        timeout: Duration,
    ) -> Result<Vec<(Address, Duration)>> {
        if self.anycasts.lock().await.get(&namespace).is_some() {
            error!("An anycast probe for namespace '{namespace}' is already running");
            return Ok(vec![]);
        }

        let (tx, rx) = mpsc::channel(128);

        self.anycasts.lock().await.insert(namespace, tx);

        let handler = anycast::AnycastProbeHandler {
            self_addr,
            namespace,
        };

        let response = handler.execute(Arc::clone(&ctx), timeout, rx).await;
        self.anycasts.lock().await.remove(&namespace);
        response
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
