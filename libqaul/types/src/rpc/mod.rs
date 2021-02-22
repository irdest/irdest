//! RPC support module
//!
//! `libqaul-types` by itself simply provides a set of Rust types used
//! across libqaul and associated crates and services.  In order to
//! support encoding and decoding these types for the RPC layer, you
//! can enable the RPC module, which provides a set of builder
//! functions to transform types.

mod util;
mod error;
mod users;

pub use error::ConvertError;
pub(crate) use error::try_read;
