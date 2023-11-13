//! Various Frame abstractions for Irdest tools
//!
//! A frame is a self-contained packet with a unified way of parsing
//! an incoming data stream, usually ending with a payload length,
//! which should be loaded after the given header.

pub mod carrier;
pub mod micro;
