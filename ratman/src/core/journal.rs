// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use async_eris::{Block, BlockSize};
use async_std::sync::{Arc, RwLock};
use libratman::types::{Frame, Id};
use std::collections::BTreeSet;

/// Remote frame journal
pub(crate) struct Journal {
    /// Keeps track of known frames to do reflood
    known: RwLock<BTreeSet<Id>>,
    /// Simple in-memory block store ???
    blocks: RwLock<BTreeMap<Id, StorageBlock>>,
}

/// Remove the types from block
enum StorageBlock {
    /// 1K block size
    _1K(Block<1024>),
    /// 32K block size
    _32K(Block<32768>),
}

impl Journal {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            known: Default::default(),
            blocks: Default::default(),
        })
    }

    /// Dispatches a long-running task to run the journal logic
    pub(crate) async fn run(self: Arc<Self>, block_output: Receiver<Vec<u8>>) {
        while let Some(block_buf) = block_output.recv().await {
            let (eris_block, block_size) = match block_buf.len() {
                1024 => (Block::<1024>::from_vec(block), BlockSize::_1K),
                32768 => (Block::<32768>::from_vec(block), BlockSize::_32K),
                length => {
                    error!(
                        "Block collected from id {} resulted in invalid block length: {}",
                        seq_id, length
                    );
                    continue;
                }
            };

            // Verify the block hash
            let block_ref = eris_block.reference();
            if block_ref != self.sequence_id {
                error!(
                    "Block collected from id {} resulted in invalid block reference: {}",
                    seq_id, eris_block.reference
                );
                continue;
            }
        }

        self.blocks.write().await.insert(
            block_ref,
            match block_size {
                BlockSize::_1K => StorageBlock::_1K(eris_block),
                BlockSize::_32K => StorageBlock::_32K(eris_block),
            },
        )
    }

    /// Add a new frame to the known set
    pub(crate) async fn queue(&self, _: Frame) {}

    /// Save a FrameID in the known journal page
    #[allow(unused)]
    pub(crate) async fn save(&self, fid: &Id) {
        self.known.write().await.insert(fid.clone());
    }

    /// Checks if a frame ID has not been seen before
    pub(crate) async fn unknown(&self, fid: &Id) -> bool {
        !self.known.read().await.contains(fid)
    }
}
