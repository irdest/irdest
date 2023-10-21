//! Message dispatch module
//!
//! Accepts a stream of ERIS blocks on one end and returns a sequence
//! of correctly sliced CarrierFrame's for a particular MTU.
//!
//! Accepts a sequence of CarrierFrame's and re-assembles them back
//! into full ERIS blocks.

mod collector;
use async_eris::MemoryStorage;
pub(crate) use collector::BlockCollector;

mod slicer;
use libratman::types::{Address, Recipient};
pub(crate) use slicer::{new_carrier_v1, BlockSlicer, StreamSlicer};

/// Verify that a set of blocks can be turned into stream data
/// (CarrierFrames), and re-collected into full blocks again.
#[test]
#[allow(deprecated)]
fn block_stream_level() -> libratman::Result<()> {
    use crate::context::RatmanContext;
    use async_eris::BlockSize;
    use async_std::sync::Arc;
    use libratman::types::{ApiRecipient, Id, Message, TimePair};
    use rand::{rngs::OsRng, RngCore};

    let ctx = RatmanContext::new_in_memory();
    let this = async_std::task::block_on(ctx.keys.create_address());
    let that = async_std::task::block_on(ctx.keys.create_address());

    let mut msg = Message {
        id: Id::random(),
        sender: this,
        recipient: ApiRecipient::Standard(vec![that]),
        time: TimePair::sending(),
        // 32kb of data
        payload: vec![0; 1024 * 32],
        signature: vec![],
    };

    // Give our message some pezzaz
    OsRng {}.fill_bytes(&mut msg.payload);
    let payload_len = msg.payload.len();

    ///////////////////////////
    ////  Actual test begins

    let ctx2 = Arc::clone(&ctx);
    let (manifest, blocks) =
        async_std::task::block_on(
            async move { BlockSlicer::slice(&ctx2, msg, BlockSize::_1K).await },
        )?;

    // Turn the blocks into a set of carrier frames
    let carriers = StreamSlicer::slice(
        &ctx,
        Recipient::Target(that),
        this,
        blocks.clone().into_iter(),
    )?;

    println!(
        "{} bytes of data resulted in {} blocks of 1K, resulted in {} carrier frames",
        payload_len,
        blocks.len(),
        carriers.len(),
    );

    //  Create a new block collector just for this test!
    let (tx, rx) = async_std::channel::bounded(8);
    let mut collector = BlockCollector::new(tx);

    Ok(())
}
