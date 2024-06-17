// SPDX-FileCopyrightText: 2019-2024 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Slices `Message` into a series of Frames

use crate::context::RatmanContext;
use crate::journal::{self, Journal};
use async_eris::{Block, BlockKey, BlockReference, BlockStorage, MemoryStorage, ReadCapability};
use fjall::{Config, Keyspace};
use libratman::tokio::sync::mpsc::Sender;
use libratman::types::LetterheadV1;
use libratman::{
    frame::carrier::CarrierFrameHeader,
    types::{Address, Ident32, InMemoryEnvelope, Recipient, SequenceIdV1},
    Result,
};
use libratman::{BlockError, RatmanError};
use rand::rngs::OsRng;
use rand::{random, RngCore};
use std::collections::{HashMap, VecDeque};
use std::convert::TryInto;
use std::env::temp_dir;
use std::sync::Arc;

pub struct BlockWorker {
    pub read_cap: ReadCapability,
}

impl BlockWorker {
    /// Stream blocks from disk, slice them into sendable frames, and forward
    /// them to the sender system
    pub async fn traverse_block_tree<const L: usize>(
        self,
        journal: Arc<Journal>,
        letterhead: LetterheadV1,
        tx: Sender<(Block<L>, LetterheadV1)>,
    ) -> Result<()> {
        let mut subtrees = VecDeque::new();
        subtrees.push_back(self.read_cap);

        // Recursively go down the block stream and encode them to frames
        while let Some(ref tree) = subtrees.pop_front() {
            let fetch_ref = tree.root_reference.clone();
            let curr_block: Block<L> =
                journal
                    .blocks
                    .fetch(&fetch_ref)
                    .await?
                    .ok_or(RatmanError::Block(BlockError::Eris(
                        async_eris::Error::BlockNotFound,
                    )))?;

            let mut scan_block = curr_block.clone();
            scan_block.chacha20(&tree.root_key);

            println!(
                "Traversing '{}' on level {}",
                scan_block.reference(),
                tree.level
            );
            if tree.level > 0 {
                for rk_pair_raw in scan_block.chunks_exact(64) {
                    let has_content = rk_pair_raw.iter().any(|x| *x != 0);
                    if has_content {
                        let rk_pair = (
                            BlockReference(rk_pair_raw[..32].try_into().unwrap()),
                            BlockKey(rk_pair_raw[32..].try_into().unwrap()),
                        );

                        subtrees.push_back(ReadCapability::from_rk_pair(
                            rk_pair,
                            tree.level - 1,
                            L,
                        ));
                    }
                }
            }

            // Send a copy of the block to the sender system
            if let Err(e) = tx.send((curr_block, letterhead.clone())).await {
                error!("failed to send block {fetch_ref} to sender system: {e}");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
use libratman::tokio;

#[libratman::tokio::test]
async fn test_block_walker() {
    use std::collections::HashMap;

    let td = temp_dir();
    let j = Arc::new(
        Journal::new(Keyspace::open(Config::new(td.join("test_block_walker.jfall"))).unwrap())
            .unwrap(),
    );

    // 1 MibiBytes of delicious random data. nom! nom!
    let mut content = vec![0; 1024 * 1024 * 1];
    OsRng.fill_bytes(&mut content);

    println!("Fill data: complete");

    let letterhead = LetterheadV1 {
        from: Address::random(),
        to: Recipient::Address(Address::random()),
        stream_size: content.len() as u64,
        auxiliary_data: vec![],
    };

    let block_store = MemoryStorage::default();
    let key: [u8; 32] = random();

    // Encode the blocks to the journal so we can walk over them
    let read_cap1 = async_eris::encode(
        &mut content.as_slice(),
        &key,
        async_eris::BlockSize::_1K,
        &j.blocks,
    )
    .await
    .unwrap();

    println!("Encode blocks to disk: complete");

    // Encode to in-memory store to have something to compare with later
    let read_cap2 = async_eris::encode(
        &mut content.as_slice(),
        &key,
        async_eris::BlockSize::_1K,
        &block_store,
    )
    .await
    .unwrap();

    println!("Encode blocks to memory: complete");
    assert_eq!(read_cap1, read_cap2);
    println!("read_cap1 == read_cap2");

    let (tx, mut rx) = tokio::sync::mpsc::channel(16);
    tokio::spawn(
        BlockWorker {
            read_cap: read_cap1,
        }
        .traverse_block_tree::<1024>(Arc::clone(&j), letterhead, tx),
    );

    let mut traversed_blocks = HashMap::<BlockReference, Vec<u8>>::new();
    loop {
        let (bl, _): (Block<1024>, LetterheadV1) = match rx.recv().await {
            Some(b) => b,
            None => break,
        };

        traversed_blocks.insert(bl.reference(), bl.as_slice().to_vec());
    }

    let original_blocks = block_store.read().unwrap();

    // Fucking pray
    assert_eq!(*original_blocks, traversed_blocks);
}

pub struct BlockSlicer;

impl BlockSlicer {
    pub async fn produce_frames<const L: usize>(
        self,
        b: Block<L>,
        sender: Address,
        recipient: Recipient,
    ) -> Result<Vec<InMemoryEnvelope>> {
        let mut buf = vec![];
        let header_size = CarrierFrameHeader::get_blockdata_size(sender, recipient);
        let max_payload_size = 1100; // fixme: /o\

        let block_ref = Ident32::from_bytes(b.reference().as_slice());

        // We chunk the data block into as many pieces as are required for
        // the target MTU.  Each carrier frame gets assigned the same
        // sequence ID hash, with an incrementing numerical count.  This way
        // we can re-order frames that have arrived out of order.
        let mut ctr = 0;
        let max = 1
            + (
                // The length of the block
                b.as_slice().len()
                // divided by the MTU - what is required for the header
                / (max_payload_size as usize - header_size - 4)
            );
        for chunk in b.as_slice().chunks(max_payload_size as usize) {
            assert!(ctr as usize <= max);
            trace!(
                "Cutting block {} into {} length chunks",
                block_ref,
                chunk.len()
            );

            use std::convert::TryFrom;
            let seq_id = SequenceIdV1 {
                hash: block_ref,
                num: ctr,
                max: u8::try_from(max).expect("maximum frame number too large!"),
            };

            // Create a header and encode it into an InMemoryEnvelope
            let header = CarrierFrameHeader::new_blockdata_frame(
                sender,
                recipient,
                seq_id,
                chunk.len() as u16,
            );

            // Push the header + chunk data to the output buffer
            buf.push(InMemoryEnvelope::from_header_and_payload(
                header,
                chunk.to_vec(),
            )?);

            // Increment sequence counter
            ctr += 1;
        }

        Ok(buf)
    }
}
