//! Ratman storage journal module
//!
//! This module keeps track of network traffic events and payloads on disk to
//! prevent data loss in case of crashes or power failure.  The storage backend
//! is provided by [fjall.rs](fjall).
//!
//! Each type of data is held in its own JournalPage (also called a journal
//! partition), which has an associated type and encoding/decoding mechanism
//! with bincode.  The overall Journal is a collection of multiple journal
//! partitions for different types of data:
//!
//! - In-flight frames: these are individual packets that couldn't yet be
//! delivired to their recipient, either originating on the local node or some
//! remote.  Cached frames are explicitly not able to assemble into a full block
//! and are deleted first in case of storage quota limitations.
//!
//! - ERIS blocks: these are encrypted content blocks for messages that are
//! either still assembling or are being cached for a remote network
//! participant.
//!
//! - Stream manifests: each message stream generates a manifest, which encodes
//! metadata about its origin, type, and which blocks are associated with it.  A
//! manifest is needed to
//!
//! - Routing table: keep track of known peers via their links and various
//! metrics like MTU, uptime, and average ping.
//!
//! - Known frame IDs: keep track of known frame IDs to avoid re-broadcasting
//! the same messages infinitely.
//!
//! - Link metadata: certain message streams have associations between them, or
//! can be tagged with additional information for importance to prevent them
//! from being cleaned from the journal in case of storage quota limits.

use self::{
    event::{BlockEvent, FrameEvent, LinkEvent, ManifestEvent, RouteEvent},
    page::{JournalCache, JournalPage, SerdeFrameType},
};
use crate::routes::RouteEntry;
use fjall::{Keyspace, PartitionCreateOptions};
use libratman::{types::Id, Result};
use std::marker::PhantomData;

mod event;
mod page;

/// Fully integrated storage journal
pub struct Journal {
    db: Keyspace,
    /// Single cached frames that haven't yet been delivired
    pub frames: JournalPage<FrameEvent>,
    /// Fully cached blocks that may already have been delivered
    pub blocks: JournalPage<BlockEvent>,
    /// Fully cached manifests for existing block streams
    pub manifests: JournalPage<ManifestEvent>,
    /// A simple lookup set for known frame IDs
    pub seen_frames: JournalCache<Id>,
    /// Route metadata table
    pub routes: JournalPage<RouteEvent>,
    /// Message stream metadata table
    pub links: JournalPage<LinkEvent>,
}

impl Journal {
    pub fn new(db: Keyspace) -> Result<Self> {
        let frames = JournalPage(
            db.open_partition("journal.frames", PartitionCreateOptions::default())?,
            PhantomData,
        );

        let blocks = JournalPage(
            db.open_partition("journal.blocks", PartitionCreateOptions::default())?,
            PhantomData,
        );

        let manifests = JournalPage(
            db.open_partition("journal.manifests", PartitionCreateOptions::default())?,
            PhantomData,
        );

        let seen_frames = JournalCache(
            db.open_partition("journal.seen_frames", PartitionCreateOptions::default())?,
            PhantomData,
        );

        let routes = JournalPage(
            db.open_partition("meta.routes", PartitionCreateOptions::default())?,
            PhantomData,
        );

        let links = JournalPage(
            db.open_partition("meta.links", PartitionCreateOptions::default())?,
            PhantomData,
        );

        Ok(Self {
            db,
            frames,
            blocks,
            manifests,
            seen_frames,
            routes,
            links,
        })
    }

    pub fn is_unknown(&self, frame_id: &Id) -> Result<bool> {
        self.seen_frames.get(frame_id)
    }

    pub fn save_as_known(&self, frame_id: &Id) -> Result<()> {
        self.seen_frames.insert(frame_id)
    }
}
