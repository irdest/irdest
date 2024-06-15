// SPDX-FileCopyrightText: 2019-2024 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
// SPDX-FileCopyrightText: 2022 embr <hi@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Asynchronous Ratman routing core

use crate::{
    context::RatmanContext,
    links::LinksMap,
    routes::{EpNeighbourPair, RouteTable},
};
use async_eris::ReadCapability;
use libratman::{
    types::{InMemoryEnvelope, LetterheadV1, Neighbour, Recipient},
    RatmanError, Result,
};
use std::sync::Arc;

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
    context: &Arc<RatmanContext>,
    read_cap: ReadCapability,
    letterhead: LetterheadV1,
) -> Result<()> {
    // let (tx, rx) = mpsc::channel(size_commonbuf_t::<L>());
    // let socket = RawSocketHandle::new(reader);

    // Setup the block slicer
    // let (_iter_tx, chunk_iter) = ChunkIter::<L>::new();
    // let read_cap_f = spawn_local(block_slicer_task(journal, chunk_iter));

    // Read from the socket until we have reached the upper message limit
    // while socket.read_counter() < total_message_size {
    // We read a chunk from disk and handle content encryption
    // first, then write out the encrypted chunk and resulting
    // nonce into the outer block to handle.
    // let (encrypted_chunk, chunk_nonce) = {
    //     let mut raw_data = socket.read_chunk::<L>().await?;
    //     if raw_data.1 < L {
    //         debug!("Reached the last chunk in the data stream");
    //     }

    // // Encrypt the data before doing anything else!
    // let shared_key = context.keys.diffie_hellman(from, to).await.expect(&format!(
    //     "Diffie-Hellman key-exchange failed between {} and {}",
    //     from, to,
    // ));
    // let nonce = crypto::encrypt_chunk(&shared_key, &mut raw_data.0);
    // (raw_data, nonce) // data no longer raw!
    // };

    // }

    // let _read_cap = read_cap_f
    //     .await
    //     .expect("failed to produce message manifest")?;

    Ok(())
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
        None => return Err(RatmanError::NoSuchAddress(target_address)),
    };

    let (_, ep) = drivers.get(epid as usize).await;
    ep.send(envelope, Neighbour::Single(nb), None).await
}

// todo: implement the exception mechanism
pub(crate) async fn flood_frame(
    _routes: &Arc<RouteTable>,
    drivers: &Arc<LinksMap>,
    envelope: InMemoryEnvelope,
    _except: Option<(String, Neighbour)>,
) -> Result<()> {
    // Loop over every driver and send a version of the envelope to it
    for (ep_name, ep) in drivers.get_all().await.into_iter() {
        let env = envelope.clone();
        if let Err(e) = ep.send(env, Neighbour::Flood, None).await {
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
