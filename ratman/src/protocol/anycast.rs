use crate::{context::RatmanContext, procedures};
use libratman::{
    frame::carrier::CarrierFrameHeader,
    tokio::{sync::mpsc::Receiver, time},
    types::{Address, InMemoryEnvelope, Namespace, Recipient},
    Result,
};
use std::{sync::Arc, time::Duration};

pub struct AnycastProbeHandler {
    pub namespace: Namespace,
    pub self_addr: Address,
}

impl AnycastProbeHandler {
    pub async fn execute(
        self,
        ctx: Arc<RatmanContext>,
        timeout: Duration,
        mut rx: Receiver<(Address, Duration)>,
    ) -> Result<Vec<(Address, Duration)>> {
        let mut responses = vec![];
        let header = CarrierFrameHeader::new_anycast_probe_frame(
            self.self_addr,
            Recipient::Namespace(self.namespace),
        );

        procedures::flood_frame(
            &ctx.routes,
            &ctx.links,
            InMemoryEnvelope::from_header_and_payload(header, vec![])?,
            None,
        )
        .await?;

        // Mostly ignore timeout error because we expect it to happen.  As long as
        // we received _something_ we don't need to care
        if time::timeout(timeout, async {
            if let Some((addr, duration)) = rx.recv().await {
                responses.push((addr, duration));
            }
        })
        .await
        .is_err()
            && responses.len() == 0
        {
            warn!("Anycast probe timeout reached, but no responses were received!");
        };

        Ok(responses)
    }
}
