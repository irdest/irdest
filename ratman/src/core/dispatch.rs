//! Asynchronous Ratman routing core

use crate::{
    core::{Collector, DriverMap, EpTargetPair, RouteTable},
    Message, Result, Slicer,
};
use async_std::{sync::Arc, task};
use netmod::{Frame, Recipient, Target};

pub(crate) struct Dispatch {
    routes: Arc<RouteTable>,
    drivers: Arc<DriverMap>,
    collector: Arc<Collector>,
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
        })
    }

    pub(crate) async fn send_msg(&self, msg: Message) -> Result<()> {
        let r = msg.recipient;
        trace!("dispatching message to recpient: {:?}", r);

        // This is a hardcoded MTU for now.  We need to adapt the MTU
        // to the interface we're broadcasting on and we potentially
        // need a way to re-slice, or combine frames that we encounter
        // for better transmission metrics
        let frames = Slicer::slice(1312, msg);

        frames.into_iter().fold(Ok(()), |res, f| match (res, r) {
            (Ok(()), Recipient::User(_)) => task::block_on(async move { self.send_one(f).await }),

            (res, _) => res,
        })
    }

    /// Dispatch a single frame across the network
    pub(crate) async fn send_one(&self, frame: Frame) -> Result<()> {
        let EpTargetPair(epid, trgt) = match self
            .routes
            .resolve(match frame.recipient {
                Recipient::User(id) => id,
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

        let ep = self.drivers.get(epid as usize).await;
        Ok(ep.send(frame, trgt).await?)
    }

    pub(crate) async fn flood(&self, frame: Frame) -> Result<()> {
        for ep in self.drivers.get_all().await.into_iter() {
            let f = frame.clone();
            let target = Target::Flood(frame.recipient.scope());
            ep.send(f, target).await.unwrap();
        }

        Ok(())
    }

    /// Reflood a message to the network, except the previous interface
    pub(crate) async fn reflood(&self, frame: Frame, ep: usize) {
        for ep in self.drivers.get_without(ep).await.into_iter() {
            let f = frame.clone();
            let target = Target::Flood(f.recipient.scope());
            task::spawn(async move { ep.send(f, target).await.unwrap() });
        }
    }
}
