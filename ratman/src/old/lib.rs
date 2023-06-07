#[macro_use]
extern crate tracing;

pub mod scaffold;

pub mod clock;
mod core;
mod crypto;
mod data;
mod protocol;
mod router;
mod slicer;
mod storage;

#[cfg(feature = "daemon")]
pub mod daemon;

// Provide exports to the rest of the crate
pub(crate) use crate::{core::Core, data::Payload, protocol::Protocol, slicer::TransportSlicer};

// Public API facade
pub use crate::{
    data::{Message, MsgId},
    router::Router,
};
pub use netmod;
pub use types::{Address, Error, Recipient, Result, TimePair, ID_LEN};
