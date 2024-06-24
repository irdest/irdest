// SPDX-FileCopyrightText: 2019-2024 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{
    journal::Journal,
    links::{GenericEndpoint, LinksMap},
    procedures::{self},
    routes::{EpNeighbourPair, RouteTable},
    storage::route::RouteState,
};
use libratman::tokio::{
    select, sync::broadcast::Sender as BcastSender, sync::mpsc::Sender, task::spawn,
};
use libratman::{
    frame::carrier::modes::{self as fmodes, DATA, MANIFEST},
    frame::{carrier::AnnounceFrame, FrameParser},
    types::{InMemoryEnvelope, Recipient},
    NetmodError, RatmanError,
};
use std::sync::Arc;
use tripwire::Tripwire;

use super::{ingress::MessageNotifier, BlockCollector, BlockNotifier};

/// Run a batch of receive jobs for a given endpoint and state context
pub(crate) async fn exec_switching_batch(
    // Current netmod ID us
    id: usize,
    // Always run a batch of receive jobs to mitigate excessive
    // context switching.  Batch limit should be dynamically chosen on
    // load, but can be overriden for specific usecases
    batch_size: usize,
    // Routes container to update tables based on announcements
    routes: &Arc<RouteTable>,
    // Even though this switch only runs for a single endpoint, we
    // still need access to the other links when flooding messages.
    // This way we don't have to make different switches explicitly
    // synchronise.
    links: &Arc<LinksMap>,
    // The switch needs access to the central state manager for blocks
    // and frames
    journal: &Arc<Journal>,
    // Allow spawning frames on the local collector that are addressed
    // to a local address
    collector: &Arc<BlockCollector>,
    // Allow the switch to shut down gracefully
    tripwire: Tripwire,
    // The netmod driver endpoint to switch messages for
    (ep_name, ep): (&String, &Arc<GenericEndpoint>),
    // Control flow endpoint to send signals to this switch between batches
    ingress_tx: Sender<MessageNotifier>,
    collector_tx: Sender<InMemoryEnvelope>,
    // We only take the sender because we can spawn receivers from it
    // with .subscribe()
    block_notify_tx: BcastSender<BlockNotifier>,
    // Metrics collector state to allow diagnostic analysis of this
    // procedure
    // #[cfg(feature = "dashboard")] metrics: &Arc<metrics::Metrics>,
) {
    for _ in 0..batch_size {
        let (InMemoryEnvelope { header, buffer }, neighbour) = select! {
            biased;
            _ = tripwire.clone() => break,
            item = ep.next() => {
                match item {
                    Ok(f) => f,
                    _ => continue,
                }
            }
        };

        let block_notify_tx = block_notify_tx.clone();

        trace!(
            "Received frame ({}) from {}/{:?}",
            match header.get_seq_id() {
                Some(seq_id) => format!("{}", seq_id.hash),
                None => format!("<???>"),
            },
            ep_name,
            neighbour,
        );

        // If the dashboard feature is enabled we first update the
        // metrics engine before all the data gets moved away >:(
        // #[cfg(feature = "dashboard")]
        // {
        //     let metric_labels = &metrics::Labels {
        //         sender_id: header.get_sender(),
        //         recp_type: header.get_recipient().as_ref().into(),
        //         recp_id: header
        //             .get_recipient()
        //             .map(|r| r.inner_address().to_string())
        //             .unwrap_or_else(|| "None".to_owned()),
        //     };
        //     metrics.frames_total.get_or_create(metric_labels).inc();
        //     metrics
        //         .bytes_total
        //         .get_or_create(metric_labels)
        //         .inc_by(header.get_payload_length() as u64);
        // }

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

                // Check that we haven't seen this frame/ message ID before.
                // This prevents infinite replication of any flooded frame.
                if journal.is_unknown(&announce_id).is_ok() {
                    journal.save_as_known(&announce_id).unwrap();
                    debug!("Received announcement for {}", header.get_sender());

                    if let Ok((remainder, Ok(announce_frame))) =
                        AnnounceFrame::parse(buffer.as_slice())
                    {
                        // fail softly ;-;
                        assert!(remainder.len() == 0);

                        // Update the routing table and re-flood the announcement
                        if let Err(e) = routes
                            .update(
                                EpNeighbourPair(id, neighbour.assume_single()),
                                header.get_sender(),
                                match announce_frame {
                                    AnnounceFrame::V1(v1) => v1,
                                },
                            )
                            .await
                        {
                            warn!("failed to update route for peer {id}: {e}");
                            continue;
                        }

                        if let Err(e) = procedures::flood_frame(
                            &routes,
                            &links,
                            InMemoryEnvelope { header, buffer },
                            neighbour.maybe_single(),
                        )
                        .await
                        {
                            error!("failed to flood announcement frame: {e:?}");
                        }
                    } else {
                        warn!("Received invalid AnnounceFrame: ignoring route update");
                    }
                }
            }
            //
            // A data frame that is addressed to a particular address
            (mode, Some(Recipient::Address(address))) => {
                // Check if the target address is "reachable"
                match routes.reachable(address).await {
                    // A locally addressed data frame is queued on the collector
                    Some(RouteState::Active) if mode == DATA => {
                        if let Err(_e) =
                            collector_tx.send(InMemoryEnvelope { header, buffer }).await
                        {
                            error!(
                                "Failed to queue frame in sequence {:?}",
                                header.get_seq_id()
                            );
                            continue;
                        }
                    }
                    // Locally addressed manifest is cached in the journal, then
                    // we notify the ingress system to start collecting the full
                    // message stream.
                    Some(RouteState::Active) if mode == MANIFEST => {
                        let journal = Arc::clone(&journal);
                        let ingress_tx = ingress_tx.clone();
                        spawn(async move {
                            if let Err(e) =
                                journal.queue_manifest(InMemoryEnvelope { header, buffer })
                            {
                                error!("failed to queue Manifest: {e:?}.  This will result in unrecoverable blocks");
                            }
                            if let Err(e) = ingress_tx.send(MessageNotifier(header.get_seq_id().unwrap().hash)).await {
                                error!("failed to notify ingress system for manifest: {e}");
                            }
                        }).await.unwrap();
                    }
                    // Any frame for a reachable remote address will be forwarded
                    Some(RouteState::Active) => {
                        match procedures::dispatch_frame(
                            routes,
                            links,
                            collector,
                            block_notify_tx.clone(),
                            InMemoryEnvelope { header, buffer },
                        )
                        .await
                        {
                            Err(RatmanError::Netmod(NetmodError::ConnectionLost(envelope))) => {
                                debug!("Connection dropped while forwarding frame!  Message contents were saved");
                                if let Err(e) = journal.queue_frame(envelope) {
                                    error!("failed to queue frame to journal: {e}!  Data has been dropped");
                                }
                            }
                            Err(e) => {
                                warn!("Error occured while dispatching frame: {e}!  Message contents were lost");
                                continue;
                            }
                            _ => continue,
                        }
                    }
                    Some(RouteState::Idle) | Some(RouteState::Lost) => {
                        warn!("offline buffering not yet implemented >_<");
                    }
                    // A frame for an unreachable address (either
                    // local or remote) will be queued in the
                    // journal
                    None => {
                        let journal = Arc::clone(&journal);
                        spawn(async move {
                            if let Err(e) = journal.queue_frame(InMemoryEnvelope { header, buffer })
                            {
                                error!("failed to queue frame to journal: {e:?}");
                            }
                        });
                    }
                }
            }
            //
            // A data or manifest frame that is addressed to a network namespace
            (_, Some(Recipient::Namespace(_ns))) => {
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
                if journal.is_unknown(&announce_id).unwrap() {
                    journal.save_as_known(&announce_id).unwrap();
                    if let Err(e) = procedures::flood_frame(
                        routes,
                        links,
                        InMemoryEnvelope { header, buffer },
                        None,
                    )
                    .await
                    {
                        error!("failed to flood frame to namespace: {e:?}");
                    }
                }
            }
            //
            // Unknown/ Invalid frame types get logged
            (_ftype, recipient) => {
                warn!(
                    "Ignoring unknown/invalid frame type {} (recipient: {:?})",
                    _ftype, recipient
                );
                continue;
            }
        }
    }
}

