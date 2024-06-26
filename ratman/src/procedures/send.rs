// SPDX-FileCopyrightText: 2019-2024 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
// SPDX-FileCopyrightText: 2022 embr <hi@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Asynchronous Ratman routing core

use super::{
    ingress::MessageNotifier, slicer::BlockSlicer, BlockCollector, BlockNotifier, BlockWorker,
};
use crate::{
    journal::Journal,
    links::LinksMap,
    routes::{EpNeighbourPair, RouteTable},
};
use async_eris::{Block, ReadCapability};
use colored::Colorize;
use libratman::{
    frame::{
        carrier::{CarrierFrameHeader, ManifestFrame, ManifestFrameV1},
        FrameGenerator,
    },
    rt::new_async_thread,
    tokio::{
        select,
        sync::{
            broadcast::Sender as BcastSender,
            mpsc::{channel, Sender},
        },
        task::spawn,
    },
    types::{Ident32, InMemoryEnvelope, LetterheadV1, Neighbour, Recipient, SequenceIdV1},
    NonfatalError, RatmanError, Result,
};
use std::sync::Arc;
use tripwire::Tripwire;

pub struct SenderSystem {
    pub tx_1k: Sender<(ReadCapability, LetterheadV1)>,
    pub tx_32k: Sender<(ReadCapability, LetterheadV1)>,
}

/// Start a new async sender system based on an existing reader stream
///
/// A sender system consists of the following tasks:
///
/// 1) An async reader for the TcpStream/ input stream
///
/// 2) A chunk iterator that is continuously updated with chunks read
///    from the async reader task
///
/// 3) Chunk iterator is handed to the block slicer, which produces a
///    set of output blocks.
///
/// 4) Output blocks are streamed to a frame generator task
///
/// 5) Generated frames and block are cached in the respective journal
///    pages
///
/// 6) Resolve the target route and setup an auto-resolver to update
///    the route state every N? seconds.  This allows broken links to be
///    respected and reduces the amount of required resends.
///
/// 7) Send the batch of frames via the required interface
///
/// Each task can depend on other tasks, but MUST yield when not able
/// to continue.  This way data is streamed into the system, handled
/// into blocks, sliced into frames, cached, (later: stored to disk),
/// then resolved and dispatched.
///
/// A single thread is allocated for a send over the size of 512MB.  A
/// shared thread is used for all messages below that size.
///
/// The reader will keep reading until the upper message size from the
/// letter manifest is reached.  After this the stream will be
/// forcably terminated, and created blocks and frames in any of the
/// created sequences MUST be marked with `incomplete=?` in the
/// journal tagging system
// note(refactor): this function does too many things and is too
// deeply nested.  Ideally the different branches (mainly: send to
// local and send to remote) are broken out into separate functions,
// skipping the need for the dispatch functions to know about the
// collector at all!  Ideally there's 1-2 async functions that handle
// the spawn and loop setups, and then small handler functions which
// can return Result<_> which are handled in a central place for the
// sender stream.
pub(crate) async fn exec_sender_system<const L: usize>(
    journal: &Arc<Journal>,
    routes: &Arc<RouteTable>,
    drivers: &Arc<LinksMap>,
    collector: &Arc<BlockCollector>,
    block_bcast: BcastSender<BlockNotifier>,
    ingress_tx: Sender<MessageNotifier>,
    tripwire: Tripwire,
) -> Sender<(ReadCapability, LetterheadV1)> {
    let (tx_l, mut rx_l) = channel(32);
    {
        let journal = Arc::clone(journal);
        let routes = Arc::clone(routes);
        let drivers = Arc::clone(drivers);
        let collector = Arc::clone(&collector);
        new_async_thread(
            format!("sender-system-{}k", L / 1024),
            1024 * 8,
            async move {
                loop {
                    let routes = Arc::clone(&routes);
                    let drivers = Arc::clone(&drivers);
                    let collector = Arc::clone(&collector);
                    let block_bcast = block_bcast.clone();
                    let ingress_tx = ingress_tx.clone();

                    debug!("Setup sender system for {}kB blocks", L / 1024);
                    let tw = tripwire.clone();
                    let (read_cap, letterhead): (ReadCapability, LetterheadV1) = select! {
                        _ = tw => break,
                        i = rx_l.recv() => {
                            match i {
                                Some(i) => i,
                                None => break,
                            }
                        }
                    };

                    trace!(
                        "Got block to handle, (to: {}, len: {})",
                        letterhead.to.inner_address().pretty_string(),
                        letterhead.stream_size
                    );
                    let (local_tx, mut local_rx) = channel::<(Block<L>, LetterheadV1)>(4);
                    let manifest = ManifestFrameV1::from((read_cap.clone(), letterhead.clone()));
                    let journal = Arc::clone(&journal);
                    spawn(BlockWorker { read_cap }.traverse_block_tree::<L>(
                        Arc::clone(&journal),
                        letterhead.clone(),
                        local_tx,
                    ));

                    spawn(async move {
                        let routes = Arc::clone(&routes);
                        let drivers = Arc::clone(&drivers);
                        let collector = Arc::clone(&collector);
                        let block_bcast = block_bcast.clone();
                        let manifest = manifest.clone();
                        let ingress_tx = ingress_tx.clone();

                        while let Some((block, letterhead)) = local_rx.recv().await {
                            let bid = block.reference();

                            let frame_buf = match BlockSlicer
                                .produce_frames(block, letterhead.from, letterhead.to)
                                .await
                            {
                                Ok(buf) => buf,
                                Err(e) => {
                                    error!("failed to slice block to frames: {e}");
                                    continue;
                                }
                            };

                            let frame_count = frame_buf.get(0).unwrap().buffer.len();

                            let routes = Arc::clone(&routes);
                            let drivers = Arc::clone(&drivers);
                            let collector = Arc::clone(&collector);
                            let block_bcast = block_bcast.clone();

                            let bid32 = Ident32::from_bytes(bid.as_slice()).pretty_string();

                            trace!(
                                "Block {} turned into {}x {:.1}kB frames",
                                bid32,
                                frame_buf.len(),
                                frame_count as f32 / 1024.0,
                            );
                            {
                                let routes = Arc::clone(&routes);
                                let drivers = Arc::clone(&drivers);
                                let collector = Arc::clone(&collector);
                                let block_bcast = block_bcast.clone();

                                let routes = Arc::clone(&routes);
                                let drivers = Arc::clone(&drivers);
                                let collector = Arc::clone(&collector);
                                let block_bcast = (&block_bcast).clone();
                                for envelope in frame_buf {
                                    trace!(
                                        "Dispatch {} byte frame {}/{}",
                                        envelope.buffer.len(),
                                        bid32,
                                        envelope.header.get_seq_id().unwrap().num
                                    );

                                    if let Err(e) = dispatch_frame(
                                        &routes,
                                        &drivers,
                                        &collector,
                                        block_bcast.clone(),
                                        envelope,
                                    )
                                    .await
                                    {
                                        error!("failed to dispatch frame: {e}");
                                    }
                                }
                            }
                        }

                        //// ENCODE MANIFEST

                        let mut payload_buf = vec![];
                        ManifestFrame::V1(manifest.clone())
                            .generate(&mut payload_buf)
                            .unwrap();

                        let header = CarrierFrameHeader::new_blockmanifest_frame(
                            letterhead.from,
                            letterhead.to,
                            SequenceIdV1 {
                                hash: Ident32::from_bytes(read_cap.root_reference.as_slice()),
                                num: 0,
                                max: 1,
                            },
                            payload_buf.len() as u16,
                        );

                        let mut full_buf = vec![];
                        header.clone().generate(&mut full_buf).unwrap();

                        full_buf.append(&mut payload_buf);
                        let envelope = InMemoryEnvelope {
                            header,
                            buffer: full_buf,
                        };

                        if let Ok(true) = routes.is_local(letterhead.to.inner_address()).await {
                            journal.queue_manifest(envelope.clone()).await.unwrap();
                            if let Err(e) = ingress_tx
                                .send(MessageNotifier(envelope.header.get_seq_id().unwrap().hash))
                                .await
                            {
                                warn!("failed to notify local task of manifest: {e}");
                            }
                        } else {
                            let block_bcast = block_bcast.clone();

                            if let Err(_e) = dispatch_frame(
                                &Arc::clone(&routes),
                                &Arc::clone(&drivers),
                                &Arc::clone(&collector),
                                block_bcast,
                                envelope,
                            )
                            .await
                            {
                                warn!("failed to dispatch manifest; stream may arrive but is unreadable");
                            }
                        }
                    });
                }

                //
                Ok(())
            },
        );
    }

    tx_l
}

