// SPDX-FileCopyrightText: 2024 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use async_eris::BlockReference;
use libratman::{
    frame::carrier::{CarrierFrameHeader, ManifestFrame},
    types::{Address, Ident32, Recipient},
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
    pub forwarded: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManifestLinkData {
    pub root_ref: Ident32,
    pub extra_blocks: Vec<BlockReference>,
    pub metadata: String,
}
