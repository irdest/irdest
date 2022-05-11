// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Asynchronous Ratman routing core

use crate::{
    core::{Collector, DriverMap, EpTargetPair, RouteTable},
    Message, Result, Slicer,
};
use async_std::{sync::Arc, task};
use netmod::Target;
use types::{Frame, Recipient};

pub(crate) struct Dispatch {
    routes: Arc<RouteTable>,
    drivers: Arc<DriverMap>,
    collector: Arc<Collector>,
    metrics: Arc<metrics::Metrics>,
}

impl Dispatch {
    /// Create a new frame dispatcher
    pub(crate) fn new(
        routes: Arc<RouteTable>,
        drivers: Arc<DriverMap>,
        collector: Arc<Collector>,
    ) -> Arc<Self> {
        Arc::new(Self {
            routes,
            drivers,
            collector,
            metrics: Arc::new(metrics::Metrics::default()),
        })
    }

    pub(crate) fn register_metrics(&self, registry: &mut prometheus_client::registry::Registry) {
        self.metrics.register(registry);
    }

    pub(crate) async fn send_msg(&self, msg: Message) -> Result<()> {
        let r = msg.recipient.clone();
        trace!("dispatching message to recpient: {:?}", r);
        self.metrics
            .messages_total
            .get_or_create(&metrics::Labels {
                recipient: (&msg.recipient).into(),
            })
            .inc();

        // This is a hardcoded MTU for now.  We need to adapt the MTU
        // to the interface we're broadcasting on and we potentially
        // need a way to re-slice, or combine frames that we encounter
        // for better transmission metrics
        let frames = Slicer::slice(1312, msg);

        frames.into_iter().fold(Ok(()), |res, f| match (res, &r) {
            (Ok(()), Recipient::Standard(_)) => task::block_on(self.send_one(f)),
            (Ok(()), Recipient::Flood(_)) => task::block_on(self.flood(f)),
            (res, _) => res,
        })
    }

    /// Dispatch a single frame across the network
    pub(crate) async fn send_one(&self, frame: Frame) -> Result<()> {
        let EpTargetPair(epid, trgt) = match self
            .routes
            .resolve(match frame.recipient {
                ref recp @ Recipient::Standard(_) => recp.scope().expect("empty recipient"),
                Recipient::Flood(_) => unreachable!(),
            })
            .await
        {
            Some(resolve) => resolve,

            // FIXME: "local address" needs to be handled in a
            // much more robust manner than this!  Previously this
            // issue was caught on a different layer, but we can't
            // rely on this anymore.  So: differentiate between
            // local and remote addresses and route accordingly.
            None => {
                self.collector.queue_and_spawn(frame.seqid(), frame).await;
                return Ok(());
            }
        };

        self.metrics
            .frames_total
            .get_or_create(&metrics::Labels {
                recipient: (&frame.recipient).into(),
            })
            .inc();
        let ep = self.drivers.get(epid as usize).await;
        Ok(ep.send(frame, trgt).await?)
    }

    pub(crate) async fn flood(&self, frame: Frame) -> Result<()> {
        for ep in self.drivers.get_all().await.into_iter() {
            let f = frame.clone();
            let target = Target::Flood(frame.recipient.scope().expect("empty recipient"));
            self.metrics
                .frames_total
                .get_or_create(&metrics::Labels {
                    recipient: (&frame.recipient).into(),
                })
                .inc();
            ep.send(f, target).await.unwrap();
        }

        Ok(())
    }

    /// Reflood a message to the network, except the previous interface
    pub(crate) async fn reflood(&self, frame: Frame, ep: usize) {
        for ep in self.drivers.get_without(ep).await.into_iter() {
            let f = frame.clone();
            let target = Target::Flood(f.recipient.scope().expect("empty recipient"));
            task::spawn(async move { ep.send(f, target).await.unwrap() });
        }
    }
}

mod metrics {
    use prometheus_client::{
        encoding::text::Encode,
        metrics::{counter::Counter, family::Family},
        registry::Registry,
    };

    #[derive(Clone, Hash, PartialEq, Eq, Encode)]
    pub(super) struct Labels {
        pub recipient: Recipient,
    }

    #[derive(Clone, Hash, PartialEq, Eq)]
    pub(super) enum Recipient {
        Standard,
        Flood,
    }

    impl From<&types::Recipient> for Recipient {
        fn from(v: &types::Recipient) -> Self {
            match v {
                &types::Recipient::Standard(_) => Self::Standard,
                &types::Recipient::Flood(_) => Self::Flood,
            }
        }
    }

    // Manually implement Encode to produce eg. `recipient=standard` rather than `recipient=Standard`.
    impl Encode for Recipient {
        fn encode(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
            match self {
                Self::Standard => write!(w, "standard"),
                Self::Flood => write!(w, "flood"),
            }
        }
    }

    #[derive(Default)]
    pub(super) struct Metrics {
        pub messages_total: Family<Labels, Counter>,
        pub frames_total: Family<Labels, Counter>,
    }

    impl Metrics {
        pub fn register(&self, registry: &mut Registry) {
            registry.register(
                "ratman_dispatch_messages",
                "Total number of messages dispatched",
                Box::new(self.messages_total.clone()),
            );
            registry.register(
                "ratman_dispatch_frames",
                "Total number of frames dispatched",
                Box::new(self.frames_total.clone()),
            );
        }
    }
}
