// SPDX-FileCopyrightText: 2019-2023 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{
    context::RatmanContext,
    core::{DriverMap, Journal, RouteTable, RouteType},
    dispatch::BlockCollector,
    util::IoPair,
};
use async_std::{channel::bounded, sync::Arc, task};
use libratman::{
    netmod::InMemoryEnvelope,
    types::{frames::modes as fmodes, Recipient},
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
    drivers: Arc<DriverMap>,
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
        drivers: Arc<DriverMap>,
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

    /// Get one batch of messages from the driver interface points
    async fn run_batch(self: Arc<Self>, id: usize, batch_size: usize) {
        let (ep_name, ep) = self.drivers.get(id).await;

        for _ in 0..batch_size {
            let (InMemoryEnvelope { header, buffer }, t) = match ep.next().await {
                Ok(f) => f,
                _ => continue,
            };

            trace!("Receiving frame via from {}/{}", ep_name, t);

            // Examine the frame, it will be one of the following:
            //
            // - An announcement
            // - A data frame, addressed to a local address
            // - A manifest frame, addressed to a localaddress
            // - A data frame, to be forwarded
            // - A manifest frame, to be forwarded

            match (header.get_modes(), header.get_recipient()) {
                // For an announcement frame we ignore the recipient, even if it exists
                (fmodes::ANNOUNCE, _) => {
                    let announce_id = match header.get_seq_id() {
                        Some(seq) => seq.hash,
                        None => {
                            warn!("Received Announce frame with invalid SequenceId! Ignoring");
                            continue;
                        }
                    };

                    // Check that we haven't seen this announcement before
                    if self.journal.unknown(&announce_id).await {
                        self.routes.update(id as u8, t, header.get_sender()).await;
                        
                        // todo: reflood
                    }
                }
                // A data frame that is addressed to a particular address
                (fmodes::DATA, Some(Recipient::Target(address))) => {
                    // Check if the target address is "reachable"
                    match self.routes.reachable(address).await {
                        // A frame for a local address will be queued
                        // in the collector
                        Some(RouteType::Local) => {
                            if let Err(e) = self
                                .collector
                                .queue_and_spawn(InMemoryEnvelope { header, buffer })
                                .await
                            {
                                error!(
                                    "Faied to queue frame in sequence {:?}",
                                    header.get_seq_id()
                                );
                                continue;
                            }
                        }
                        // A frame for a reachable remote address will be forwarded
                        Some(RouteType::Remote(ep)) => todo!(),
                        // A frame for an unreachable address (either
                        // local or remote) will be queued in the
                        // journal
                        None => {
                            self.journal
                                .frame_queue(InMemoryEnvelope { header, buffer })
                                .await
                        }
                    }
                }
                // A data frame that is addressed to a network namespace
                (fmodes::DATA, Some(Recipient::Flood(ns))) => {}
                // A manifest frame that is addressed to a particular address
                (fmodes::MANIFEST, Some(Recipient::Target(address))) => {}
                // A manifest frame that is addressed to a network namespace
                (fmodes::MANIFEST, Some(Recipient::Flood(ns))) => {}
                // Unknown/ Invalid frame types get logged
                (_ftype, recipient) => {
                    warn!(
                        "Received unknown/invalid frame type {} (recipient: {:?})",
                        _ftype, recipient
                    );
                    continue;
                }
            }

            // // Switch the traffic to the appropriate place
            // use {Recipient::*, RouteType::*};
            // match f.recipient {
            //     Flood(_ns) => {
            //         let seqid = f.seq.seqid;
            //         if self.journal.unknown(&seqid).await {
            //             if let Some(sender) = Protocol::is_announce(&f) {
            //                 debug!("Received announcement for {}", sender);
            //                 self.routes.update(id as u8, t, sender).await;
            //             } else {
            //                 self.collector.queue_and_spawn(f.seqid(), f.clone()).await;
            //             }

            //             self.dispatch.reflood(f, id, t).await;
            //         }
            //     }
            //     ref recp @ Standard(_) => match recp.scope() {
            //         Some(scope) => match self.routes.reachable(scope).await {
            //             Some(Local) => self.collector.queue_and_spawn(f.seqid(), f).await,
            //             Some(Remote(_)) => self.dispatch.send_one(f).await.unwrap(),
            //             None => self.journal.queue(f).await,
            //         },
            //         None => {}
            //     },
            // }

            // Match on the modes bitfield to determine what kind of
            // frame we have
            match header.get_modes() {
                fmodes::ANNOUNCE => {
                    debug!("Reiceved announcement for {}", header.get_sender());
                    self.routes.update(id as u8, t, header.get_sender()).await;
                }
                fmodes::DATA if header.get_recipient().is_some() => {}
                fmodes::MANIFEST => {}
                f_type => {
                    warn!("Received unknown frame type: {}", f_type);
                }
            }

            // #[cfg(feature = "dashboard")]
            // {
            //     let metric_labels = &metrics::Labels {
            //         sender_id: f.sender,
            //         recp_type: (&f.recipient).into(),
            //         recp_id: f.recipient.scope().expect("empty recipient"),
            //     };
            //     self.metrics.frames_total.get_or_create(metric_labels).inc();
            //     self.metrics
            //         .bytes_total
            //         .get_or_create(metric_labels)
            //         .inc_by(f.payload.len() as u64);
            // }

            // match meta.modes {
            //     fmodes::ANNOUNCE => {}
            //     fmodes::DATA => {}
            //     fmodes::MANIFEST => {}
            //     t => {
            //         warn!("Unknown frame type: {}", t);
            //     }
            // }

            todo!()
        }
    }
}

#[cfg(feature = "dashboard")]
mod metrics {
    use libratman::types::{Address, ApiRecipient};
    use prometheus_client::{
        encoding::text::Encode,
        metrics::{counter::Counter, family::Family},
        registry::{Registry, Unit},
    };

    #[derive(Clone, Hash, PartialEq, Eq, Encode)]
    pub(super) struct Labels {
        pub sender_id: Address,
        pub recp_type: IdentityType,
        pub recp_id: Address,
    }

    #[derive(Clone, Hash, PartialEq, Eq)]
    pub(super) enum IdentityType {
        Standard,
        Flood,
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
