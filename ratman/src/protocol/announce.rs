use crate::{context::RatmanContext, dispatch::new_carrier_v1};
use async_std::{sync::Arc, task};
use libratman::types::{
    frames::{AnnounceFrameV1, CarrierFrame, FrameGenerator, OriginDataV1, RouteDataV1},
    Address, Id,
};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

/// Periodically announce an address to the network
pub struct AddressAnnouncer {
    address: Address,
    key_id: Id,
}

impl AddressAnnouncer {
    pub(crate) async fn run(self, online: Arc<AtomicBool>, ctx: Arc<RatmanContext>) {
        while online.load(Ordering::Acquire) {
            // Create some OriginData and sign it
            let origin = OriginDataV1::now();

            let origin_signature = {
                let mut origin_buf = vec![];
                origin.clone().generate(&mut origin_buf).unwrap();
                ctx.keys
                    .sign_message(self.address, origin_buf.as_slice())
                    .await
                    .unwrap()
            };

            // Create a full announcement
            let announce = AnnounceFrameV1 {
                origin,
                origin_signature: origin_signature.to_bytes(),
                route: RouteDataV1 {
                    mtu: 0,
                    size_hint: 0,
                },
            };

            // Pack it into a carrier and handle the nested encoding
            let mut carrier = new_carrier_v1(None, self.address, None);
            announce.generate(&mut carrier.payload).unwrap();
            let meta = carrier.as_meta();
            let mut buffer = vec![];
            CarrierFrame::V1(carrier).generate(&mut buffer).unwrap();

            // Send it into the network
            ctx.core
                .flood_frame(libratman::netmod::InMemoryEnvelope { meta, buffer })
                .await
                .unwrap();

            // Wait some amount of time
            task::sleep(Duration::from_secs(2)).await;
        }
    }
}
