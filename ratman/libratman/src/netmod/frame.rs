//! Networking frames

use crate::types::Address;

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

impl Default for Target {
    fn default() -> Self {
        Self::Single(0)
    }
}
