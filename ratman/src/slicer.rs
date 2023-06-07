// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Slices `Message` into a series of Frames

use libratman::types::{Frame, Message};

/// Slices messages into managable chunks
pub(crate) struct TransportSlicer;

impl TransportSlicer {
    /// Take a `Message` and split it into a list of `Frames`
    pub(crate) fn slice(max: usize, msg: Message) -> Vec<Frame> {
        // let payload = bincode::serialize(&Payload {
        //     payload: msg.payload,
        //     timesig: msg.timesig,
        //     sign: msg.sign,
        // })
        // .unwrap();

        // payload
        //     .as_slice()
        //     .chunks(max)
        //     .fold(
        //         SeqBuilder::new(msg.sender, msg.recipient, msg.id),
        //         |seq, chunk| seq.add(chunk.into_iter().cloned().collect()),
        //     )
        //     .build()
        todo!()
    }
}

/// A simple slicer for ERIS blocks
#[allow(unused)]
pub(crate) struct BlockSlicer;

impl BlockSlicer {
    #[allow(unused)]
    // Signature should be Vec<Block<BS>> with BS
    pub(crate) async fn slice<const BS: usize>(router: &(), msg: Message) -> Vec<Vec<u8>> {
        // let mut blocks = MemoryStorage::new();

        // let key = router
        //     .keys
        //     .diffie_hellman(
        //         msg.sender,
        //         msg.recipient.scope().expect(
        //             "Can't encrypt message to
        //     missing recipient",
        //         ),
        //     )
        //     .await
        //     .expect("Failed to perform diffie-hellman");

        // async_eris::encode(
        //     &mut msg.payload.as_slice(),
        //     key.as_bytes(),
        //     BlockSize::_1K,
        //     &mut blocks,
        // )
        // .await
        // .unwrap();

        vec![]
    }
}
