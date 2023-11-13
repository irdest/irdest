// SPDX-FileCopyrightText: 2019-2023 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{
    context::RatmanContext,
    core::{LinksMap, Journal, RouteTable, RouteType},
    dispatch::BlockCollector,
    util::IoPair,
};
use async_std::{channel::bounded, sync::Arc, task};
use libratman::{
    netmod::InMemoryEnvelope,
    types::{
        frames::modes::{self as fmodes, DATA, MANIFEST},
        Recipient,
    },
};

use super::dispatch::Dispatch;

/// A frame switch inside Ratman to route packets and signals
///
/// The switch is given the job to poll endpoints in a loop and then
/// send the incoming frames to various points.
///
/// - Journal: the ID is not reachable
/// - Collector: the ID is local
pub(crate) struct Switch {
    /// Used only to check if the route is deemed reachable
    routes: Arc<RouteTable>,
    journal: Arc<Journal>,
    collector: Arc<BlockCollector>,
    drivers: Arc<LinksMap>,
    dispatch: Arc<Dispatch>,

    /// Control channel to start new endpoints
    ctrl: IoPair<usize>,

    #[cfg(feature = "dashboard")]
    metrics: Arc<metrics::Metrics>,
}

impl Switch {
    /// Create a new switch for the various routing components
    pub(crate) fn new(
        routes: Arc<RouteTable>,
        journal: Arc<Journal>,
        collector: Arc<BlockCollector>,
        drivers: Arc<LinksMap>,
        dispatch: Arc<Dispatch>,
    ) -> Arc<Self> {
        Arc::new(Self {
            routes,
            journal,
            collector,
            drivers,
            dispatch,
            ctrl: bounded(1),
            #[cfg(feature = "dashboard")]
            metrics: Arc::new(metrics::Metrics::default()),
        })
    }

    #[cfg(feature = "dashboard")]
    pub(crate) fn register_metrics(&self, registry: &mut prometheus_client::registry::Registry) {
        self.metrics.register(registry);
    }

    /// Add a new interface to the run switch
    pub(crate) async fn add(&self, id: usize) {
        self.ctrl.0.send(id).await.unwrap();
    }

    /// Dispatches a long-running task to run the switching logic
    pub(crate) fn run(self: Arc<Self>) {
        task::spawn(async move {
            while let Ok(i) = self.ctrl.1.recv().await {
                let switch = Arc::clone(&self);
                switch.run_batch(i, 1024).await;
            }
        });
    }

