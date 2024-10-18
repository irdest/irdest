// SPDX-FileCopyrightText: 2019-2024 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{
    journal::Journal,
    links::{GenericEndpoint, LinksMap},
    procedures,
    protocol::Protocol,
    routes::{EpNeighbourPair, RouteTable},
};
use chrono::Utc;
use libratman::{
    frame::carrier::modes::{self as fmodes, DATA, MANIFEST, NAMESPACE_ANYCAST},
    frame::{
        carrier::{AnnounceFrame, CarrierFrameHeader},
        FrameParser,
    },
    types::{InMemoryEnvelope, Recipient},
    NetmodError, RatmanError,
};
use libratman::{
    tokio::{
        select,
        sync::{broadcast::Sender as BcastSender, mpsc::Sender},
        task::yield_now,
    },
    types::RouterMeta,
};
use std::sync::Arc;
use tripwire::Tripwire;

use super::{ingress::MessageNotifier, BlockCollector, BlockNotifier};

/// Run a batch of receive jobs for a given endpoint and state context
pub(crate) async fn exec_switching_batch(
    // Current netmod ID us
    id: usize,
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
    // Reference to protocol tracker
    protocol: &Arc<Protocol>,
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
    loop {
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
        let payload_slice = header.get_size()..;

        trace!(
            "[{ep_name}] received frame of type {}",
            fmodes::str_name(header.get_modes())
        );

        ////////////////////////////////////////////
        //
        // Match on the header MODES and RECIPIENT
        //
        ////////////////////////////////////////////
        match (header.get_modes(), header.get_recipient()) {
            // We handle router announcements by updating the available
            // advertised buffer space for a neighbour, and then DON'T spread it
            // further
            (fmodes::ROUTER_PEERING, _) => {
                let payload_buf = &buffer.as_slice()[payload_slice];

                match RouterMeta::parse(payload_buf) {
                    Ok((_, router_meta)) => {
                        *(routes
                            .solver_state
                            .write()
                            .await
                            .available_buffer
                            .entry(EpNeighbourPair(id, neighbour.assume_single()))
                            .or_default()) = router_meta.available_buffer;

                        // todo
                    }
                    Err(e) => {
                        warn!("Received invalid RouterMeta payload: {e}");
                        continue;
                    }
                }
            }
            //
            // For an announcement frame we ignore the recipient, even if it exists
            (fmodes::ANNOUNCE, _) => {
                let announce_id = match header.get_seq_id() {
                    Some(seq) => seq.hash,
                    None => {
                        warn!("Received Announce frame with no SequenceId! Ignoring");
                        continue;
                    }
                };

                // Check that we haven't seen this frame/ message ID before.
                // This prevents infinite replication of any flooded frame.
                if journal.is_unknown(&announce_id).await.is_ok() {
                    journal.save_as_known(&announce_id).await.unwrap();
                    debug!("Received announcement for {}", header.get_sender());

                    let announce_buf = &buffer.as_slice()[payload_slice];

                    match AnnounceFrame::parse(announce_buf) {
                        Ok((remainder, Ok(announce_frame))) => {
                            // fixme: fail softly ;-;
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
                        }
                        Ok((remainder, Err(e))) => {
                            error!(
                                "Failed to parse announcement: [remains:{remainder:?}] [error:{e}]"
                            );
                        }
                        Err(e) => {
                            error!("Completely failed announce parsing: {e}");
                        }
                    }
                }
            }
            //
            // Handle anycast requests
            (mode, Some(Recipient::Namespace(namespace))) if mode == NAMESPACE_ANYCAST => {
                let now = Utc::now();

                // If the namespace isn't actually listened to on this node this
                // loop will return nothing and we don't send any reply
                let local_addrs = protocol.get_namespace_listeners(namespace).await;
                for addr in local_addrs {
                    if let Err(e) = procedures::dispatch_frame(
                        routes,
                        links,
                        collector,
                        block_notify_tx.clone(),
                        InMemoryEnvelope::from_header_and_payload(
                            CarrierFrameHeader::new_anycast_reply_frame(
                                addr,
                                Recipient::Address(header.get_sender()),
                                now.clone(),
                            )
                            .expect("failed to encode anycast probe response"),
                            vec![],
                        )
                        .expect("failed to encode anycast probe response"),
                    )
                    .await
                    {
                        error!("failed to send anycast probe response: {e}");
                        continue;
                    }
                }
            }

            ///////////////////////////////////////////////////////////
            //
            // Any frame that's addressed to an address
            (mode, Some(Recipient::Address(address))) if mode == DATA || mode == MANIFEST => {
                trace!("Received [mode:{mode}] frame for {address}");

                // Check if the target address is "reachable"
                match routes.reachable(address).await {
                    // Any frame for a reachable remote address will be forwarded
                    Some(_) => {
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
                                if let Err(e) = journal.queue_frame(envelope).await {
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
                    // A route entry with actual route data is a local address
                    None => {
                        // If it's a manifest we queue that and start the ingress machine
                        if mode == MANIFEST {
                            let manifest_id = header.get_seq_id().unwrap().hash;

                            if let Err(e) = journal
                                .queue_manifest(InMemoryEnvelope { header, buffer })
                                .await
                            {
                                error!("failed to queue manifest: {e}");
                            }

                            if let Err(e) = ingress_tx.send(MessageNotifier(manifest_id)).await {
                                error!("failed to notify local task of manifest: {e}");
                            }
                        }
                        // Otherwise it's a data frame and we queue it in the collector
                        else {
                            trace!("Queue locally addressed frame in collector");

                            if let Err(e) =
                                collector_tx.send(InMemoryEnvelope { header, buffer }).await
                            {
                                let (s_hash, s_num, s_max) = match header.get_seq_id() {
                                    Some(seq) => (seq.hash.pretty_string(), seq.num, seq.max),
                                    None => ("unknown".to_string(), 0, 0),
                                };

                                error!(
                                    "Failed to queue frame in sequence {}[{}/{}]: {e}",
                                    s_hash, s_num, s_max
                                );
                                continue;
                            }
                        }
                    }
                }
            }
            //
            // A data or manifest frame that is addressed to a network namespace
            (_, Some(Recipient::Namespace(space))) => {
                trace!("Received data frame for {space}");

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
                if journal.is_unknown(&announce_id).await.unwrap() {
                    journal.save_as_known(&announce_id).await.unwrap();
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

        yield_now().await;
    }

    info!("Switch loop has terminated!");
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
