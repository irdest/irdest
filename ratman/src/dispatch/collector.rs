use crate::{
    journal::{types::BlockData, Journal},
    storage::{
        block::{IncompleteBlockData, StorageBlock},
        MetadataDb,
    },
};
use libratman::{
    tokio::{
        sync::{
            mpsc::{channel, Receiver, Sender},
            RwLock,
        },
        task,
    },
    types::{Id, InMemoryEnvelope, SequenceIdV1},
    EncodingError, RatmanError, Result,
};
use serde::Deserialize;
use std::{collections::BTreeMap, sync::Arc};

type EnvSender = Sender<(SequenceIdV1, InMemoryEnvelope)>;
type EnvReceiver = Receiver<(SequenceIdV1, InMemoryEnvelope)>;
type SenderStore = Arc<RwLock<BTreeMap<Id, EnvSender>>>;

/// Takes a series of frames and re-constructs a single ERIS block
pub struct BlockCollectorWorker {
    sequence_id: Id,
    max_num: u8,
    buffer: Vec<InMemoryEnvelope>,
    senders: SenderStore,
    journal: Arc<Journal>,
    meta_db: Arc<MetadataDb>,
}

impl BlockCollectorWorker {
    /// Spawn this!
    pub async fn run(mut self, mut recv: EnvReceiver) {
        let this = &mut self;
        while let Some((seq_id, envelope)) = recv.recv().await {
            let insert_at_end = seq_id.num as usize >= this.buffer.len();
            trace!(
                "Insert chunk in sequence {} to index {}",
                seq_id.hash,
                if insert_at_end { -1 } else { seq_id.num as i8 }
            );

            // If the index we're looking at is beyond the limit of the current
            // vector, append the envelope to the end.  We do this until the
            // indices start being in range, at which point we insert into the
            // exact index instead.
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
                    .for_each(|chunk| block.extend_from_slice(chunk.get_payload_slice()));

                // Then offer the finished block up to the block god
                match StorageBlock::reconstruct_from_vec(block) {
                    Ok(block) => self
                        .journal
                        .blocks
                        .insert(
                            block.reference().to_string(),
                            &BlockData {
                                data: block.into(),
                                valid: true,
                            },
                        )
                        .expect("failed to insert block into journal!"),
                    Err(e) => error!("failed to reconstruct block: {e:?}"),
                }

                // self.journal.blocks.insert(format!("{seq_id:?}"), block);
                // this.output.send((block, seq_id)).await;

                // Finally shut down this block collection worker
                self.senders.write().await.remove(&seq_id.hash);
                break;
            }
        }
    }
}

pub struct BlockCollector {
    inner: SenderStore,
    journal: Arc<Journal>,
    meta_db: Arc<MetadataDb>,
}

impl BlockCollector {
    pub async fn new(journal: Arc<Journal>, meta_db: Arc<MetadataDb>) -> Result<Arc<Self>> {
        let this = Arc::new(Self {
            inner: Default::default(),
            journal,
            meta_db,
        });

        // Restore existing workers for blocks that were still being assembled
        // when the router last shut down
        for entry in Arc::clone(&this.meta_db).incomplete.0.iter() {
            let (key, val) = entry?;
            let id = Id::from_bytes(&key);
            let incomplete = rmp_serde::from_slice::<'_, IncompleteBlockData>(&val)
                .map_err(|e| EncodingError::Internal(e.to_string()))?;

            info!(
                "Restoring block collection worker for block {id}; {}/{}",
                incomplete.buffer.len(),
                incomplete.max_num
            );
            for frame in incomplete.buffer {
                this.queue_and_spawn(InMemoryEnvelope {
                    header: frame.0.maybe_inner()?,
                    buffer: frame.1,
                })
                .await?;
            }
        }

        Ok(this)
    }

    /// Queue a new frame and spawn a collection worker if none exists yet
    pub async fn queue_and_spawn(self: &Arc<Self>, env: InMemoryEnvelope) -> Result<()> {
        let sequence_id = env
            .header
            .get_seq_id()
            .map_or(Err(RatmanError::DesequenceFault), |i| Ok(i))?;
        let _max_num = sequence_id.max;

        let read = self.inner.read().await;
        let maybe_sender = read.get(&sequence_id.hash);
        match maybe_sender {
            Some(sender) => {
                debug!("Queue new frame for block_id {}", sequence_id.hash);
                sender
                    .send((sequence_id, env))
                    .await
                    .expect("failed to send frame sequence to collection worker");
            }
            None => {
                let (tx, rx) = channel(8);
                let senders = Arc::clone(&self.inner);
                debug!(
                    "Spawn new frame collector for block_id {}",
                    sequence_id.hash
                );
                task::spawn_local(
                    BlockCollectorWorker {
                        sequence_id: sequence_id.hash,
                        max_num: sequence_id.max,
                        buffer: vec![],
                        senders,
                        journal: Arc::clone(&self.journal),
                        meta_db: Arc::clone(&self.meta_db),
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
                if let Err(e) = tx.send((sequence_id, env)).await {
                    error!("failed to forward frame to collection worker: {e}");
                }
            }
        }

        Ok(())
    }
}
