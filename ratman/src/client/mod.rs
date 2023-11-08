//! Ratman client interface module
//!
//! ## Implemented by ratcat:
//! 
//! - send (--multiple)
//!   - to-contact "[search by name/ note]"
//!   - to-address "[hex string]"
//!   - flood "[namespace]"
//! - receive (--count)
//!   - get-one
//!   - fetch-sub
//!
//! ## Implemented by ratctl:
//! 
//! - sub "[namespace]" (--timeout)
//!   - add
//!   - rm
//! - status
//!   - system
//!   - addr
//!   - link
//! - addr
//!   - create
//!   - up
//!   - down
//!   - delete
//! - peer
//!   - query
//!   - list
//! - link
//!   - add
//!   - up
//!   - down
//!   - rm
//! - 