    /// Handle one batch of messages from the driver interface points
    async fn run_batch(self: Arc<Self>, id: usize, batch_size: usize) {
        let (ep_name, ep) = self.drivers.get(id).await;

        for _ in 0..batch_size {
            let (InMemoryEnvelope { header, buffer }, t) = match ep.next().await {
                Ok(f) => f,
                _ => continue,
            };
            trace!(
                "Received frame ({}) from {}/{}",
                match header.get_seq_id() {
                    Some(seq_id) => format!("{}", seq_id.hash),
                    None => format!("<???>"),
                },
                ep_name,
                t
            );

            // If the dashboard feature is enabled we first update the
            // metrics engine before all the data gets moved away >:(
            #[cfg(feature = "dashboard")]
            {
                let metric_labels = &metrics::Labels {
                    sender_id: header.get_sender(),
                    recp_type: header.get_recipient().as_ref().into(),
                    recp_id: header
                        .get_recipient()
                        .map(|r| r.address().to_string())
                        .unwrap_or_else(|| "None".to_owned()),
                };
                self.metrics.frames_total.get_or_create(metric_labels).inc();
                self.metrics
                    .bytes_total
                    .get_or_create(metric_labels)
                    .inc_by(header.get_payload_length() as u64);
            }

            // This is the core Ratman switch logic.  The accepted
            // frame header will be inspected below, and then the full
            // contents of the frame are handled appropriately.
            //
            // Currently a frame can be one of the following:
            //
            // - An announcement
            // - A data frame, to be forwarded
            // - A data frame, sent to a local address
            // - A manifest frame, to be forwarded
            // - A manifest frame, sent to a local address
            //
            // In future this function will also handle other
            // ratman-to-ratman protocols.

            ////////////////////////////////////////////
            //
            // Match on the header MODES and RECIPIENT
            //
            ////////////////////////////////////////////
            match (header.get_modes(), header.get_recipient()) {
                //
                // For an announcement frame we ignore the recipient, even if it exists
                (fmodes::ANNOUNCE, _) => {
                    let announce_id = match header.get_seq_id() {
                        Some(seq) => seq.hash,
                        None => {
                            warn!("Received Announce frame with invalid SequenceId! Ignoring");
                            continue;
                        }
                    };

                    // Check that we haven't seen this frame/ message
                    // ID before.  This prevents infinite replication
                    // of any flooded frame.
                    if self.journal.is_unknown(&announce_id).await {
                        self.journal.save_as_known(&announce_id).await;
                        // debug!("Received announcement for {}", header.get_sender());

                        // Update the routing table and re-flood the announcement
                        self.routes.update(id as u8, t, header.get_sender()).await;
                        self.dispatch
                            .flood_frame(
                                InMemoryEnvelope { header, buffer },
                                Some((ep_name.clone(), t)),
                            )
                            .await;
                    }
                }
                //
                // A data frame that is addressed to a particular address
                (mode, Some(Recipient::Target(address))) => {
                    // Check if the target address is "reachable"
                    match self.routes.reachable(address).await {
                        // A locally addressed data frame is inserted
                        // into the collector
                        Some(RouteType::Local) if mode == DATA => {
                            if let Err(e) = self
                                .collector
                                .queue_and_spawn(InMemoryEnvelope { header, buffer })
                                .await
                            {
                                error!(
                                    "Failed to queue frame in sequence {:?}",
                                    header.get_seq_id()
                                );
                                continue;
                            }
                        }
                        // A locally addressed manifest is given to
                        // the journal to collect
                        Some(RouteType::Local) if mode == MANIFEST => {
                            self.journal
                                .queue_manifest(InMemoryEnvelope { header, buffer })
                                .await;
                        }
                        // Any other frame types are currently ignored
                        Some(RouteType::Local) => {
                            warn!("Received invalid frame type: {}", mode);
                            continue;
                        }
                        // Any frame for a reachable remote address will be forwarded
                        Some(RouteType::Remote(_)) => {
                            self.dispatch
                                .dispatch_frame(InMemoryEnvelope { header, buffer })
                                .await;
                        }
                        // A frame for an unreachable address (either
                        // local or remote) will be queued in the
                        // journal
                        None => {
                            self.journal
                                .frame_queue(InMemoryEnvelope { header, buffer })
                                .await;
                        }
                    }
                }
                //
                // A data or manifest frame that is addressed to a network namespace
                (_, Some(Recipient::Flood(ns))) => {
                    let announce_id = match header.get_seq_id() {
                        Some(seq) => seq.hash,
                        None => {
                            warn!("Received Data::Flood frame with invalid SequenceId! Ignoring");
                            continue;
                        }
                    };

                    // todo: check if a local subscription for this
                    // namespace exists!

                    // If we haven seen this frame before, we keep
                    // track of it and then re-flood it into the
                    // network.
                    if self.journal.is_unknown(&announce_id).await {
                        self.journal.save_as_known(&announce_id).await;
                        self.dispatch
                            .flood_frame(InMemoryEnvelope { header, buffer }, None)
                            .await;
                    }
                }
                //
                // Unknown/ Invalid frame types get logged
                (_ftype, recipient) => {
                    warn!(
                        "Received unknown/invalid frame type {} (recipient: {:?})",
                        _ftype, recipient
                    );
                    continue;
                }
            }
        }
    }
}

#[cfg(feature = "dashboard")]
mod metrics {
    use libratman::types::{Address, ApiRecipient, Recipient};
    use prometheus_client::{
        encoding::text::Encode,
        metrics::{counter::Counter, family::Family},
        registry::{Registry, Unit},
    };

    #[derive(Clone, Hash, PartialEq, Eq, Encode)]
    pub(super) struct Labels {
        pub sender_id: Address,
        pub recp_type: IdentityType,
        // todo: can we make this an address type again?  Should we?
        pub recp_id: String,
    }

    #[derive(Clone, Hash, PartialEq, Eq)]
    pub(super) enum IdentityType {
        Standard,
        Flood,
        Empty,
    }

    impl From<Option<&Recipient>> for IdentityType {
        fn from(v: Option<&Recipient>) -> Self {
            match v {
                Some(&Recipient::Target(_)) => Self::Standard,
                Some(&Recipient::Flood(_)) => Self::Flood,
                None => Self::Empty,
            }
        }
    }

    impl From<&ApiRecipient> for IdentityType {
        fn from(v: &ApiRecipient) -> Self {
            match v {
                &ApiRecipient::Standard(_) => Self::Standard,
                &ApiRecipient::Flood(_) => Self::Flood,
            }
        }
    }

    // Manually implement Encode to produce eg. `recp_type=standard` rather than `recp_type=Standard`.
    impl Encode for IdentityType {
        fn encode(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
            match self {
                Self::Standard => write!(w, "standard"),
                Self::Flood => write!(w, "flood"),
                Self::Empty => write!(w, "none"),
            }
        }
    }

    #[derive(Default)]
    pub(super) struct Metrics {
        pub frames_total: Family<Labels, Counter>,
        pub bytes_total: Family<Labels, Counter>,
    }

    impl Metrics {
        pub fn register(&self, registry: &mut Registry) {
            registry.register(
                "ratman_switch_received_frames",
                "Total number of received frames",
                Box::new(self.frames_total.clone()),
            );
            registry.register_with_unit(
                "ratman_switch_received",
                "Total size of received frames",
                Unit::Bytes,
                Box::new(self.bytes_total.clone()),
            );
        }
    }
}
