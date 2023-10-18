// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Slices `Message` into a series of Frames

#![allow(unused)]

use crate::context::RatmanContext;
use async_eris::{BlockReference, BlockSize, MemoryStorage, ReadCapability};
use async_std::sync::Arc;
use libratman::types::{
    frames::{modes, CarrierFrame, CarrierFrameV1, SequenceIdV1},
    Address, Id, Message, Recipient, Result, SeqBuilder,
};

pub struct StreamSlicer;

pub(crate) fn new_carrier_v1(
    recipient: Option<Recipient>,
    sender: Address,
    seq_id: Option<SequenceIdV1>,
) -> CarrierFrameV1 {
    CarrierFrameV1::pre_alloc(modes::DATA, recipient, sender, seq_id, None)
}

impl StreamSlicer {
    /// Take a stream of ERIS blocks and slice them into
    // TODO: should this function take a Block<BS>, which would
    // enforce a block size at the type level
    //
    // TODO: update this function to be a stream
    pub fn slice<I: Iterator<Item = (BlockReference, Vec<u8>)>>(
        ctx: Arc<RatmanContext>,
        recipient: Recipient,
        sender: Address,
        input: I,
    ) -> Result<Vec<CarrierFrame>> {
        let mut buf = vec![];
        let schema_frame = new_carrier_v1(
            Some(recipient),
            sender,
            Some(SequenceIdV1 {
                hash: Id::random(),
                num: 123,
                max: 123,
            }),
        );

        // Iterate over all available blocks and their hash
        // references.  The hash reference is used as the first part
        // of the SequenceId to make re-association on the other side
        // possible.
        for (block_ref, block_data) in input {
            let max_payload_size =
                schema_frame.get_max_size(ctx.core.get_route_mtu(Some(recipient)))?;
            let block_ref = Id::from_bytes(block_ref.as_slice());

            // We chunk the data block into as many pieces as are
            // required for the current MTU.  Each carrier frame gets
            // assigned the same sequence ID hash, with an
            // incrementing numerical count.  This way we can re-order
            // frames that have arrived out of order.
            let mut ctr = 0;
            for chunk in block_data.as_slice().chunks(max_payload_size as usize) {
                let seq_id = SequenceIdV1 {
                    hash: block_ref,
                    num: ctr,
                    // fixme: properly handle this casting, and make
                    // sure we don't get weird division errors here!
                    max: (block_data.as_slice().len() / max_payload_size as usize) as u8,
                };

                let mut carrier_v1 = new_carrier_v1(Some(recipient), sender, Some(seq_id));
                carrier_v1.payload = chunk.into();
                buf.push(CarrierFrame::V1(carrier_v1));
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
    pub(crate) async fn slice<const BS: usize>(
        ctx: &RatmanContext,
        msg: Message,
    ) -> Result<(ReadCapability, MemoryStorage)> {
        let mut blocks = MemoryStorage::new();

        let key = ctx
            .keys
            .diffie_hellman(
                msg.sender,
                msg.recipient
                    .scope()
                    .expect("Can't encrypt message to missing recipient"),
            )
            .await
            .expect("failed to perform diffie-hellman");

        let manifest = async_eris::encode(
            &mut msg.payload.as_slice(),
            key.as_bytes(),
            BlockSize::_1K,
            &mut blocks,
        )
        .await
        .expect("failed to encode block stream");

        Ok((manifest, blocks))
    }
}
