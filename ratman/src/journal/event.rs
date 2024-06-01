use async_eris::{Block, BlockReference};
use libratman::{
    frame::carrier::{CarrierFrameHeader, ManifestFrame},
    types::{Address, Id, SequenceIdV1},
};
use serde::{Deserialize, Serialize};

use crate::{routes::RouteEntry, storage::block::StorageBlock};

use super::page::SerdeFrameType;

/// Events applied to the block partition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BlockEvent {
    /// Add a new block to the BlockRef->Block table
    Insert(StorageBlock),
    /// Remove an existing block, marking it for deletion
    Delete(BlockReference),
    /// Mark an existing block as invalid
    ///
    /// Invalid blocks can still be accessed locally but will never be
    /// transmitted to another network participant.  This can be used to stop
    /// sharing a file with others while keeping it available to re-share in the
    /// future
    Invalidate(BlockReference),
    /// Undo an invalidation event
    Revalidate(BlockReference),
    // fixme: wouldn't it make more sense to implement the link mechanism on the
    // ManifestJournal level?
    // Link(LinkMeta),
}

/// Events applied to the frame partition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FrameEvent {
    /// Add a new frame to the persistent frame journal
    Insert {
        seq: SequenceIdV1,
        header: SerdeFrameType<CarrierFrameHeader>,
        payload: Vec<u8>,
    },
    /// Remove an existing frame from the persistent frame journal
    Delete(SequenceIdV1),
}

/// Events applied to the manifest partition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ManifestEvent {
    Add {
        sender: Address,
        recipient: Address,
        manifest: SerdeFrameType<ManifestFrame>,
    },
    Remove {
        root_ref: Id,
    },
    CreateLink(LinkMeta),
}

/// Events applied to the manifest link partition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LinkMeta {
    /// Insert or update a manifest link
    Upsert {
        root_ref: Id,
        extra_blocks: Vec<BlockReference>,
        metadata: String,
    },
    /// Destroy an existing manifest link
    ///
    /// The underlying manifest and block data will remain unchanged; however in
    /// case of storage quota limits the data may be deleted in favour of
    /// keeping higher priority messages around.
    Destroy { root_ref: Id },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RouteEvent {
    Insert {
        peer: Address,
        link_id: Id,
        route_id: Id,
        route: RouteEntry,
    },
    Remove(RouteId),
}
