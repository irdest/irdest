//! Utility module to handle verifying peers and initialising drivers
//!
//! Ratman can either be launched with a known set of peers, or it
//! must be configured to `accept_unknown_peers`.

use crate::util::DriverMap;
use libratman::{netmod::Endpoint, NetmodError, RatmanError};

/// A helper that parses, validates, and attaches peer data to drivers
///
/// The netmod endpoints themselves must already be allocated and
/// ready to use before this process can happen.
pub struct PeeringBuilder {
    drivers: DriverMap,
}

impl PeeringBuilder {
    /// Create a new peering builder with an existing mapping of netmods
    ///
    /// The strings used for identification are used as prefixes in
    /// the peer syntax.
    pub fn new(drivers: DriverMap) -> Self {
        Self { drivers }
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

        match self.drivers.get(driver_id) {
            Some(endpoint) => {
                // Ignore the peer_id for now
                let _peer_id = endpoint.start_peering(address_str).await;
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

    /// Dissolve this type and take the inner driver map
    pub fn consume(self) -> DriverMap {
        self.drivers
    }
}
