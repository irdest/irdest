//! Client API protocol definitions
//!
//!

mod addr;
mod contact;
mod link;
mod peer;
mod recv;

pub use addr::*;
pub use contact::*;
pub use link::*;
pub use peer::*;
pub use recv::*;

use crate::types::Id;

pub const CLIENT_API_VERSION: u8 = 1;

/// Sent from the router to the client when a client connects
pub struct Handshake {
    /// Indicate to the client which version of the protocol is used
    ///
    /// A client that connects with an older version MUST print an
    /// error to the user, indicating that the tools version they are
    /// using is not compatible with the Router version.
    pub proto_version: u8,
}

/// Sent from the router to the client on every 'ping'
pub struct Ping {
    /// Indicate to the client which subscription IDs are available
    ///
    /// A client can then decide to pull a particular subscription Id
    /// to get the next message stream for that subscription
    pub available_subscriptions: Vec<Id>,
}
