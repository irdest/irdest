// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::storage::block::StorageBlock;
use async_eris::{Block, BlockKey, BlockReference, BlockStorage, ReadCapability};
use async_std::{
    channel::{bounded, unbounded, Receiver, Sender},
    sync::{Arc, RwLock},
};
use async_trait::async_trait;
use libratman::{
    netmod::InMemoryEnvelope,
    types::{
        frames::{CarrierFrameHeader, FrameParser, ManifestFrame, ManifestFrameV1},
        ApiRecipient, Id, Message, Recipient, SequenceIdV1, TimePair,
    },
};
use std::collections::{HashMap, HashSet};

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

struct MessageNotifier {
    read_cap: ReadCapability,
    header: CarrierFrameHeader,
}

/// Remote frame journal
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
    manifest_notifier: (Sender<MessageNotifier>, Receiver<MessageNotifier>),
}

impl Journal {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            known: Default::default(),
            blocks: Default::default(),
            frames: Default::default(),
            manifest_notifier: bounded(8),
        })
    }

    /// Dispatches a long-running task to run the journal logic
    pub(crate) async fn run(self: Arc<Self>, block_output: JournalReceiver) {
        while let Ok((block_buf, sequence_id)) = block_output.recv().await {
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
        }
    }

    /// Block and yield the next completed message from the queue
    pub(crate) async fn next_message(&self) -> Option<Message> {
        match self.manifest_notifier.1.recv().await {
            Ok(MessageNotifier { read_cap, header }) => {
                let mut decoded = vec![];
                let block_store = self.blocks.read().await;
                if let Err(e) = async_eris::decode(&mut decoded, &read_cap, &*block_store).await {
                    error!("failed to decode ERIS block stream: {}", e);
                }
                drop(block_store);

                let mut time = TimePair::sending();
                time.receive();

                Some(Message {
                    id: header.get_seq_id().unwrap().hash,
                    sender: header.get_sender(),
                    recipient: match header.get_recipient() {
                        Some(Recipient::Target(trgt)) => ApiRecipient::Standard(vec![trgt]),
                        Some(Recipient::Flood(ns)) => ApiRecipient::Flood(ns),
                        _ => unreachable!(), // Probably reachable but I don't care right now
                    },
                    time,
                    payload: decoded,
                    signature: vec![],
                })
            }
            _ => None,
        }
    }

    /// Add a new frame to the known set
    pub(crate) async fn frame_queue(&self, env: InMemoryEnvelope) {
        let seq_id = env.header.get_seq_id().unwrap().hash;
        self.frames.write().await.insert(seq_id, env);
        debug!("Frame {} successfully storedin journal!", seq_id);
    }

    /// Provide a block manifest and collect a full message
    pub(crate) async fn queue_manifest(&self, envelope: InMemoryEnvelope) {
        let seq_id = envelope.header.get_seq_id().unwrap().hash;
        let manifest_notifier = self.manifest_notifier.0.clone();

        async_std::task::spawn(async move {
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
                    block_size: block_size as usize,
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
            manifest_notifier
                .send(MessageNotifier {
                    read_cap,
                    header: envelope.header,
                })
                .await;
        });
    }

    /// Save a InMemoryEnvelopeID in the known journal page
    #[allow(unused)]
    pub(crate) async fn save(&self, fid: &Id) {
        self.known.write().await.insert(fid.clone());
    }

    /// Checks if a frame ID has not been seen before
    pub(crate) async fn unknown(&self, fid: &Id) -> bool {
        !self.known.read().await.contains(fid)
    }
}
