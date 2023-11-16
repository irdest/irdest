// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::storage::block::StorageBlock;
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

/// A wrapper around HashMap to enable the async-eris trait to work
#[derive(Default)]
struct JournalBlockStore(HashMap<Id, StorageBlock>);

#[async_trait]
impl<const BS: usize> BlockStorage<BS> for JournalBlockStore {
    async fn store(&mut self, block: &Block<BS>) -> std::io::Result<()> {
        self.0.insert(
            Id::from_bytes(block.reference().as_slice()),
            match BS {
                1024 => StorageBlock::_1K(Block::copy_from_vec(block.clone().to_vec())),
                32768 => StorageBlock::_32K(Block::copy_from_vec(block.clone().to_vec())),
                _ => unreachable!(),
            },
        );

        Ok(())
    }

    async fn fetch(&self, reference: &BlockReference) -> std::io::Result<Option<Block<BS>>> {
        Ok(self
            .0
            .get(&Id::from_bytes(reference.as_slice()))
            .map(|x| match x {
                StorageBlock::_1K(block) => Block::copy_from_vec(block.clone().to_vec()),
                StorageBlock::_32K(block) => Block::copy_from_vec(block.clone().to_vec()),
            }))
    }
}

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

/// Combined frame and block journal
///
/// This component has three parts to it
///
/// ## Frame Journal
///
/// Frames that can't be delivered, either because the local address
/// is offline, or because the remote address isn't reachable via the
/// currently available connections are given to the frame journal.
///
/// When an address comes online, the contents of this journal (for
/// that particular address) are then either given to the dispatcher,
/// or the collector.
///
///
/// ## Block Journal
///
/// The collector assembles frames into completed blocks that are
/// inserted into the block journal.  It is shared amongst all
/// addresses, meaning that if two users/ applications on the same
/// machine received the same message twice (for example via a flood
/// namespace), it is only kept in storage once.
///
/// When a manifest is received an assembler task is spawned which
/// checks the block journal for the required block hashes, then
/// assembles a complete message stream and hands it to the client API
/// handler.
///
/// ## Known frames page
///
/// To avoid endless replication of messages the journal keeps track
/// of frame IDs that it has seen before, even when the contents
/// aren't being saved.  This is an important mechanism in the case of
/// announcements, which will otherwise keep echoing through the
/// network forever... *makes haunting noises*.
pub(crate) struct Journal {
    /// Keeps track of known frames to do reflood
    known: RwLock<HashSet<Id>>,
    /// In-memory block queue
    ///
    /// These are blocks that were fullly re-assembled, but either are
    /// addressed to a flood namespace and haven't been marked as
    /// "seen", or are directly addressed to a recipient which is
    /// offline.
    blocks: RwLock<JournalBlockStore>,
    /// In-memory frame queue
    ///
    /// This queue is used for individual frames which could not be
    /// forwarded to a valid recipient, but which could also not be
    /// decoded into a valid block.
    frames: RwLock<HashMap<Id, InMemoryEnvelope>>,
    /// A notifier channel which can be awaited for new blocks
    ///
    /// The channel only contains the block ID, which can then be
    /// received from the main block queue
    m_notify_send: Sender<MessageNotifier>,
    /// A channel which can be polled to wait for new incoming messages
    incoming: (Sender<Letterhead>, Receiver<Letterhead>),
}

impl Journal {
    pub(crate) fn new() -> (Arc<Self>, Receiver<MessageNotifier>) {
        let (m_notify_send, m_notify_recv) = channel(8);
        (
            Arc::new(Self {
                known: Default::default(),
                blocks: Default::default(),
                frames: Default::default(),
                m_notify_send,
                incoming: channel(8),
            }),
            m_notify_recv,
        )
    }

