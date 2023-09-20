// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Slices `Message` into a series of Frames

use crate::{context::RatmanContext, core::Payload};
use async_eris::{BlockSize, MemoryStorage, ReadCapability};
use libratman::types::{Frame, Message, Result, SeqBuilder};

/// Slices messages into managable chunks
// TODO: refactor this type to be a rolling window slicer based on a
// stream of eris blocks.
pub(crate) struct TransportSlicer;

impl TransportSlicer {
    /// Take a `Message` and split it into a list of `Frames`
    pub(crate) fn slice(max: usize, msg: Message) -> Vec<Frame> {
        let payload = bincode::serialize(&Payload {
            payload: msg.payload,
            time: msg.time,
            signature: msg.signature,
        })
        .unwrap();

        payload
            .as_slice()
            .chunks(max)
            .fold(
                SeqBuilder::new(msg.sender, msg.recipient, msg.id),
                |seq, chunk| seq.add(chunk.into_iter().cloned().collect()),
            )
            .build()
    }
}

/// A simple slicer for ERIS blocks
#[allow(unused)]
pub(crate) struct BlockSlicer;

impl BlockSlicer {
    #[allow(unused)]
    // Signature should be Vec<Block<BS>> with BS
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
