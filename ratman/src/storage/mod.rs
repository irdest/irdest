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
    crypto::{self, decrypt_raw, encrypt_chunk, encrypt_raw, Keypair},
    journal::page::CachePage,
    storage::{
        addr_key::{AddressData, EncryptedKey},
        block::IncompleteBlockData,
        link::LinkData,
        route::RouteData,
    },
};
use ed25519_dalek::{PublicKey, SecretKey, Signature};
use fjall::{Keyspace, PartitionCreateOptions};
use libratman::{
    types::{Address, ClientAuth, Id},
    Result,
};
use rand::rngs::OsRng;
use std::{
    borrow::BorrowMut, cell::RefCell, collections::BTreeMap, convert::TryInto, marker::PhantomData,
};
use x25519_dalek::SharedSecret;

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
// todo: currently this type handles some encryption operations directly even
// though there's no real reason for this.  Maybe move this code to the crypto
// module?
pub struct MetadataDb {
    db: Keyspace,
    pub addrs: CachePage<AddressData>,
    pub routes: CachePage<RouteData>,
    pub links: CachePage<LinkData>,
    pub incomplete: CachePage<IncompleteBlockData>,
}

impl MetadataDb {
    pub fn new(db: Keyspace) -> Result<Self> {
        let addrs = CachePage(
            db.open_partition("meta_addrs", PartitionCreateOptions::default())?,
            PhantomData,
        );
        let routes = CachePage(
            db.open_partition("meta_routes", PartitionCreateOptions::default())?,
            PhantomData,
        );
        let links = CachePage(
            db.open_partition("meta_links", PartitionCreateOptions::default())?,
            PhantomData,
        );
        let incomplete = CachePage(
            db.open_partition("meta_incomplete", PartitionCreateOptions::default())?,
            PhantomData,
        );

        Ok(Self {
            db,
            addrs,
            routes,
            links,
            incomplete,
        })
    }
}
