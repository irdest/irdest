//! Utility module to handle verifying peers and initialising drivers
//!
//! Ratman can either be launched with a known set of peers, or it
//! must be configured to `accept_unknown_peers`.

use crate::{links::LinksMap, storage::MetadataDb};
use libratman::{types::RouterMeta, NetmodError, RatmanError};
use std::sync::Arc;

/// A helper that parses, validates, and attaches peer data to drivers
///
/// The netmod endpoints themselves must already be allocated and
/// ready to use before this process can happen.
pub struct PeeringBuilder {
    links: Arc<LinksMap>,
    meta_db: Arc<MetadataDb>,
}

impl PeeringBuilder {
    /// Create a new peering builder with an existing mapping of netmods
    ///
    /// The strings used for identification are used as prefixes in
    /// the peer syntax.
    pub(crate) fn new(links: Arc<LinksMap>, meta_db: Arc<MetadataDb>) -> Self {
        Self { links, meta_db }
    }

    /// Attach a peer to one of the existing drivers
    ///
    /// This function will log errors that are encountered, but not
    /// fail.
    pub async fn attach(&mut self, peer: &str) -> Result<(), RatmanError> {
        let (driver_id, address_str) = match peer.split_once(':') {
            Some(split) => split,
            None => {
                error!("Invalid peer line: '{}', peer.  Refer to peer syntax documentation and try again", peer);
                return Err(RatmanError::Netmod(NetmodError::InvalidPeer(peer.into())));
            }
        };

        match self.links.get_by_name(driver_id).await {
            Some(endpoint) => {
                let router_meta = RouterMeta {
                    key_id: self.meta_db.router_id(),
                    known_peers: self.meta_db.addrs.len()?,
                    available_buffer: 0,
                };

                // Ignore the peer_id for now
                debug!("Start peering request with {address_str}");
                let _peer_id = endpoint.start_peering(address_str, router_meta).await;
                Ok(())
            }
            None => {
                error!(
                    "unknown driver identifier: {}!  Peer '{}' will be ignored",
                    driver_id, peer
                );
                Err(RatmanError::Netmod(NetmodError::InvalidPeer(peer.into())))
            }
        }
    }
}
