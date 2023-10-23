use crate::core::JournalSender;
use async_std::{
    channel::{self, Receiver, Sender},
    sync::{Arc, RwLock},
    task,
};
use libratman::{
    netmod::InMemoryEnvelope,
    types::{Id, Result, SequenceIdV1},
    RatmanError,
};
use std::collections::BTreeMap;

type EnvSender = Sender<(SequenceIdV1, InMemoryEnvelope)>;
type EnvReceiver = Receiver<(SequenceIdV1, InMemoryEnvelope)>;
type SenderStore = Arc<RwLock<BTreeMap<Id, EnvSender>>>;

/// Takes a series of frames and re-constructs a single ERIS block
pub struct BlockCollectorWorker {
    sequence_id: Id,
    max_num: u8,
    buffer: Vec<InMemoryEnvelope>,
    senders: SenderStore,
    output: JournalSender,
}

impl BlockCollectorWorker {
    /// Spawn this!
    pub async fn run(mut self, recv: EnvReceiver) {
        let this = &mut self;
        while let Ok((seq_id, envelope)) = recv.recv().await {
            this.buffer.insert(seq_id.num.into(), envelope);
            this.buffer.sort_by(|a, b| {
                a.header
                    .get_seq_id()
                    .unwrap()
                    .hash
                    .partial_cmp(&b.header.get_seq_id().unwrap().hash)
                    .unwrap()
            });

            // If the block is complete
            if this.buffer.len() == this.max_num as usize {
                // Remove the sender
                this.senders.write().await.remove(&seq_id.hash);

                // Re-assemble the block
                let mut block = vec![];
                core::mem::replace(&mut this.buffer, Default::default())
                    .into_iter()
                    .for_each(|mut chunk| {
                        block.append(&mut chunk.buffer);
                    });

                // Then offer the finished block up to the block god
                this.output.send((block, seq_id)).await;
            }
        }
    }
}

pub struct BlockCollector {
    inner: SenderStore,
    output: JournalSender,
}

impl BlockCollector {
    pub fn new(output: JournalSender) -> Arc<Self> {
        Arc::new(Self {
            inner: Default::default(),
            output,
        })
    }

    /// Queue a new frame and spawn a collection worker if none exists yet
    pub async fn queue_and_spawn(self: &Arc<Self>, env: InMemoryEnvelope) -> Result<()> {
        let sequence_id = env
            .header
            .get_seq_id()
            .map_or(Err(RatmanError::DesequenceFault), |i| Ok(i))?;
        let max_num = sequence_id.max;

        let read = self.inner.read().await;
        let maybe_sender = read.get(&sequence_id.hash);
        match maybe_sender {
            Some(sender) => {
                debug!("Queue new frame for block_id {}", sequence_id.hash);
                sender.send((sequence_id, env)).await;
                Ok(())
            }
            None => {
                let (tx, rx) = channel::bounded(8);
                let senders = Arc::clone(&self.inner);
                debug!(
                    "Spawn new frame collector for block_id {}",
                    sequence_id.hash
                );
                task::spawn(
                    BlockCollectorWorker {
                        sequence_id: sequence_id.hash,
                        max_num,
                        buffer: vec![],
                        senders,
                        output: self.output.clone(),
                    }
                    .run(rx),
                );
                drop(read);
                self.inner.write().await.insert(sequence_id.hash, tx);
                Ok(())
            }
        }
    }
}
