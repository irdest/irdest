//! Microframe client API type definitions

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

/// Apply a simple filter for trust relationships
#[repr(C)]
pub enum TrustFilter {
    GreatEq(u8),
    Less(u8),
}
