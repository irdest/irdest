//! Message dispatch module
//!
//! Accepts a stream of ERIS blocks on one end and returns a sequence
//! of correctly sliced CarrierFrame's for a particular MTU.
//!
//! Accepts a sequence of CarrierFrame's and re-assembles them back
//! into full ERIS blocks.

mod collector;
mod slicer;

use crate::{
    config::ConfigTree, context::RatmanContext, crypto, journal::Journal,
    storage::block::StorageBlock,
};
use async_eris::{BlockReference, MemoryStorage, ReadCapability};
use curve25519_dalek::traits::VartimePrecomputedMultiscalarMul;
use libratman::{
    api::socket_v2::RawSocketHandle,
    chunk::ChunkIter,
    frame::micro::MicroframeHeader,
    rt::size_commonbuf_t,
    tokio::{net::TcpStream, sync::mpsc, task::spawn_local},
    tokio_util::compat::{Compat, TokioAsyncReadCompatExt},
    types::{Address, Recipient},
    Result,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub(crate) use collector::BlockCollector;
pub(crate) use slicer::{BlockSlicer, StreamSlicer};

/// A high-level message manifest which is used to encode information
/// about where and how a message should be sent
pub struct LetterManifest {
    pub from: Address,
    pub to: Address,
    pub selected_block_size: usize,
    pub total_message_size: usize,
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
    context: &Arc<RatmanContext>,
    reader: TcpStream,
    // todo: replace with sled integration
    journal: Journal,
    LetterManifest {
        from,
        to,
        selected_block_size,
        total_message_size,
    }: LetterManifest,
) -> Result<()> {
    // let (tx, rx) = mpsc::channel(size_commonbuf_t::<L>());
    let mut socket = RawSocketHandle::new(reader);

    // Setup the block slicer
    let (iter_tx, chunk_iter) = ChunkIter::<L>::new();
    let read_cap_f = spawn_local(block_slicer_task(journal, chunk_iter));

    // Read from the socket until we have reached the upper message
    // limit
    while socket.read_counter() < total_message_size {
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
        todo!()
    }

    let read_cap = read_cap_f
        .await
        .expect("failed to produce message manifest")?;

    Ok(())
}

/// Setup the task reading from the chunk iter and producing ERIS
/// blocks from the content
async fn block_slicer_task<const L: usize>(
    mut journal: Journal,
    mut iter: ChunkIter<L>,
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
