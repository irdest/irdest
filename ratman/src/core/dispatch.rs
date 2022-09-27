// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 Yureka Lilian <yuka@yuka.dev>
// SPDX-FileCopyrightText: 2022 embr <hi@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Asynchronous Ratman routing core

use crate::{
    core::{Collector, DriverMap, EpTargetPair, RouteTable},
    Message, Result, TransportSlicer,
};
use async_std::{sync::Arc, task};
use netmod::Target;
use types::{Frame, Recipient};

pub(crate) struct Dispatch {
    routes: Arc<RouteTable>,
    drivers: Arc<DriverMap>,
    collector: Arc<Collector>,
    #[cfg(feature = "dashboard")]
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
            #[cfg(feature = "dashboard")]
            metrics: Arc::new(metrics::Metrics::default()),
        })
    }

    #[cfg(feature = "dashboard")]
    pub(crate) fn register_metrics(&self, registry: &mut prometheus_client::registry::Registry) {
        self.metrics.register(registry);
    }

    pub(crate) async fn send_msg(&self, msg: Message) -> Result<()> {
        let r = msg.recipient.clone();
        trace!("dispatching message to recpient: {:?}", r);
        #[cfg(feature = "dashboard")]
        self.metrics
            .messages_total
            .get_or_create(&metrics::Labels {
                recp_type: (&r).into(),
                recp_id: r.scope().expect("empty recipient"),
            })
            .inc();

        // This is a hardcoded MTU for now.  We need to adapt the MTU
        // to the interface we're broadcasting on and we potentially
        // need a way to re-slice, or combine frames that we encounter
        // for better transmission metrics
        let frames = TransportSlicer::slice(1312, msg);

        frames.into_iter().fold(Ok(()), |res, f| match (res, &r) {
            (Ok(()), Recipient::Standard(_)) => task::block_on(self.send_one(f)),
            (Ok(()), Recipient::Flood(_)) => task::block_on(self.flood(f)),
            (res, _) => res,
        })
    }

    /// Dispatch a single frame across the network
    pub(crate) async fn send_one(&self, frame: Frame) -> Result<()> {
        let scope = match frame.recipient {
            ref recp @ Recipient::Standard(_) => recp.scope().expect("empty recipient"),
            Recipient::Flood(_) => unreachable!(),
        };

        #[cfg(feature = "dashboard")]
        {
            let metric_labels = &metrics::Labels {
                recp_type: metrics::RecipientType::Standard,
                recp_id: scope,
            };
            self.metrics.frames_total.get_or_create(metric_labels).inc();
            self.metrics
                .bytes_total
                .get_or_create(metric_labels)
                .inc_by(frame.payload.len() as u64);
        }

        let EpTargetPair(epid, trgt) = match self.routes.resolve(scope).await {
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

        let ep = self.drivers.get(epid as usize).await;
        Ok(ep.send(frame, trgt, None).await?)
    }

    pub(crate) async fn flood(&self, frame: Frame) -> Result<()> {
        for ep in self.drivers.get_all().await.into_iter() {
            let f = frame.clone();
            let scope = frame.recipient.scope().expect("empty recipient");
            let target = Target::Flood(scope);

            #[cfg(feature = "dashboard")]
            {
                let metric_labels = &metrics::Labels {
                    recp_type: metrics::RecipientType::Flood,
                    recp_id: scope,
                };
                self.metrics.frames_total.get_or_create(metric_labels).inc();
                self.metrics
                    .bytes_total
                    .get_or_create(metric_labels)
                    .inc_by(frame.payload.len() as u64);
            }

            ep.send(f, target, None).await.unwrap();
        }

        Ok(())
    }

    /// Reflood a message to the network, except the previous interface
    pub(crate) async fn reflood(
        &self,
        frame: Frame,
        originator_ep: usize,
        originator_peer: Target,
    ) {
        for (ep, ep_id) in self.drivers.get_with_ids().await.into_iter() {
            // When looking at the driver that handed us the flood we
            // explicitly exclude the originating peering target to
            // avoid endless replication.  But importantly we _must_
            // pass the flood back to the originating driver because
            // it may be handling a segmented peer space
            // (i.e. connections with peers that don't peer amongst
            // themselves).
            let exclude = match originator_peer {
                Target::Single(id) if ep_id == originator_ep => Some(id),
                _ => None,
            };

            let f = frame.clone();
            let target = Target::Flood(f.recipient.scope().expect("empty recipient"));
            task::spawn(async move { ep.send(f, target, exclude).await.unwrap() });
        }
    }
}

#[cfg(feature = "dashboard")]
mod metrics {
    use prometheus_client::{
        encoding::text::Encode,
        metrics::{counter::Counter, family::Family},
        registry::{Registry, Unit},
    };

    #[derive(Clone, Hash, PartialEq, Eq, Encode)]
    pub(super) struct Labels {
        pub recp_type: RecipientType,
        pub recp_id: types::Address,
    }

    #[derive(Clone, Hash, PartialEq, Eq)]
    pub(super) enum RecipientType {
        Standard,
        Flood,
    }

    impl From<&types::Recipient> for RecipientType {
        fn from(v: &types::Recipient) -> Self {
            match v {
                &types::Recipient::Standard(_) => Self::Standard,
                &types::Recipient::Flood(_) => Self::Flood,
            }
        }
    }

    // Manually implement Encode to produce eg. `recipient=standard` rather than `recipient=Standard`.
    impl Encode for RecipientType {
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
        pub bytes_total: Family<Labels, Counter>,
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
            registry.register_with_unit(
                "ratman_dispatch",
                "Total size of dispatched frames",
                Unit::Bytes,
                Box::new(self.bytes_total.clone()),
            );
        }
    }
}
