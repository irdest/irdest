// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{journal::Journal, storage::block::StorageBlock};
use async_eris::{Block, BlockKey, BlockReference, BlockStorage, ReadCapability};
use async_trait::async_trait;
use libratman::{
    frame::{
        carrier::{CarrierFrameHeader, ManifestFrame, ManifestFrameV1},
        FrameParser,
    },
    tokio::{
        sync::mpsc::{channel, unbounded_channel, Receiver, Sender},
        sync::RwLock,
        task::spawn_local,
        time,
    },
    types::{Id, InMemoryEnvelope, Letterhead, Recipient, SequenceIdV1, TimePair},
    BlockError, RatmanError, Result,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

pub type JournalSender = Sender<(Vec<u8>, SequenceIdV1)>;
pub type JournalReceiver = Receiver<(Vec<u8>, SequenceIdV1)>;

/// Block and yield the next completed message from the queue
pub(crate) async fn pop_from_receiver(receiver: &mut Receiver<Letterhead>) -> Option<Letterhead> {
    match receiver.recv().await {
        Some(letterhead) => {
            info!(
                "[{:?}] Received new message '(id {})' from {}!",
                letterhead.to, letterhead.stream_id, letterhead.from,
            );

            Some(letterhead)
        }
        _ => None,
    }
}

pub(crate) struct MessageNotifier {
    read_cap: ReadCapability,
    header: CarrierFrameHeader,
}

/// Decode a single message from a manifest/ read_capability
///
/// Spawn a task that calls this function again if it failed.
async fn decode_message(
    journal: &Arc<Journal>,
    MessageNotifier {
        ref read_cap,
        ref header,
    }: &MessageNotifier,
) -> Result<Letterhead> {
    let mut payload_buffer = vec![];
    async_eris::decode(&mut payload_buffer, read_cap, &journal.blocks)
        .await
        .map_err(|e| RatmanError::Block(BlockError::from(e)))?;

    // todo: this is a terrible api and it needs to change.  But
    // also this type might be completely useless??
    let mut time = TimePair::sending();
    time.receive();

    debug!("Decoding letterhead was successful!");
    Ok(Letterhead {
        from: header.get_sender(),
        to: header.get_recipient().unwrap(),
        time,
        stream_id: header.get_seq_id().unwrap().hash,
        payload_length: header.get_payload_length(),
        auxiliary_data: vec![],
    })
}

/// Run an async task that attempts to re-assemble messages from a
/// Manifest, and spawns a long-running version of itself if some
/// blocks are still missing.
pub(crate) async fn run_message_assembler(
    journal: Arc<Journal>,
    // mut notifier: Receiver<MessageNotifier>,
    sender: Sender<Letterhead>,
) {
    let (_, mut notifier) = channel::<MessageNotifier>(1); // fixme: is this even still needed???

    while let Some(message_notifier) = notifier.recv().await {
        let message_id = message_notifier.header.get_seq_id().unwrap().hash;
        let sender = sender.clone();

        match decode_message(&journal, &message_notifier).await {
            Ok(message) => {
                sender.send(message).await;
            }
            Err(_) => {
                let journal = Arc::clone(&journal);

                warn!(
                    "Can't assemble {}, blocks missing!  Trying again later...",
                    message_id
                );
                spawn_local(async move {
                    let mut ctr = 0;

                    loop {
                        let millis = (100 + (ctr * 20)).clamp(0, 32);
                        debug!(
                            "Waiting {}ms for attempt #{} to assemble message {}",
                            millis, ctr, message_id,
                        );
                        time::sleep(Duration::from_millis(millis)).await;
                        match decode_message(&journal, &message_notifier).await {
                            Ok(msg) => {
                                sender.send(msg).await;
                                break;
                            }
                            Err(e) => {
                                error!("failed to re-assemble message because of {}", e)
                            }
                        }

                        ctr += 1;
                    }
                });
            }
        }
    }
}
