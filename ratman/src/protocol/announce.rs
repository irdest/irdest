use crate::{config::ConfigTree, context::RatmanContext};
use async_std::{sync::Arc, task};
use bincode::Config;
use libratman::{
    netmod::InMemoryEnvelope,
    types::{
        frames::{AnnounceFrameV1, CarrierFrameHeader, FrameGenerator, OriginDataV1, RouteDataV1},
        Address, Id,
    },
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
    pub(crate) async fn run(
        self,
        online: Arc<AtomicBool>,
        cfg: &ConfigTree,
        ctx: Arc<RatmanContext>,
    ) {
        let announce_delay = cfg
            .get_subtree("ratmand")
            .and_then(|subtree| subtree.get_number_value("announce_delay"))
            .unwrap_or_else(|| {
                debug!("ratmand/announce_delay was not set, assuming default of 2 seconds");
                2
            }) as u64;

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

            // Create a full announcement and encode it
            let announce = AnnounceFrameV1 {
                origin,
                origin_signature: origin_signature.to_bytes(),
                route: RouteDataV1 {
                    mtu: 0,
                    size_hint: 0,
                },
            };

            let announcement_payload = {
                let mut buffer = vec![];
                announce.generate(&mut buffer);
                buffer
            };

            // Pack it into a carrier and handle the nested encoding
            let header = CarrierFrameHeader::new_announce_frame(
                self.address,
                announcement_payload.len() as u16,
            );

            // Send it into the network
            if let Err(e) = ctx
                .core
                .flood_frame(InMemoryEnvelope::from_header(header, announcement_payload).unwrap())
                .await
            {
                error!("failed to flood announcement: {}", e)
            }

            // Wait some amount of time
            task::sleep(Duration::from_secs(announce_delay)).await;
        }
    }
}
