//! Message dispatcher
//!
//! Accepts an application byte stream on one end, and spits out a
//! sequence of encrypted, correctly sliced frames for a particular
//! transport MTU.

pub(crate) mod slicer;
