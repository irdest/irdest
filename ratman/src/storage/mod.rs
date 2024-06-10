//! This module handles several storage engines.  The backing database is fjall, a
//!
//! - Block storage: keep track of full blocks
//!
//! - Frame storage: keep track of in-flight frames that don't fully assemble a
//! block (yet)
//!
//! - Peer metadata: persistent routing tables
//!
//! -

use crate::{
    journal::page::CachePage,
    storage::{addr_key::StorageAddress, link::LinkData, route::RouteData},
};

pub mod addr_key;
pub mod block;
pub mod link;
pub mod route;

/// Metadata database handle
///
/// This database keeps track of various bits of metadata which aren't directly
/// related to the flow of message streams through an Irdest network.
///
/// - Registered addresses and their encrypted private key information
///
/// - Routing table: keep track of known peers via their links and various
/// metrics like MTU, uptime, and average ping.
///
/// - Link metadata: certain message streams have associations between them, or
/// can be tagged with additional information for importance to prevent them
/// from being cleaned from the journal in case of storage quota limits.
pub struct MetadataDb {
    pub addrs: CachePage<StorageAddress>,
    pub routes: CachePage<RouteData>,
    pub links: CachePage<LinkData>,
}
