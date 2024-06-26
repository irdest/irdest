// SPDX-FileCopyrightText: 2024 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

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
//! - Known frame IDs: keep track of known frame IDs to avoid re-broadcasting
//! the same messages infinitely.
//!

use self::{
    page::{CachePage, JournalCache, SerdeFrameType},
    types::{BlockData, FrameData, ManifestData},
};
use crate::storage::route::RouteData;

use fjall::{compaction::SizeTiered, Keyspace, PartitionCreateOptions, PartitionHandle};
use libratman::{
    frame::{carrier::ManifestFrame, FrameParser},
    tokio::task::spawn_blocking,
};
use libratman::{
    types::{Ident32, InMemoryEnvelope},
    Result,
};
use std::{marker::PhantomData, sync::Arc};

pub mod page;
pub mod types;

#[cfg(test)]
mod test;

/// Fully integrated storage journal
///
/// For latency critical insertions it is recommended to use the dispatch queue
/// (`queue_x` functions) instead of directly accessing the database.  For
/// non-latency critical insertions and all reads use the database access
/// functions directly.
///
/// Warning: if a later read depends on the immediate availability of a previous
/// insert it is highly recommended not to use the dispatch queue.
pub struct Journal {
    #[allow(unused)]
    db: Keyspace,
    /// Single cached frames that haven't yet been delivired
    pub frames: CachePage<FrameData>,
    /// Fully cached blocks that may already have been delivered
    pub blocks: CachePage<BlockData>,
    /// Fully cached manifests for existing block streams
    pub manifests: CachePage<ManifestData>,
    /// A simple lookup set for known frame IDs
    pub seen_frames: JournalCache<Ident32>,
    /// Route metadata table
    pub routes: CachePage<RouteData>,
    // /// Message stream metadata table
    // pub links: CachePage<LinkData>,
}

fn setup_hot_partition(name: &str, db: &Keyspace) -> Result<PartitionHandle> {
    let part = db.open_partition(name, options())?;
    part.set_compaction_strategy(Arc::new(SizeTiered::default()));
    part.set_max_memtable_size(512 * 1024 * 1024); // 512 MB of write cache
    Ok(part)
}

fn options() -> PartitionCreateOptions {
    PartitionCreateOptions::default()
        // .level_ratio(4)
        // .level_count(4)
        .block_size(32 * 1024)
}

impl Journal {
    pub fn new(db: Keyspace) -> Result<Self> {
        let frames = CachePage(setup_hot_partition("frame_data", &db)?, PhantomData);
        let blocks = CachePage(setup_hot_partition("block_data", &db)?, PhantomData);
        let manifests = CachePage(
            db.open_partition("blocks_manifests", options())?,
            PhantomData,
        );
        let seen_frames = JournalCache(db.open_partition("frames_seen", options())?, PhantomData);
        let routes = CachePage(db.open_partition("meta_routes", options())?, PhantomData);

        Ok(Self {
            db,
            frames,
            blocks,
            manifests,
            seen_frames,
            routes,
            // links,
        })
    }

    pub fn is_unknown(&self, frame_id: &Ident32) -> Result<bool> {
        self.seen_frames.get(frame_id)
    }

    pub fn save_as_known(&self, frame_id: &Ident32) -> Result<()> {
        self.seen_frames.insert(frame_id)
    }

    pub async fn num_blocks(self: &Arc<Self>) -> Result<u64> {
        let this = Arc::clone(&self);
        Ok(spawn_blocking(move || this.blocks.0.len().map(|l| l as u64)).await??)
    }

    /// Store a frame in the journal
    ///
    /// Frame keys are composed of the block ID and the number in sequence
    /// (`<seq>::<num>`).  Frames without a sequence ???
    pub async fn queue_frame(
        &self,
        InMemoryEnvelope { header, buffer }: InMemoryEnvelope,
    ) -> Result<()> {
        let seq_id = header.get_seq_id().unwrap();

        self.frames
            .insert(
                format!("{}::{}", seq_id.hash, seq_id.num),
                &FrameData {
                    header: SerdeFrameType::from(header),
                    payload: buffer,
                },
            )
            .await
    }

    pub async fn queue_manifest(&self, env: InMemoryEnvelope) -> Result<()> {
        let (_, manifest) = ManifestFrame::parse(env.get_payload_slice())?;
        let seq_id = env.header.get_seq_id().unwrap();

        self.manifests
            .insert(
                seq_id.hash.to_string(),
                &ManifestData {
                    sender: env.header.get_sender(),
                    recipient: env.header.get_recipient().unwrap(),
                    manifest: SerdeFrameType::from(manifest?),
                    forwarded: false,
                },
            )
            .await?;
        Ok(())
    }
}
