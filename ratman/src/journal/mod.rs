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

use std::marker::PhantomData;

use fjall::{Keyspace, PartitionCreateOptions};
use libratman::{types::SequenceIdV1, Result};

use crate::routes::RouteEntry;

use self::{
    event::{BlockEvent, FrameEvent, LinkEvent, ManifestEvent, RouteEvent},
    page::{JournalPage, SerdeFrameType},
};

mod event;
mod page;

/// Combined frame and block journal
///
/// This component has three main parts to it
///
/// ## Frame Journal
///
/// Frames that can't be delivered, either because the local address
/// is offline, or because the remote address isn't reachable via the
/// currently available connections are given to the frame journal.
///
/// When an address comes online, the contents of this journal (for
/// that particular address) are then either given to the dispatcher,
/// or the collector.
///
///
/// ## Block Journal
///
/// The collector assembles frames into completed blocks that are
/// inserted into the block journal.  It is shared amongst all
/// addresses, meaning that if two users/ applications on the same
/// machine received the same message twice (for example via a flood
/// namespace), it is only kept in storage once.
///
/// When a manifest is received an assembler task is spawned which
/// checks the block journal for the required block hashes, then
/// assembles a complete message stream and hands it to the client API
/// handler.
///
/// ## Known frames page
///
/// To avoid endless replication of messages the journal keeps track
/// of frame IDs that it has seen before, even when the contents
/// aren't being saved.  This is an important mechanism in the case of
/// announcements, which will otherwise keep echoing through the
/// network forever... *makes haunting noises*.

pub struct Journal {
    db: Keyspace,
    pub frames: JournalPage<FrameEvent>,
    pub blocks: JournalPage<BlockEvent>,
    pub manifests: JournalPage<ManifestEvent>,
    pub routes: JournalPage<RouteEvent>,
    // todo: this doesn't make any sense lol
    pub seen_frames: JournalPage<SerdeFrameType<SequenceIdV1>>,
    pub links: JournalPage<LinkEvent>,
}

impl Journal {
    pub fn new(db: Keyspace) -> Result<Self> {
        let frames = JournalPage(
            ks.open_partition("journal.frames", PartitionCreateOptions::default())?,
            PhantomData,
        );
        let blocks = JournalPage(
            ks.open_partition("journal.blocks", PartitionCreateOptions::default())?,
            PhantomData,
        );
        let manifests = JournalPage(
            ks.open_partition("journal.manifests", PartitionCreateOptions::default())?,
            PhantomData,
        );
        let seen_frames = JournalPage(
            ks.open_partition("journal.seen_frames", PartitionCreateOptions::default())?,
            PhantomData,
        );
        let links = JournalPage(
            ks.open_partition("journal.links", PartitionCreateOptions::default())?,
            PhantomData,
        );

        Ok(Self {
            db,
            frames,
            blocks,
            manifests,
            seen_frames,
            links,
        })
    }
}
