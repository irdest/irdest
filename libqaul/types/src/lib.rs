//! A type fascade for `libqaul`
//!
//! Using this crate directly is usually not neccessary, instead you
//! should use `libqaul-sdk` to pull in the RPC adapter for the API.
//!
//! To learn more about how to write components for the qaul.net rpc
//! system (qrpc), check out the `qrpc-sdk` crate documentation.
//!
//! All types in this crate should be `Serialize` and `Deserialize` to
//! allow them to be re-used for higher-layer RPC protocols, such as
//! the HTTP server used for the `emberweb` UI client.

/// Re-export the core Identity from ratman
pub use ratman_identity::Identity;

pub mod contacts;
pub mod error;
pub mod messages;
pub mod services;
pub mod users;
pub mod diff;

pub mod rpc;

// // TODO: rpc feature gate
// pub(crate) mod types_capnp {
//     #![allow(unused)] // don't bother me pls
//     include!(concat!(env!("OUT_DIR"), "/schema/types_capnp.rs"));
// }

