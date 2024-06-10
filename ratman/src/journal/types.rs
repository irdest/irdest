use async_eris::BlockReference;
use libratman::{
    frame::carrier::{CarrierFrameHeader, ManifestFrame},
    types::{Address, Id, Recipient},
};
use serde::{Deserialize, Serialize};

use crate::{journal::page::SerdeFrameType, storage::block::StorageBlock};

/// Events applied to the block partition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockData {
    pub data: StorageBlock,
    pub valid: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FrameData {
    pub header: SerdeFrameType<CarrierFrameHeader>,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManifestData {
    pub sender: Address,
    pub recipient: Recipient,
    pub manifest: SerdeFrameType<ManifestFrame>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManifestLinkData {
    pub root_ref: Id,
    pub extra_blocks: Vec<BlockReference>,
    pub metadata: String,
}