// #[cfg(feature = "dashboard")]
// pub(crate) use self::metrics as switch_metrics;

#[allow(unused)]
#[cfg(feature = "dashboard")]
pub(crate) mod metrics {
    use libratman::types::{Address, Recipient};
    use prometheus_client::{
        encoding::text::Encode,
        metrics::{counter::Counter, family::Family},
        registry::{Registry, Unit},
    };

    #[derive(Clone, Hash, PartialEq, Eq, Encode)]
    pub(crate) struct Labels {
        pub sender_id: Address,
        pub recp_type: IdentityType,
        // todo: can we make this an address type again?  Should we?
        pub recp_id: String,
    }

    #[derive(Clone, Hash, PartialEq, Eq)]
    pub(crate) enum IdentityType {
        Standard,
        Flood,
        Empty,
    }

    impl From<Option<&Recipient>> for IdentityType {
        fn from(v: Option<&Recipient>) -> Self {
            match v {
                Some(&Recipient::Address(_)) => Self::Standard,
                Some(&Recipient::Namespace(_)) => Self::Flood,
                None => Self::Empty,
            }
        }
    }

    impl From<&Recipient> for IdentityType {
        fn from(v: &Recipient) -> Self {
            match v {
                &Recipient::Address(_) => Self::Standard,
                &Recipient::Namespace(_) => Self::Flood,
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
    pub(crate) struct Metrics {
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
