// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{context::RatmanContext, journal::Journal};
use async_eris::ReadCapability;
use libratman::{
    frame::carrier::{ManifestFrame, ManifestFrameV1},
    tokio::{
        fs::File,
        io::AsyncWrite,
        select,
        sync::{
            broadcast::{Receiver as BcastReceiver, Sender as BcastSender},
            mpsc::{Receiver, Sender},
        },
        task::spawn_local,
    },
    tokio_util::compat::{Compat, TokioAsyncReadCompatExt},
    types::{Ident32, LetterheadV1},
    Result,
};
use std::sync::Arc;
use tripwire::Tripwire;

// /// Block and yield the next completed message from the queue
// pub(crate) async fn pop_from_receiver(
//
// ) -> Option<Letterhead> {
//     match receiver.recv().await {
//         Some(letterhead) => {
//             info!(
//                 "[{:?}] Received new message '(id {})' from {}!",
//                 letterhead.to, letterhead.stream_id, letterhead.from,
//             );

//             Some(letterhead)
//         }
//         _ => None,
//     }
// }

/// Notify the ingress system of a new manifest in the journal
pub(crate) struct MessageNotifier(pub Ident32);

/// Notify all stream assemblers that there's new blocks available
///
/// This is a technical limitation since we can't build an association between a
/// newly created block and a manifest that that block belongs to.  Only the
/// manifest will know whether it has all the blocks needed to reassemble a
/// message.  So on every new block we notify all assemblers currently waiting
/// for something to happen, and then fail again if the block was not part of
/// the stream they're reassembling.
#[derive(Clone)]
pub(crate) struct BlockNotifier;

/// Listen to manifest events, indicating that we should start assembling a full
/// message stream.  This spawns a task that will retry blocks that haven't made
/// it yet
pub async fn exec_ingress_system(
    ctx: Arc<RatmanContext>,
    mut rx: Receiver<MessageNotifier>,
    block_notifier: BcastSender<BlockNotifier>,
) {
    loop {
        let tripwire = ctx.tripwire.clone();
        let block_notifier = block_notifier.clone();
        select! {
            biased;
            _ = tripwire => {},
            manifest_notifier = rx.recv() => {
                if manifest_notifier.is_none() {
                    break;
                }

                let ctx = Arc::clone(&ctx);
                let tripwire = ctx.tripwire.clone();
                spawn_local(async move {
                    if let Err(e) = decode_message(ctx, manifest_notifier.unwrap(), tripwire, block_notifier.subscribe()).await {
                        error!("failed to reassemble message stream: {e}");
                        return;
                    }
                });
            }
        }
    }

    info!("Ingress system shut down");
}

/// Decode a single message from a manifest/ read_capability
///
/// Spawn a task that calls this function again if it failed.
async fn decode_message(
    ctx: Arc<RatmanContext>,
    manifest: MessageNotifier,
    tripwire: Tripwire,
    mut block_notify: BcastReceiver<BlockNotifier>,
) -> Result<LetterheadV1> {
    let manifest = ctx.journal.manifests.get(&manifest.0.to_string())?.unwrap();

    // fixme: this won't work on non-linux?
    let null_file = File::open("/dev/null").await?;
    let read_cap = match manifest.manifest.maybe_inner()? {
        ManifestFrame::V1(v1) => <ManifestFrameV1 as Into<Result<ReadCapability>>>::into(v1)?,
    };

    // check that we have all the bits we need to decode a message stream.  If
    // we do we notify the API frontend
    if async_eris::decode(&mut null_file.compat(), &read_cap, &ctx.journal.blocks)
        .await
        .is_ok()
    {
    }
    // If we weren't able to decode the full stream we wait for a block notifier
    // event and then try again.
    else {
        loop {
            let tw = tripwire.clone();
            let null_file = File::open("/dev/null").await?;
            select! {
                biased;
                _ = tw => break,
                _ = block_notify.recv() => {
                    if async_eris::decode(&mut null_file.compat(), &read_cap, &ctx.journal.blocks)
                        .await
                        .is_ok()
                    {
                        debug!("Completed new block stream!");
                        break;
                    }
                }
            }
        }
    }

    // loop {}

    // let mut payload_buffer = vec![];
    // async_eris::decode(&mut payload_buffer, read_cap, &journal.blocks)
    //     .await
    //     .map_err(|e| RatmanError::Block(BlockError::from(e)))?;

    // // todo: this is a terrible api and it needs to change.  But
    // // also this type might be completely useless??
    // let mut time = TimePair::sending();
    // time.receive();

    // debug!("Decoding letterhead was successful!");
    // Ok(Letterhead {
    //     from: header.get_sender(),
    //     to: header.get_recipient().unwrap(),
    //     time,
    //     stream_id: header.get_seq_id().unwrap().hash,
    //     payload_length: header.get_payload_length(),
    //     auxiliary_data: vec![],
    // })

    todo!()
}

// /// Run an async task that attempts to re-assemble messages from a
// /// Manifest, and spawns a long-running version of itself if some
// /// blocks are still missing.
// pub(crate) async fn run_message_assembler(
//     journal: Arc<Journal>,
//     // mut notifier: Receiver<MessageNotifier>,
//     sender: Sender<Letterhead>,
// ) {
//     let (_, mut notifier) = channel::<MessageNotifier>(1); // fixme: is this even still needed???

//     while let Some(message_notifier) = notifier.recv().await {
//         let message_id = message_notifier.header.get_seq_id().unwrap().hash;
//         let sender = sender.clone();

//         match decode_message(&journal, &message_notifier).await {
//             Ok(message) => {
//                 sender.send(message).await;
//             }
//             Err(_) => {
//                 let journal = Arc::clone(&journal);

//                 warn!(
//                     "Can't assemble {}, blocks missing!  Trying again later...",
//                     message_id
//                 );
//                 spawn_local(async move {
//                     let mut ctr = 0;

//                     loop {
//                         let millis = (100 + (ctr * 20)).clamp(0, 32);
//                         debug!(
//                             "Waiting {}ms for attempt #{} to assemble message {}",
//                             millis, ctr, message_id,
//                         );
//                         time::sleep(Duration::from_millis(millis)).await;
//                         match decode_message(&journal, &message_notifier).await {
//                             Ok(msg) => {
//                                 sender.send(msg).await;
//                                 break;
//                             }
//                             Err(e) => {
//                                 error!("failed to re-assemble message because of {}", e)
//                             }
//                         }

//                         ctr += 1;
//                     }
//                 });
//             }
//         }
//     }
// }
