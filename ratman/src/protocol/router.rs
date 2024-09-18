// SPDX-FileCopyrightText: 2024 Katharina Fey <kookie@spacekookie.de>

use crate::{context::RatmanContext, procedures};
use libratman::{
    frame::{carrier::CarrierFrameHeader, FrameGenerator},
    tokio::{select, spawn, time::sleep},
    types::{Address, Ident32, InMemoryEnvelope, RouterMeta},
};
use std::{sync::Arc, time::Duration};

pub struct RouterAnnouncement {
    pub key_id: Ident32,
}

impl RouterAnnouncement {
    pub fn run(self: Arc<Self>, ctx: Arc<RatmanContext>) {
        spawn(async move {
            debug!("Start router announcer with key_id={}", self.key_id);

            // hardcoded 512MB of buffer because we don't have any quotas yet.
            let remaining = /* 512 MB */ (512 * 1024 * 1024) - ctx.journal.frames.0.disk_space();

            loop {
                let ctx = Arc::clone(&ctx);
                select! {
                    biased;
                    _ = ctx.tripwire.clone() => break,
                    _ = sleep(Duration::from_secs(30))  => {
                        let router_announce = RouterMeta {
                            key_id: self.key_id,
                            available_buffer: remaining,
                            known_peers: 0,
                        };

                        let announce_buffer = {
                            let mut buf = vec![];
                            router_announce.generate(&mut buf).unwrap();
                            buf
                        };

                        // Pack it into a carrier and handle the nested encoding
                        let header =
                            CarrierFrameHeader::new_announce_frame(Address(self.key_id), announce_buffer.len() as u16);

                        let full_anon_buffer =
                            InMemoryEnvelope::from_header_and_payload(header, announce_buffer).unwrap();


                        if let Err(e) = procedures::flood_frame(&ctx.routes, &ctx.links, full_anon_buffer, None).await {
                            error!("failed to send router announcement: {e}; stopping announcements");
                            break;
                        }
                    }
                }
            }
        });
    }
}
