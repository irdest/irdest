// SPDX-FileCopyrightText: 2019-2024 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Slices `Message` into a series of Frames

use crate::context::RatmanContext;
use async_eris::{Block, BlockReference, BlockStorage, ReadCapability};
use libratman::tokio::sync::mpsc::Sender;
use libratman::{
    frame::carrier::CarrierFrameHeader,
    types::{Address, Ident32, InMemoryEnvelope, Recipient, SequenceIdV1},
    Result,
};
use libratman::{BlockError, RatmanError};
use std::collections::VecDeque;
use std::sync::Arc;

pub struct BlockWorker {
    pub read_cap: ReadCapability,
}

impl BlockWorker {
    // /// Take a stream of ERIS blocks and slice them into sendable frames
    // pub fn slice<I: Iterator<Item = (BlockReference, Vec<u8>)>>(
    //     _ctx: &Arc<RatmanContext>,
    //     recipient: Recipient,
    //     sender: Address,
    //     input: I,
    // ) -> Result<Vec<InMemoryEnvelope>> {
    //     let mut buf = vec![];
    //     let _header_size = CarrierFrameHeader::get_blockdata_size(sender, recipient);

    //     // Iterate over all available blocks and their hash references.  The
    //     // hash reference is used as the first part of the SequenceId to make
    //     // re-association on the other side possible.
    //     for (block_ref, block_data) in input {
    //         let max_payload_size = 1100; // fixme /o\
    //         let _max_in_sequence = block_data.as_slice().len() / max_payload_size as usize;
    //         let block_ref = Ident32::from_bytes(block_ref.as_slice());

    //         // We chunk the data block into as many pieces as are required for
    //         // the target MTU.  Each carrier frame gets assigned the same
    //         // sequence ID hash, with an incrementing numerical count.  This way
    //         // we can re-order frames that have arrived out of order.
    //         let mut ctr = 0;
    //         let max = 1 + (block_data.as_slice().len() / max_payload_size as usize);
    //         for chunk in block_data.as_slice().chunks(max_payload_size as usize) {
    //             assert!(ctr as usize <= max);
    //             trace!(
    //                 "Cutting block {} into {} length chunks",
    //                 block_ref,
    //                 chunk.len()
    //             );

    //             use std::convert::TryFrom;
    //             let seq_id = SequenceIdV1 {
    //                 hash: block_ref,
    //                 num: ctr,
    //                 max: u8::try_from(max).expect("maximum frame number too large!"),
    //             };

    //             // Create a header and encode it into an InMemoryEnvelope
    //             let header = CarrierFrameHeader::new_blockdata_frame(
    //                 sender,
    //                 recipient,
    //                 seq_id,
    //                 chunk.len() as u16,
    //             );
    //             buf.push(InMemoryEnvelope::from_header_and_payload(
    //                 header,
    //                 chunk.to_vec(),
    //             )?);

    //             // Increment sequence counter
    //             ctr += 1;
    //         }
    //     }

    //     // Finally, simply return the output collection
    //     Ok(buf)
    // }

    /// Stream blocks from disk, slice them into sendable frames, and forward
    /// them to the sender system
    pub async fn traverse_block_tree<const L: usize>(
        mut self,
        ctx: &Arc<RatmanContext>,
        tx: Sender<Block<L>>,
    ) -> Result<()> {
        let mut fetch_ref = self.read_cap.root_reference;
        let mut subtrees = VecDeque::new();
        subtrees.push_back(self.read_cap);

        // Utility for fetching a block ref
        let fetch_block = |fref| ctx.journal.blocks.fetch(fref);

        // Recursively go down the block stream and encode them to frames
        while let Some(tree) = subtrees.pop_front() {
            let curr_block: Block<L> =
                fetch_block(&fetch_ref)
                    .await?
                    .ok_or(RatmanError::Block(BlockError::Eris(
                        async_eris::Error::BlockNotFound,
                    )))?;

            // Send a copy of the block to the sender system
            if let Err(e) = tx.send(curr_block).await {
                error!("failed to send block {fetch_ref} to sender system: {e}");
            }
        }

        // loop {

        // }

        Ok(())
    }
}
