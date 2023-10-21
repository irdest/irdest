//! Networking frames

use crate::types::{
    frames::{CarrierFrameV1, ProtoCarrierFrameMeta},
    Address,
};
use std::fmt::{self, Display};

/// Describes an endpoint's send target
///
/// This is different from a Recipient in that it doesn't encode
/// information about a user on the global network.  It's values are
/// used by one-to-many Endpoint implementors to desambiguate their
/// own routing tables without having to replicate the Ratman internal
/// routing table.
///
/// If your endpoint doesn't implement a one-to-many link (i.e. if
/// it's always one-to-one), just let this value to `Single(0)`
/// (`Target::default()`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    /// Send message to all reachable endpoints
    Flood(Address),
    /// Encodes a specific target ID
    Single(u16),
}

impl Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Flood(addr) => format!("Flood({})", addr),
                Self::Single(t) => format!("Peer({})", t),
            }
        )
    }
}

impl Default for Target {
    fn default() -> Self {
        Self::Single(0)
    }
}

/// Container for carrier frame metadata and a full message buffer
#[derive(Debug, Clone)]
pub struct InMemoryEnvelope {
    pub meta: ProtoCarrierFrameMeta,
    pub buffer: Vec<u8>,
}