    /// Run an async task that accepts completed blocks from the
    /// collector, checks their integrity and then inserts them into
    /// the block journal.
    pub(crate) async fn run_block_acceptor(self: Arc<Self>, mut block_output: JournalReceiver) {
        while let Some((block_buf, sequence_id)) = block_output.recv().await {
            debug!(
                "Attempting to re-construct block({} bytes)for sequence {}",
                block_buf.len(),
                sequence_id.hash
            );

            let eris_block = match StorageBlock::reconstruct(block_buf) {
                Ok(block) => block,
                Err(e) => {
                    error!(
                        "Block collected from id {} failed because {}",
                        sequence_id.hash, e
                    );

                    continue;
                }
            };

            // Verify the block hash
            let block_ref = eris_block.reference();
            if block_ref.as_slice() != sequence_id.hash.as_bytes() {
                error!(
                    "Block collected from id {} resulted in invalid block reference: {}",
                    sequence_id.hash, block_ref,
                );
                continue;
            }

            self.blocks
                .write()
                .await
                .0
                .insert(Id::from_bytes(block_ref.as_slice()), eris_block);
            trace!("Block insertion for sequence {} complete", sequence_id.hash);
        }
    }

    /// Add a new frame to the known set
    pub(crate) async fn frame_queue(&self, env: InMemoryEnvelope) {
        let seq_id = env.header.get_seq_id().unwrap().hash;
        self.frames.write().await.insert(seq_id, env);
        debug!("Frame {} successfully stored in journal!", seq_id);
    }

    /// Provide a block manifest and collect a full message
    pub(crate) async fn queue_manifest(&self, envelope: InMemoryEnvelope) {
        let seq_id = envelope.header.get_seq_id().unwrap().hash;
        let m_notify_send = self.m_notify_send.clone();

        spawn_local(async move {
            let read_cap = match ManifestFrame::parse(envelope.get_payload_slice()) {
                Ok((
                    _,
                    Ok(ManifestFrame::V1(ManifestFrameV1 {
                        block_size,
                        block_level,
                        root_reference,
                        root_key,
                    })),
                )) => ReadCapability {
                    root_reference: BlockReference::from(root_reference.slice()),
                    root_key: BlockKey::from(root_key.slice()),
                    block_size: match block_size {
                        1 => 1024,
                        32 => 32 * 1024,
                        bs => {
                            error!("Unsupported block size: {}!", bs);
                            return;
                        }
                    },
                    level: block_level,
                },
                Ok((_, Err(e))) => {
                    error!("Invalid manifest for message '{}': {:?}", seq_id, e);
                    return;
                }
                Err(e) => {
                    error!("Failed to parse manifest for message '{}': {:?}", seq_id, e);
                    return;
                }
            };

            // Send the ReadCapability through the notifier channel
            m_notify_send
                .send(MessageNotifier {
                    read_cap,
                    header: envelope.header,
                })
                .await;
        });
    }

    /// Save a carrier frame ID in the known frames journal page
    pub(crate) async fn save_as_known(&self, fid: &Id) {
        self.known.write().await.insert(fid.clone());
    }

    /// Checks if a frame ID has not been seen before
    pub(crate) async fn is_unknown(&self, fid: &Id) -> bool {
        !self.known.read().await.contains(fid)
    }
}

/// Decode a single message from a manifest/ read_capability
///
/// Spawn a task that calls this function again if it failed.
async fn decode_message(
    blocks: &RwLock<JournalBlockStore>,
    MessageNotifier {
        ref read_cap,
        ref header,
    }: &MessageNotifier,
) -> Result<Letterhead> {
    let block_reader = blocks.read().await;

    let mut payload_buffer = vec![];
    async_eris::decode(&mut payload_buffer, read_cap, &*block_reader)
        .await
        .map_err(|e| RatmanError::Block(BlockError::from(e)))?;

    // Release the block lock as quickly as possible
    drop(block_reader);

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
    mut notifier: Receiver<MessageNotifier>,
    sender: Sender<Letterhead>,
) {
    while let Some(message_notifier) = notifier.recv().await {
        let message_id = message_notifier.header.get_seq_id().unwrap().hash;
        let sender = sender.clone();

        match decode_message(&journal.blocks, &message_notifier).await {
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
                        match decode_message(&journal.blocks, &message_notifier).await {
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
