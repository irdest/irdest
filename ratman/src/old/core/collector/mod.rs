// SPDX-FileCopyrightText: 2020, 2022 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2020 Jess 3Jane <me@jess.coffee>
// SPDX-FileCopyrightText: 2020 Alyssa Ross <hi@alyssa.is>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! The frame/message collector module
//!
//! The collector module is a bit more complicated than other modules,
//! because of the layers of state and control inversion it has to
//! contend with.
//!
//! It would be possible to do all in one file, but it would quickly
//! become too complicated, and unmaintainable.  Instead, this module
//! splits the code into three sections: the state, the worker, and
//! the manager.  The former two exploit the latter for profit.
//!
//! The manager is exposed from this module as `Collector`, so that
//! the routing core and other modules don't have to care about the
//! inner workings.  The state mostly provides a way to create and
//! yield workers, that are being polled by the manager.  The workers
//! themselves have very little control over their environment, only
//! getting access to the state manager to ask for more work, and then
//! making themselves redundant by handing in their finished messages.

use crate::Message;
use async_std::{
    sync::{Arc, Mutex},
    task,
};
use std::collections::BTreeMap;
use tracing_futures::Instrument;
use types::{Frame, Id};

pub(self) type Locked<T> = Arc<Mutex<T>>;

mod state;
pub(self) use state::State;

mod worker;
pub(self) use worker::Worker;

/// The main collector management structure and API facade
pub(crate) struct Collector {
    state: Arc<State>,
    workers: Locked<BTreeMap<Id, Arc<Worker>>>,
}

impl Collector {
    /// Create a new collector
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            state: Arc::new(State::new()),
            workers: Default::default(),
        })
    }

    /// Queue a new frame to collect
    ///
    /// This function can spawn new workers when needed
    #[cfg(test)]
    pub(crate) async fn queue(&self, seq: Id, f: Frame) {
        trace!("Queuing frame for collection");
        self.state.queue(seq, f).await;

        let mut map = self.workers.lock().await;
        if !map.contains_key(&seq) {
            map.insert(seq, Arc::new(Worker::new(seq, Arc::clone(&self.state))));
        }
    }

    /// Queue the work, and spawn a worker if required
    #[instrument(skip(self, f), level = "trace")]
    pub(crate) async fn queue_and_spawn(&self, seq: Id, f: Frame) {
        self.state.queue(seq, f).await;

        let mut map = self.workers.lock().await;
        if !map.contains_key(&seq) {
            debug!("Spawning new collector thread");
            map.insert(seq, Arc::new(Worker::new(seq, Arc::clone(&self.state))));
            drop(map);

            // This function tries to re-lock!
            self.spawn_worker(seq).await;
        }
    }

    /// Get any message that has been completed
    pub(crate) async fn completed(&self) -> Message {
        self.state.completed().await
    }

    #[cfg(test)]
    pub(crate) async fn num_queued(&self) -> usize {
        self.state.num_queued().await
    }

    #[cfg(test)]
    #[allow(unused)]
    pub(crate) async fn num_completed(&self) -> usize {
        self.state.num_completed().await
    }

    /// Get raw access to a worker poll cycle, for testing purposes
    #[cfg(test)]
    async fn get_worker(&self, seq: Id) -> Arc<Worker> {
        Arc::clone(&self.workers.lock().await.get(&seq).unwrap())
    }

    /// Spawn an async task runner for a worker
    async fn spawn_worker(&self, seq: Id) {
        let workers = Arc::clone(&self.workers);

        let worker = {
            let map = workers.lock().await;
            Arc::clone(&map.get(&seq).unwrap())
        };

        task::spawn(
            async move {
                trace!("Spawning worker");

                // This loop breaks when the worker is done
                while let Some(()) = worker.poll().await {}

                // Then remove it
                let mut map = workers.lock().await;
                map.remove(&seq).unwrap();
            }
            .instrument(trace_span!("Worker", seq = seq.to_string().as_str())),
        );
    }
}

#[cfg(test)]
use crate::Address;

#[test]
fn queue_one() {
    use crate::{TimePair, TransportSlicer};
    use types::Recipient;

    let (sender, recipient, id) = (Address::random(), Address::random(), Address::random());
    let mut seq = TransportSlicer::slice(
        128,
        Message {
            id,
            sender,
            recipient: Recipient::Standard(vec![recipient]),
            payload: vec![0, 1, 2, 3, 1, 3, 1, 2, 1, 3, 3, 7],
            timesig: TimePair::sending(),
            sign: vec![0, 1],
        },
    );

    assert_eq!(seq.len(), 1);
    let frame = seq.remove(0);
    let seqid = id;

    task::block_on(async move {
        let c = Collector::new();

        // There is one queued frame
        c.queue(seqid, frame).await;
        assert!(c.num_queued().await == 1);

        // After we handle it, the worker can die
        let w = c.get_worker(seqid).await;
        assert!(w.poll().await == None);

        // Now get the finished message
        assert!(c.completed().await.id == seqid);
    });
}

// This test has produced some weird behaviour on aarch64-darwin.
// Generally we should re-evaluate this test in terms of scope and
// functionality, i.e. we should switch to a different encoding
// mechanism, etc.
#[ignore]
#[test]
fn queue_many() {
    use crate::{TimePair, TransportSlicer};
    use types::Recipient;

    let (sender, recipient, id) = (Address::random(), Address::random(), Address::random());
    let seq = TransportSlicer::slice(
        8,
        Message {
            id,
            sender,
            recipient: Recipient::Standard(vec![recipient]),
            payload: vec![0, 1, 2, 3, 1, 3, 1, 2, 1, 3, 3, 7],
            timesig: TimePair::sending(),
            sign: vec![],
        },
    );

    let seqid = id;
    let len = seq.len();
    assert_eq!(len, 9);

    task::block_on(async move {
        let c = Collector::new();

        for f in seq {
            c.queue(seqid, f).await;
        }

        // There is n queued frames
        assert_eq!(c.num_queued().await, len);

        let w = c.get_worker(seqid).await;

        // There will be len - 1 items, followed by a None.
        for _ in 1..len {
            assert_eq!(w.poll().await, Some(()));
        }
        assert_eq!(w.poll().await, None);

        // Now get the finished message
        assert_eq!(c.completed().await.id, seqid);
    });
}
