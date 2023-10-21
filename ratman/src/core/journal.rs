// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use async_eris::{Block, BlockReference};
use async_std::{
    channel::{Receiver, Sender},
    sync::{Arc, RwLock},
};
use libratman::{
    netmod::InMemoryEnvelope,
    types::{Id, Message, SequenceIdV1},
};
use std::collections::{BTreeMap, BTreeSet};

use crate::storage::block::StorageBlock;

pub type JournalSender = Sender<(Vec<u8>, SequenceIdV1)>;
pub type JournalReceiver = Receiver<(Vec<u8>, SequenceIdV1)>;

/// Remote frame journal
pub(crate) struct Journal {
    /// Keeps track of known frames to do reflood
    known: RwLock<BTreeSet<Id>>,
    /// Simple in-memory block store ???
    blocks: RwLock<BTreeMap<Id, StorageBlock>>,
}

impl Journal {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            known: Default::default(),
            blocks: Default::default(),
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
                .insert(Id::from_bytes(block_ref.as_slice()), eris_block);
        }
    }

    pub(crate) async fn next(&self) -> Message {
        todo!()
    }

    /// Add a new frame to the known set
    pub(crate) async fn queue(&self, _: InMemoryEnvelope) {}

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
