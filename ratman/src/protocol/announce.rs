use crate::{context::RatmanContext, crypto, procedures, storage::MetadataDb};
use ed25519_dalek::ed25519::signature::SignerMut;
use libratman::{
    frame::{
        carrier::{AnnounceFrame, AnnounceFrameV1, CarrierFrameHeader, OriginDataV1, RouteDataV1},
        FrameGenerator,
    },
    tokio::time,
    types::{AddrAuth, Address, Ident32, InMemoryEnvelope},
    Result,
};
use std::{sync::Arc, time::Duration};

/// Periodically announce an address to the network
pub struct AddressAnnouncer {
    addr: Address,
    pub(super) auth: AddrAuth,
    client_id: Ident32,
    db: Arc<MetadataDb>,
}

impl AddressAnnouncer {
    /// Start a new address announcer with a client authenticator.  Even when
    /// the starting client goes away, this is used to keep the thread local key
    /// cache session alive
    pub async fn new(
        addr: Address,
        auth: AddrAuth,
        client_id: Ident32,
        ctx: &Arc<RatmanContext>,
    ) -> Result<Self> {
        Ok(Self {
            addr,
            auth,
            client_id,
            db: Arc::clone(&ctx.meta_db),
        })
    }
}

impl AddressAnnouncer {
    pub(crate) async fn generate_announce(&self) -> Result<AnnounceFrame> {
        let origin = OriginDataV1::now();
        let origin_signature = {
            let mut origin_buf = vec![];
            origin.clone().generate(&mut origin_buf).unwrap();
            let mut key = crypto::get_addr_key(&self.db, self.addr, self.auth)?;

            // return signature
            key.inner.sign(origin_buf.as_slice())
        };

        // Create a full announcement and encode it
        Ok(AnnounceFrame::V1(AnnounceFrameV1 {
            origin,
            origin_signature: origin_signature.to_bytes(),
            route: RouteDataV1 {
                mtu: 0,
                size_hint: 0,
            },
        }))
    }

    pub(crate) async fn run(&self, announce_delay: u16, ctx: &Arc<RatmanContext>) -> Result<()> {
        // Create a new announcement
        let announce = self.generate_announce().await?;

        let announce_buffer = {
            let mut buf = vec![];
            announce.generate(&mut buf)?;
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
        procedures::flood_frame(
            &ctx.routes,
            &ctx.links,
            InMemoryEnvelope::from_header_and_payload(header, announce_buffer).unwrap(),
            None,
        )
        .await?;

        // Wait some amount of time
        time::sleep(Duration::from_secs(announce_delay as u64)).await;

        Ok(())
    }
}
