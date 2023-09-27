use async_eris::Block;
use async_std::{
    channel::{self, Receiver, Sender},
    sync::RwLock,
    task,
};
use libratman::{netmod::InMemoryEnvelope, types::frames::SequenceIdV1};

type EnvSender = Sender<(SequenceIdV1, InmemoryEnvelope)>;
type EnvReceiver = Receiver<(SequenceIdV1, InmemoryEnvelope)>;
type SenderStore = Arc<RwLock<BTreeMap<Id, EnvSender>>>;

/// Takes a series of frames and re-constructs a single ERIS block
pub struct BlockCollectorWorker {
    sequence_id: Id,
    max_num: u8,
    buffer: Vec<InMemoryEnvelope>,
    senders: SenderStore,
    output: Sender<Vec<u8>>,
}

impl BlockCollectorWorker {
    /// Spawn this!
    pub async fn run(mut self, recv: EnvReceiver) {
        while let Some((seq_id, envelope)) = recv.recv().await {
            self.buffer.insert(seq_id.num, envelope);
            self.buffer.sort_by(|a, b| {
                a.meta
                    .seq_id
                    .unwrap()
                    .num
                    .partial_cmp(&b.meta.seq_id.unwrap().num)
                    .unwrap()
            });

            // If the block is complete
            if self.buffer.len() == self.max_num as usize {
                // Remove the sender
                senders.write().await.remove(seq_id);

                // Re-assemble the block
                let mut block = vec![];
                self.buffer.into_iter().for_each(|mut chunk| {
                    block.append(&mut chunk.buffer);
                });

                // Then offer the finished block up to the block god
                self.output.send(block).await;
            }
        }
    }
}

pub struct BlockCollector {
    inner: SenderStore,
    output: Sender<Vec<u8>>,
}

impl BlockCollector {
    pub fn new(output: Sender<Vec<u8>>) -> Arc<Self> {
        Arc::new(Self {
            inner: Default::default(),
            output,
        })
    }

    /// Queue a new frame and spawn a collection worker if none exists yet
    pub async fn queue_and_spawn(
        self: &Arc<Self>,
        sequence_id: Id,
        max_num: u8,
        env: InMemoryEnvelope,
    ) {
        let maybe_sender = self.inner.read().await.get(&sequence_id);
        match maybe_sender {
            Some(sender) => sender.send(env).await,
            None => {
                let (tx, rx) = channel::bounded(8);
                let senders = Arc::clone(&self.inner);
                task::spawn(
                    BlockCollectorWorker {
                        sequence_id,
                        max_num,
                        buffer: vec![],
                        senders,
                        output: self.output.clone(),
                    }
                    .run(rx),
                );
                self.inner.write().await.insert(sequence_id, tx);
            }
        }
    }
}
