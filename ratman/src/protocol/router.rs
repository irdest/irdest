// SPDX-FileCopyrightText: 2024 Katharina Fey <kookie@spacekookie.de>

use libratman::{
    tokio::{select, spawn},
    types::Ident32,
};
use std::sync::Arc;
use tripwire::Tripwire;

pub struct RouterAnnouncement {
    key_id: Ident32,
    tripwire: Tripwire,
}

impl RouterAnnouncement {
    pub fn run(self: Arc<Self>) {
        spawn(async move {
            debug!("Start router announcer with key_id={}", self.key_id);

            loop {
                let ctx = Arc::clone(&self);
                select! {
                    biased;
                    _ = ctx.tripwire.clone() => break,
                    res = announcer.run(announce_delay, &ctx) => {
                        // match res {
                        //     Ok(_) => {},
                        //     Err(e) => {
                        //         error!("failed to send announcement: {e}");
                        //         break;
                        //     }
                        // }
                    }
                }
            }

            // info!("Address announcer {} shut down!", address.pretty_string());
        });
    }
}
