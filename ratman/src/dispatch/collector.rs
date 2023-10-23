use crate::core::JournalSender;
use async_std::{
    channel::{self, Receiver, Sender},
    sync::{Arc, RwLock, RwLockReadGuard},
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
            let insert_at_end = seq_id.num as usize >= this.buffer.len();
            trace!(
                "Insert chunk in sequence {} to index {}",
                seq_id.hash,
                if insert_at_end { -1 } else { seq_id.num as i8 }
            );

            // If the index we're looking at is beyond the limit of
            // the current vector, append the envelope to the end.  We
            // do this until the indices start being in range, at
            // which point we insert into the exact index instead.
            if insert_at_end {
                this.buffer.push(envelope)
            } else {
                this.buffer.insert(seq_id.num.into(), envelope);
            }

            // If the block is complete
            if this.buffer.len() == this.max_num as usize {
                info!(
                    "Collected enough chunks ({}) to reconstruct block {}",
                    this.buffer.len(),
                    seq_id.hash
                );

                // Remove the sender
                this.senders.write().await.remove(&seq_id.hash);

                // Re-assemble the block
                let mut block = vec![];
                core::mem::replace(&mut this.buffer, Default::default())
                    .into_iter()
                    // todo: can we avoid copying here?
                    .for_each(|mut chunk| block.extend_from_slice(chunk.get_payload_slice()));

                // Then offer the finished block up to the block god
                this.output.send((block, seq_id)).await;

                // Finally shut down this block collection worker
                self.senders.write().await.remove(&seq_id.hash);
                break;
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

    /// Close the journal sender channel
    ///
    /// This signals that after the remaining BlockCollectionWorkers
    /// have run, no more blocks will come.
    pub fn queue_shutdown(&self) {
        // self.output.close();
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
                        max_num: sequence_id.max,
                        buffer: vec![],
                        senders,
                        output: self.output.clone(),
                    }
                    .run(rx),
                );

                // We drop our read handle, then switch to a write handle
                drop(read);
                self.inner
                    .write()
                    .await
                    .insert(sequence_id.hash, tx.clone());

                // Finally queue the first envelope!
                tx.send((sequence_id, env)).await;
            }
        }

        Ok(())
    }
}
