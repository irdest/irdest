use crate::{
    context::RatmanContext,
    journal::{types::BlockData, Journal},
    procedures::BlockNotifier,
    storage::{
        block::{IncompleteBlockData, StorageBlock},
        MetadataDb,
    },
};
use libratman::{
    tokio::{
        select,
        sync::{
            broadcast::Sender as BcastSender,
            mpsc::{channel, Receiver, Sender},
            RwLock,
        },
        task::{self, spawn_blocking},
    },
    types::{Ident32, InMemoryEnvelope, SequenceIdV1},
    EncodingError, RatmanError, Result,
};
use std::{collections::BTreeMap, sync::Arc};

type EnvSender = Sender<(SequenceIdV1, InMemoryEnvelope)>;
type EnvReceiver = Receiver<(SequenceIdV1, InMemoryEnvelope)>;
type SenderStore = Arc<RwLock<BTreeMap<Ident32, EnvSender>>>;

pub async fn exec_block_collector_system(
    ctx: Arc<RatmanContext>,
    mut rx: Receiver<InMemoryEnvelope>,
    block_bcast: BcastSender<BlockNotifier>,
) -> Result<()> {
    loop {
        let tripwire = ctx.tripwire.clone();
        select! {
            biased;
            _ = tripwire => {},
            env = rx.recv() => {
                if env.is_none() {
                    debug!("Reached the end of frame envelope stream");
                    break;
                }

                Arc::clone(&ctx.collector).queue_and_spawn(env.unwrap(), block_bcast.clone()).await?;
            }
        }
    }

    info!("Block collector system shut down");
    Ok(())
}

/// Takes a series of frames and re-constructs a single ERIS block
pub struct BlockCollectorWorker {
    max_num: u8,
    buffer: Vec<InMemoryEnvelope>,
    senders: SenderStore,
    journal: Arc<Journal>,
}

impl BlockCollectorWorker {
    /// Spawn this!
    pub async fn run(mut self, mut recv: EnvReceiver, block_bcast: BcastSender<BlockNotifier>) {
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
                    Ok(block) => {
                        let journal = Arc::clone(&self.journal);
                        spawn_blocking(move || {
                            journal
                                .blocks
                                .insert(
                                    block.reference().to_string(),
                                    &BlockData {
                                        data: block.into(),
                                        valid: true,
                                    },
                                )
                                .expect("failed to insert block into journal!")
                        })
                        .await
                        .unwrap();
                    }
                    Err(e) => error!("failed to reconstruct block: {e:?}"),
                }

                // Notify all current stream re-assemblers
                block_bcast.send(BlockNotifier);

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
    pub async fn restore(
        journal: Arc<Journal>,
        meta_db: Arc<MetadataDb>,
        block_bcast: BcastSender<BlockNotifier>,
    ) -> Result<Arc<Self>> {
        let this = Arc::new(Self {
            inner: Default::default(),
            journal,
            meta_db,
        });

        // Restore existing workers for blocks that were still being assembled
        // when the router last shut down
        for entry in Arc::clone(&this.meta_db).incomplete.0.iter() {
            let (key, val) = entry?;
            let id = Ident32::from_bytes(&key);
            let incomplete = rmp_serde::from_slice::<'_, IncompleteBlockData>(&val)
                .map_err(|e| EncodingError::Internal(e.to_string()))?;

            let journal = Arc::clone(&this.journal);
            let prefix_key = format!("{}::*", id);
            let frames = journal.frames.prefix(&prefix_key);

            let mut len = 0;
            for (_, frame_data) in frames {
                len += 1;
                this.queue_and_spawn(
                    InMemoryEnvelope {
                        header: frame_data.header.maybe_inner()?,
                        buffer: frame_data.payload,
                    },
                    block_bcast.clone(),
                )
                .await?;
            }

            info!(
                "Restoring block collection worker for block {id}; {}/{}",
                len, incomplete.max_num
            );
        }

        Ok(this)
    }

    /// Queue a new frame and spawn a collection worker if none exists yet
    pub async fn queue_and_spawn(
        self: &Arc<Self>,
        env: InMemoryEnvelope,
        block_bcast: BcastSender<BlockNotifier>,
    ) -> Result<()> {
        let sequence_id = env
            .header
            .get_seq_id()
            .map_or(Err(RatmanError::DesequenceFault), |i| Ok(i))?;
        let _max_num = sequence_id.max;

        if let Ok(Some(mut block_meta)) = self.meta_db.incomplete.get(&sequence_id.hash.to_string())
        {
            block_meta.buffer.push(sequence_id.num);
            self.meta_db
                .incomplete
                .insert(sequence_id.hash.to_string(), &block_meta)?;
        } else {
            self.meta_db.incomplete.insert(
                sequence_id.hash.to_string(),
                &IncompleteBlockData {
                    max_num: sequence_id.max,
                    buffer: vec![sequence_id.num],
                },
            )?;
        }

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

                // We drop our read handle, then switch to a write handle
                drop(read);
                self.inner
                    .write()
                    .await
                    .insert(sequence_id.hash, tx.clone());

                // Setup a new block worker
                let senders = Arc::clone(&self.inner);
                debug!(
                    "Spawn new frame collector for block_id {}",
                    sequence_id.hash
                );
                task::spawn_local(
                    BlockCollectorWorker {
                        max_num: sequence_id.max,
                        buffer: vec![],
                        senders,
                        journal: Arc::clone(&self.journal),
                    }
                    .run(rx, block_bcast.clone()),
                );

                // Finally queue the first envelope!
                if let Err(e) = tx.send((sequence_id, env)).await {
                    error!("failed to forward frame to collection worker: {e}");
                }
            }
        }

        Ok(())
    }
}
