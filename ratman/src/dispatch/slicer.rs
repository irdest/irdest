// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Slices `Message` into a series of Frames

use std::time::Duration;

use crate::context::RatmanContext;
use async_eris::{BlockReference, BlockSize, MemoryStorage, ReadCapability};
use libratman::{
    frame::carrier::{modes, CarrierFrameHeader},
    futures::AsyncRead,
    types::{Address, ClientAuth, Id, InMemoryEnvelope, Letterhead, Recipient, SequenceIdV1},
    Result,
};
use std::sync::Arc;

pub struct StreamSlicer;

impl StreamSlicer {
    /// Take a stream of ERIS blocks and slice them into
    pub fn slice<I: Iterator<Item = (BlockReference, Vec<u8>)>>(
        ctx: &Arc<RatmanContext>,
        recipient: Recipient,
        sender: Address,
        input: I,
    ) -> Result<Vec<InMemoryEnvelope>> {
        let mut buf = vec![];
        let header_size = CarrierFrameHeader::get_blockdata_size(sender, recipient);

        // Iterate over all available blocks and their hash
        // references.  The hash reference is used as the first part
        // of the SequenceId to make re-association on the other side
        // possible.
        for (block_ref, block_data) in input {
            let max_payload_size = 999; // fixme /o\
            let max_in_sequence = block_data.as_slice().len() / max_payload_size as usize;
            let block_ref = Id::from_bytes(block_ref.as_slice());

            // We chunk the data block into as many pieces as are
            // required for the current MTU.  Each carrier frame gets
            // assigned the same sequence ID hash, with an
            // incrementing numerical count.  This way we can re-order
            // frames that have arrived out of order.
            let mut ctr = 0;
            let max = 1 + (block_data.as_slice().len() / max_payload_size as usize);
            for chunk in block_data.as_slice().chunks(max_payload_size as usize) {
                assert!(ctr as usize <= max);
                trace!(
                    "Cutting block {} into {} length chunk",
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
                buf.push(InMemoryEnvelope::from_header_and_payload(
                    header,
                    chunk.to_vec(),
                )?);

                // Increment sequence counter
                ctr += 1;
            }
        }

        // Finally, simply return the output collection
        Ok(buf)
    }
}

/// A simple slicer for ERIS blocks
pub(crate) struct BlockSlicer;

impl BlockSlicer {
    pub(crate) async fn slice(
        ctx: &Arc<RatmanContext>,
        // auth: ClientAuth,
        (lhead, reader): (Letterhead, &mut (impl AsyncRead + Unpin)),
        block_size: BlockSize,
    ) -> Result<(ReadCapability, MemoryStorage)> {
        let mut blocks = MemoryStorage::new();
        // ctx.meta_db
        //     .start_stream(lhead.from, lhead.to.inner_address(), auth)?;

        // let key = ctx
        //     .keys
        //     .diffie_hellman(lhead.from, lhead.to.inner_address())
        //     .await
        //     .expect("failed to perform diffie-hellman");
        // let key2 = key.to_bytes();
        // let read_cap = async_eris::encode(reader, &key2, block_size, &mut blocks)
        //     .await
        //     .unwrap();
        // Ok((read_cap, blocks))

        todo!()
    }
}
