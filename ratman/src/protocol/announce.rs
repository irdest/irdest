use crate::{
    context::RatmanContext,
    core::{dispatch},
    storage::MetadataDb,
};

use libratman::{
    frame::{
        carrier::{AnnounceFrame, AnnounceFrameV1, CarrierFrameHeader, OriginDataV1, RouteDataV1},
        FrameGenerator,
    },
    tokio::time,
    types::{Address, ClientAuth, InMemoryEnvelope},
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

/// Periodically announce an address to the network
pub struct AddressAnnouncer {
    addr: Address,
    auth: ClientAuth,
    db: Arc<MetadataDb>,
}

impl AddressAnnouncer {
    /// Start a new address announcer with a client authenticator.  Even when
    /// the starting client goes away, this is used to keep the thread local key
    /// cache session alive
    pub fn new(addr: Address, auth: ClientAuth, ctx: &Arc<RatmanContext>) -> Self {
        ctx.meta_db.open_addr_key(addr, auth).unwrap();
        Self {
            addr,
            auth,
            db: Arc::clone(&ctx.meta_db),
        }
    }
}

// Drop the cached address key from the thread cache when the announcer goes out
// of scope.  This means that the address has been taken offline and all cached
// keys need to be wiped.
impl Drop for AddressAnnouncer {
    fn drop(&mut self) {
        self.db.close_addr_key(self.auth);
    }
}

impl AddressAnnouncer {
    pub(crate) async fn generate_announce(&self) -> AnnounceFrame {
        let origin = OriginDataV1::now();
        let origin_signature = {
            let mut origin_buf = vec![];
            origin.clone().generate(&mut origin_buf).unwrap();
            self.db
                .sign_message(self.auth, origin_buf.as_slice())
                .unwrap()
        };

        // Create a full announcement and encode it
        AnnounceFrame::V1(AnnounceFrameV1 {
            origin,
            origin_signature: origin_signature.to_bytes(),
            route: RouteDataV1 {
                mtu: 0,
                size_hint: 0,
            },
        })
    }

    pub(crate) async fn run(
        self,
        online: Arc<AtomicBool>,
        announce_delay: u16,
        ctx: Arc<RatmanContext>,
    ) {
        while online.load(Ordering::Acquire) {
            // Create a new announcement
            let announce = self.generate_announce().await;
            let announce_buffer = {
                let mut buf = vec![];
                announce.generate(&mut buf);
                buf
            };

            // Pack it into a carrier and handle the nested encoding
            let header =
                CarrierFrameHeader::new_announce_frame(self.addr, announce_buffer.len() as u16);

            // Pre-maturely mark this announcement as "known", so that
            // the switch locally will ignore it when it is inevitably
            // sent back to us.
            ctx.journal
                .save_as_known(&header.get_seq_id().unwrap().hash)
                .unwrap();

            // Send it into the network
            if let Err(e) = dispatch::flood_frame(
                &ctx.routes,
                &ctx.links,
                InMemoryEnvelope::from_header_and_payload(header, announce_buffer).unwrap(),
                None,
            )
            .await
            {
                error!("failed to flood announcement: {}", e)
            }

            trace!("Sent address announcement for {}", self.addr);

            // Wait some amount of time
            time::sleep(Duration::from_secs(announce_delay as u64)).await;
        }
    }
}
