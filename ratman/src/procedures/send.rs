// SPDX-FileCopyrightText: 2019-2024 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
// SPDX-FileCopyrightText: 2022 embr <hi@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Asynchronous Ratman routing core

use super::{slicer::BlockSlicer, BlockWorker};
use crate::{
    journal::Journal,
    links::LinksMap,
    routes::{EpNeighbourPair, RouteTable},
};
use async_eris::{Block, ReadCapability};
use colored::Colorize;
use libratman::{
    rt::new_async_thread,
    tokio::{
        select,
        sync::mpsc::{channel, Sender},
        task::spawn_local,
    },
    types::{Ident32, InMemoryEnvelope, LetterheadV1, Neighbour, Recipient},
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
pub(crate) async fn exec_sender_system<const L: usize>(
    journal: &Arc<Journal>,
    routes: &Arc<RouteTable>,
    drivers: &Arc<LinksMap>,
    tripwire: Tripwire,
) -> Sender<(ReadCapability, LetterheadV1)> {
    let (tx_l, mut rx_l) = channel(32);
    {
        let journal = Arc::clone(journal);
        let routes = Arc::clone(routes);
        let drivers = Arc::clone(drivers);
        new_async_thread(
            format!("sender-system-{}k", L / 1024),
            1024 * 8,
            async move {
                loop {
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

                    debug!(
                        "Got block to handle, (to: {}, len: {})",
                        letterhead.to.inner_address().pretty_string(),
                        letterhead.stream_size
                    );
                    let (local_tx, mut local_rx) = channel::<(Block<L>, LetterheadV1)>(4);
                    spawn_local(BlockWorker { read_cap }.traverse_block_tree::<L>(
                        Arc::clone(&journal),
                        letterhead,
                        local_tx,
                    ));

                    let routes = Arc::clone(&routes);
                    let drivers = Arc::clone(&drivers);
                    spawn_local(async move {
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

                            let bid32 = Ident32::from_bytes(bid.as_slice()).pretty_string();

                            trace!("Block {} turned into {} frames", bid32, frame_buf.len());
                            let routes = Arc::clone(&routes);
                            let drivers = Arc::clone(&drivers);
                            spawn_local(async move {
                                for envelope in frame_buf {
                                    trace!(
                                        "Dispatched frame {}/{}",
                                        bid32,
                                        envelope.header.get_seq_id().unwrap().num
                                    );

                                    if let Err(e) =
                                        dispatch_frame(&routes, &drivers, envelope).await
                                    {
                                        error!("failed to dispatch frame: {e}");
                                    }
                                }
                            });
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
        None => {
            debug!(
                "{}: failed to resolve address {}",
                "[FAIL]".custom_color(crate::util::SOFT_WARN_COLOR),
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
    // Loop over every driver and send a version of the envelope to it
    for (ep_name, ep) in drivers.get_all().await.into_iter() {
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
