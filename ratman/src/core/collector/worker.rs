// SPDX-FileCopyrightText: 2020 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! The collector worker

use super::{Locked, State};
use crate::{Message, Payload};
use async_std::sync::Arc;
use std::convert::TryInto;
use eris::{BlockStorage, Block, MemoryStorage, ReadCapability};
use types::{Frame, SeqBuilder, SeqId};

/// A self contained sub-task that collects frames into messages
pub(super) struct Worker {
    /// The sequence of the message being collected
    seq: SeqId,
    /// The buffer of existing messages
    buf: Locked<Vec<Frame>>,
    /// Collector reference for control flow
    parent: Arc<State>,
}

impl Worker {
    /// Create a new collector task for a collector parent
    pub(super) fn new(seq: SeqId, parent: Arc<State>) -> Self {
        Self {
            seq,
            parent,
            buf: Default::default(),
        }
    }

    /// Poll for new frames to assemble from the frame pool
    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn poll(&self) -> Option<()> {
        trace!("Polling for new work to be done");
        let frame = self.parent.get(&self.seq).await;
        let mut buf = self.buf.lock().await;

        if let Some(msg) = join_frames::<1024>(&mut buf, frame) {
            debug!("Joining frames");
            self.parent.finish(msg).await;
            None
        } else {
            Some(())
        }
    }
}

/// Utility function that uses the SeqBuilder to rebuild Sequence
fn join_frames<const BS: usize>(buf: &mut Vec<Frame>, new: Frame) -> Option<Message> {
    // Insert the frame
    buf.push(new);

    // Sort by sequence numbers
    buf.sort_by(|a, b| a.seq.num.cmp(&b.seq.num));

    // The last frame needs to point to `None`
    if buf.last().unwrap().seq.next.is_some() {
        return None;
    }
    // Test inductive sequence number property
    if buf.iter().enumerate().fold(true, |status, (i, frame)| {
        status && (frame.seq.num == i as u32)
    }) {
        let id = buf[0].seq.seqid;
        let sender = buf[0].sender;
        let recipient = buf[0].recipient.clone();
        let eris_encoded = match SeqBuilder::restore(buf) {
            Ok(v) => v,
            Err(_) => return None,
        };
        let layered = async_std::task::block_on(async move {
            let eris_blocks = eris_encoded.chunks_exact(BS);
            let mut store = MemoryStorage::new();
            println!("{:x?}", eris_blocks.remainder());
            let read_capability = ReadCapability::from_binary(eris_blocks.remainder()).unwrap();
            println!("{:?}", read_capability);
            for block in eris_blocks.map(|x| {
                let arr: [u8; BS] = x.try_into().unwrap();
                Block::from(arr)
            }) {
                store.store(&block).await.unwrap();
            }
            let mut res = vec![];
            eris::decode_const::<_, _, BS>(&mut res, &read_capability, &mut store).await.unwrap();
            res
        });
        let Payload {
            payload,
            mut timesig,
            sign,
        } = bincode::deserialize(&layered).unwrap();

        // Update the received timestamp in the message
        timesig.receive();

        Some(Message {
            id,
            sender,
            recipient,
            timesig,
            payload,
            sign,
        })
    } else {
        None
    }
}

#[cfg(test)]
use types::{Identity, Recipient};

// This test is broken because currently it just creates a sequence of
// bytes that can then not be deserialised by bincode into a Payload
// type.  The problem is that we want to manually build up a sequence
// of three frames to not rely on the Slicer in this test.
#[ignore]
#[test]
fn join_frame_simple() {
    let sender = Identity::random();
    let recp = Identity::random();
    let seqid = Identity::random();

    let mut seq_builder = SeqBuilder::new(sender, Recipient::Standard(vec![recp]), seqid);
    seq_builder.add((0..10).into_iter().collect());
    seq_builder.add((10..20).into_iter().collect());
    seq_builder.add((20..30).into_iter().collect());
    let mut seq = seq_builder.build();

    // The function expects a filling buffer
    let mut buf = vec![];

    assert!(join_frames::<1>(&mut buf, seq.remove(0)) == None);
    assert!(join_frames::<1>(&mut buf, seq.remove(1)) == None); // Insert out of order
    assert!(join_frames::<1>(&mut buf, seq.remove(0)).is_some());
}
