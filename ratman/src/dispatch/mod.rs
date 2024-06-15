// SPDX-FileCopyrightText: 2023-2024 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Message dispatch module
//!
//! Accepts a stream of ERIS blocks on one end and returns a sequence
//! of correctly sliced CarrierFrame's for a particular MTU.
//!
//! Accepts a sequence of CarrierFrame's and re-assembles them back
//! into full ERIS blocks.

mod send;
mod slicer;

use crate::{context::RatmanContext, journal::Journal};
use async_eris::ReadCapability;
use libratman::{
    chunk::ChunkIter,
    tokio_util::compat::{Compat, TokioAsyncReadCompatExt},
    types::LetterheadV1,
    Result,
};
use std::sync::Arc;

pub(crate) use send::{dispatch_frame, flood_frame};
pub(crate) use slicer::BlockWorker;

/// Setup the task reading from the chunk iter and producing ERIS
/// blocks from the content
async fn block_slicer_task<const L: usize>(
    mut journal: Journal,
    iter: ChunkIter<L>,
) -> Result<ReadCapability> {
    debug!("Starting block slicer on ChunkIter<{}>", L);
    async_eris::encode_const::<_, Compat<ChunkIter<L>>, L>(
        &mut (iter.compat()),
        &[0; 32],
        &mut journal.blocks,
    )
    .await
    .map_err(Into::into)
}

// /// Verify that a set of blocks can be turned into stream data
// /// (CarrierFrames), and re-collected into full blocks again.
// #[test]
// #[allow(deprecated)]
// pub fn block_stream_level() -> libratman::Result<()> {
//     use crate::context::RatmanContext;
//     use async_eris::BlockSize;
//     use tokio-to-be::sync::Arc;
//     use libratman::types::{ApiRecipient, Id, Message, TimePair};
//     use rand::{rngs::OsRng, RngCore};

//     // Very verbose logging environment
//     crate::util::setup_test_logging();

//     let ctx = RatmanContext::new_in_memory(ConfigTree::default_in_memory());
//     let this = tokio-to-be::task::block_on(ctx.keys.create_address());
//     let that = tokio-to-be::task::block_on(ctx.keys.create_address());

//     let mut msg = Message {
//         id: Id::random(),
//         sender: this,
//         recipient: ApiRecipient::Standard(vec![that]),
//         time: TimePair::sending(),
//         // 32kb of data
//         payload: vec![0; 1024 * 2],
//         signature: vec![],
//     };

//     // Give our message some pezzaz
//     OsRng {}.fill_bytes(&mut msg.payload);
//     let payload_len = msg.payload.len();

//     ///////////////////////////
//     ////  Actual test begins

//     let ctx2 = Arc::clone(&ctx);
//     let (manifest, mut blocks) = tokio-to-be::task::block_on(async move {
//         BlockSlicer::slice(&ctx2, &mut msg, BlockSize::_1K).await
//     })?;

//     for (block_ref, block_buf) in &blocks {
//         info!("block_ref: {} block_length: {}", block_ref, block_buf.len());
//     }

//     // Turn the blocks into a set of carrier frames
//     let carriers = StreamSlicer::slice(
//         &ctx,
//         Recipient::Target(that),
//         this,
//         blocks.clone().into_iter(),
//     )?;

//     info!(
//         "{} bytes of data resulted in {} blocks of 1K, resulted in {} carrier frames for MTU {}",
//         payload_len,
//         blocks.len(),
//         carriers.len(),
//         ctx.core.get_route_mtu(None),
//     );

//     //  Create a new block collector just for this test!
//     let (tx, rx) = tokio-to-be::channel::bounded(8);
//     let mut collector = BlockCollector::new(tx);

//     for envelope in carriers {
//         block_on(collector.queue_and_spawn(envelope))?;
//     }

//     info!("Queued all blocks...");
//     // collector.shutdown();

//     while let Ok((block, seq_id)) = block_on(rx.recv()) {
//         assert_eq!(block.len(), 1024);

//         info!("Re-collected another block: {}", seq_id.hash);
//         let previous_block = blocks
//             .remove(&BlockReference::from(seq_id.hash.slice()))
//             .unwrap();
//         assert_eq!(previous_block, block);

//         if blocks.len() == 0 {
//             break;
//         }
//     }

//     Ok(())
// }