/// Resolve the target address and dispatch the frame
///
/// Returns an error if resolving or sending failed
pub(crate) async fn dispatch_frame(
    routes: &Arc<RouteTable>,
    drivers: &Arc<LinksMap>,
    collector: &Arc<BlockCollector>,
    block_bcast: BcastSender<BlockNotifier>,
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

    if let Ok(true) = routes.is_local(target_address).await {
        trace!(
            "Frame addressed to local ({:?}) queue in collector!",
            envelope.header
        );
        collector.queue_and_spawn(envelope, block_bcast).await?;
        return Ok(());
    }

    let EpNeighbourPair(epid, nb) = match routes.resolve(target_address).await {
        // Return the endpoint/target ID pair from the resolver
        Some(resolve) => resolve,
        None => {
            debug!(
                "{}: failed to resolve address {}",
                "[SOFT FAIL]".custom_color(crate::util::SOFT_WARN_COLOR),
                target_address.pretty_string()
            );
            return Err(RatmanError::Nonfatal(NonfatalError::UnknownAddress(
                target_address,
            )));
        }
    };

    let (_, ep) = drivers.get(epid as usize).await;
    ep.send(envelope, Neighbour::Single(nb), None).await
}

// todo: implement the exception mechanism
pub(crate) async fn flood_frame(
    _routes: &Arc<RouteTable>,
    drivers: &Arc<LinksMap>,
    envelope: InMemoryEnvelope,
    except: Option<Ident32>,
) -> Result<()> {
    let eepies = drivers.get_all().await;
    trace!("Flood frame on {} interfaces", eepies.len());

    // Loop over every driver and send a version of the envelope to it
    for (ep_name, ep) in eepies.into_iter() {
        let env = envelope.clone();
        if let Err(e) = ep.send(env, Neighbour::Flood, except).await {
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
