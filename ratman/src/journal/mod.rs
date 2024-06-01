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

use crate::routes::RouteEntry;

use self::{event::{BlockEvent, FrameEvent, ManifestEvent}, page::JournalPage};

mod event;
mod page;


pub struct Journal {
    frames: JournalPage<FrameEvent>,
    blocks: JournalPage<BlockEvent>,
    manifests: JournalPage<ManifestEvent>,
    routes: JournalPage<RouteEvent>,
}
