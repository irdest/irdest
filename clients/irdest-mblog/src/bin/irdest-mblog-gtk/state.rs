use anyhow::{anyhow, Result};
use irdest_mblog::{Message, Recipient, NAMESPACE};
use ratman_client::types::Recipient;
use ratman_client::RatmanIpc;
use std::convert::TryFrom;

/// Central app state type which handles connection to Ratman
pub struct AppState {
    ipc: RatmanIpc,
    db: sled::Db,
}

impl AppState {
    pub fn new(ipc: ratman_client::RatmanIpc, db: sled::Db) -> Self {
        Self { ipc, db }
    }

    pub async fn next(&self) -> Option<Result<Message>> {
        self.ipc.next().await.and_then(|(tt, ratmsg)| {
            // Filter out flood messages for the wrong namespace.
            if let Recipient::Flood(ns) = ratmsg.get_recipient() {
                if ns != NAMESPACE.into() {
                    return None;
                }
            }

            Some(Message::try_from(&ratmsg))
        })
    }
}
