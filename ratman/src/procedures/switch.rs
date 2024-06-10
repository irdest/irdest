use crate::{
    core::{dispatch, GenericEndpoint, LinksMap, RouteTable, RouteType},
    dispatch::BlockCollector,
    journal::Journal,
    util::IoPair,
};
use libratman::tokio::task::spawn_local;
use libratman::{
    endpoint::EndpointExt,
    frame::carrier::modes::{self as fmodes, DATA, MANIFEST},
    types::{InMemoryEnvelope, Recipient},
};
use std::sync::Arc;

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
    // The switch dispatches locally addressed frames into the block
    // collector
    collector: &Arc<BlockCollector>,
    // The netmod driver endpoint to switch messages for
    (ep_name, ep): (&String, &Arc<GenericEndpoint>),
    // Control flow endpoint to send signals to this switch between
    // batches
    _ctrl: IoPair<usize>,
    // Metrics collector state to allow diagnostic analysis of this
    // procedure
    // #[cfg(feature = "dashboard")] metrics: &Arc<metrics::Metrics>,
) {
    for _ in 0..batch_size {
        let (InMemoryEnvelope { header, buffer }, t) = match ep.next().await {
            Ok(f) => f,
            _ => continue,
        };

        trace!(
            "Received frame ({}) from {}/{:?}",
            match header.get_seq_id() {
                Some(seq_id) => format!("{}", seq_id.hash),
                None => format!("<???>"),
            },
            ep_name,
            t
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

                // Check that we haven't seen this frame/ message
                // ID before.  This prevents infinite replication
                // of any flooded frame.
                if journal.is_unknown(&announce_id).is_ok() {
                    journal.save_as_known(&announce_id).unwrap();
                    debug!("Received announcement for {}", header.get_sender());

                    // Update the routing table and re-flood the announcement
                    routes.update(id as u8, t, header.get_sender()).await;
                    dispatch::flood_frame(
                        &routes,
                        &links,
                        InMemoryEnvelope { header, buffer },
                        Some((ep_name.clone(), t)),
                    )
                    .await;
                }
            }
            //
            // A data frame that is addressed to a particular address
            (mode, Some(Recipient::Address(address))) => {
                // Check if the target address is "reachable"
                match routes.reachable(address).await {
                    // A locally addressed data frame is inserted
                    // into the collector
                    Some(RouteType::Local) if mode == DATA => {
                        if let Err(_e) = collector
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
                        let journal = Arc::clone(&journal);
                        spawn_local(async move {
                            if let Err(e) =
                                journal.queue_manifest(InMemoryEnvelope { header, buffer })
                            {
                                error!("failed to queue Manifest: {e:?}.  This will result in unrecoverable blocks");
                            }

                            // todo: setup consolidation task
                        });
                    }
                    // Any other frame types are currently ignored
                    Some(RouteType::Local) => {
                        warn!("Received invalid frame type: {}", mode);
                        continue;
                    }
                    // Any frame for a reachable remote address will be forwarded
                    Some(RouteType::Remote(_)) => {
                        dispatch::dispatch_frame(
                            routes,
                            links,
                            InMemoryEnvelope { header, buffer },
                        )
                        .await;
                    }
                    // A frame for an unreachable address (either
                    // local or remote) will be queued in the
                    // journal
                    None => {
                        let journal = Arc::clone(&journal);
                        spawn_local(async move {
                            journal.queue_frame(InMemoryEnvelope { header, buffer });
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
                    dispatch::flood_frame(routes, links, InMemoryEnvelope { header, buffer }, None)
                        .await;
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

#[cfg(feature = "dashboard")]
pub(crate) use self::metrics as switch_metrics;

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
