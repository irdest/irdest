// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Slices `Message` into a series of Frames

use crate::{Message, Payload};
use types::{Frame, SeqBuilder};
use std::ops::Deref;
use eris::{BlockStorage, Block, BlockReference};
use async_trait::async_trait;

/// Slices messages into managable chunks
pub(crate) struct Slicer;

struct SeqBuilderWrapper {
    inner: SeqBuilder,
}

impl SeqBuilderWrapper {
    fn new(inner: SeqBuilder) -> Self {
        Self { inner }
    }

    fn into_inner(self) -> SeqBuilder {
        self.inner
    }
}

#[async_trait]
impl<const BS: usize> BlockStorage<BS> for SeqBuilderWrapper {
    async fn store(&mut self, block: &Block<BS>) -> std::io::Result<()> {
        self.inner.add(block.deref().to_vec());
        Ok(())
    }
    async fn fetch(&self, _reference: &BlockReference) -> std::io::Result<Option<Block<BS>>> {
        Ok(None)
    }
}

impl Slicer {
    /// Take a `Message` and split it into a list of `Frames`
    pub(crate) fn slice<const BS: usize>(msg: Message) -> Vec<Frame> {
        let payload = bincode::serialize(&Payload {
            payload: msg.payload,
            timesig: msg.timesig,
            sign: msg.sign,
        })
        .unwrap();

        let mut store = SeqBuilderWrapper::new(SeqBuilder::new(msg.sender, msg.recipient, msg.id));
        let key = [0u8; 32];
        let read_capability = async_std::task::block_on(eris::encode_const::<_, _, BS>(&mut payload.as_slice(), &key, &mut store)).unwrap();
        println!("put {:?}", read_capability);
        println!("put {:x?}", read_capability.binary());
        store.inner.add(read_capability.binary());
        store.into_inner().build()
    }
}
